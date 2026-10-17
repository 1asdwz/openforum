use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Local;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::access;
use crate::dto;
use crate::error::{ApiError, ApiResult};
use crate::gate::{self, GateKind};
use crate::json as j;
use crate::models::Post;
use crate::security::CurrentUser;
use crate::state::AppState;
use crate::vote;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default, rename = "forumId")]
    pub forum_id: Option<i64>,
    pub q: Option<String>,
    #[serde(default = "default_hot")]
    pub sort: String,
    #[serde(default)]
    pub page: i32,
    #[serde(default = "default_size")]
    pub size: i32,
}

fn default_hot() -> String {
    "hot".to_string()
}
fn default_size() -> i32 {
    20
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostCreateRequest {
    pub forum_id: Option<i64>,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostUpdateRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub value: Option<i32>,
}

fn hot_score(post: &Post) -> f64 {
    let base = post.score as f64 + post.comment_count as f64 * 2.0;
    let days = (Local::now().naive_local() - post.created_at).num_days();
    let recency = if days < 7 {
        (7 - days).max(0) as f64 * 5.0
    } else {
        0.0
    };
    base + recency
}

async fn to_summary(pool: &MySqlPool, post: &Post) -> Result<Value, ApiError> {
    let forum: Option<(String, String)> =
        sqlx::query_as("SELECT slug, name FROM sub_forums WHERE id = ?")
            .bind(post.forum_id)
            .fetch_optional(pool)
            .await?;
    let (forum_slug, forum_name) = match forum {
        Some((s, n)) => (Some(s), Some(n)),
        None => (None, None),
    };
    let author = dto::author(pool, post.author_id).await?;
    Ok(json!({
        "id": post.id,
        "forumId": post.forum_id,
        "forumSlug": forum_slug,
        "forumName": forum_name,
        "title": post.title,
        "contentPreview": dto::preview(post.content.as_deref()),
        "upvotes": post.upvotes,
        "downvotes": post.downvotes,
        "score": post.score,
        "commentCount": post.comment_count,
        "isPinned": post.pinned,
        "author": author,
        "createdAt": j::jts(post.created_at),
    }))
}

async fn to_detail(
    state: &AppState,
    post: &Post,
    user_id: Option<i64>,
    is_admin: bool,
) -> Result<Value, ApiError> {
    let forum = crate::routes::forums::find_by_id(&state.pool, post.forum_id).await?;
    let mut can_vote = false;
    if user_id.is_some() {
        if let Some(forum) = forum.as_ref() {
            let verdict =
                gate::check(&state.pool, user_id, is_admin, forum, GateKind::Operate).await?;
            can_vote = verdict.passed;
        }
    }
    let forum_slug = forum.as_ref().map(|f| f.slug.clone());
    let forum_name = forum.as_ref().map(|f| f.name.clone());
    let my_vote = vote::my_vote(&state.pool, user_id, "POST", post.id).await?;
    let author = dto::author(&state.pool, post.author_id).await?;
    Ok(json!({
        "id": post.id,
        "forumId": post.forum_id,
        "forumSlug": forum_slug,
        "forumName": forum_name,
        "title": post.title,
        "content": post.content,
        "upvotes": post.upvotes,
        "downvotes": post.downvotes,
        "score": post.score,
        "commentCount": post.comment_count,
        "isPinned": post.pinned,
        "author": author,
        "canVote": can_vote,
        "myVote": my_vote,
        "createdAt": j::jts(post.created_at),
        "updatedAt": j::jts(post.updated_at),
    }))
}

async fn require_post(pool: &MySqlPool, id: i64) -> Result<Post, ApiError> {
    sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ? AND is_removed = 0")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::not_found("帖子不存在"))
}

async fn assert_can_edit(
    pool: &MySqlPool,
    post: &Post,
    user_id: i64,
) -> Result<(), ApiError> {
    if user_id == post.author_id {
        return Ok(());
    }
    if let Some(forum) = crate::routes::forums::find_by_id(pool, post.forum_id).await? {
        if access::can_manage(pool, &forum, Some(user_id)).await? {
            return Ok(());
        }
    }
    Err(ApiError::forbidden("只有作者或坛主可以操作"))
}

async fn recount_posts(conn: &mut sqlx::MySqlConnection, forum_id: i64) -> Result<(), ApiError> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM posts WHERE forum_id = ? AND is_removed = 0")
            .bind(forum_id)
            .fetch_one(&mut *conn)
            .await?;
    sqlx::query("UPDATE sub_forums SET post_count = ? WHERE id = ?")
        .bind(count as i32)
        .bind(forum_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

fn order_posts(posts: &mut [Post], sort: &str) {
    let key = if sort.trim().is_empty() {
        "hot".to_string()
    } else {
        sort.to_lowercase()
    };
    posts.sort_by(|a, b| {
        let pinned = b.pinned.cmp(&a.pinned);
        if pinned != std::cmp::Ordering::Equal {
            return pinned;
        }
        match key.as_str() {
            "new" => b.created_at.cmp(&a.created_at),
            "top" => b.score.cmp(&a.score),
            _ => hot_score(b)
                .partial_cmp(&hot_score(a))
                .unwrap_or(std::cmp::Ordering::Equal),
        }
    });
}

/// GET /posts
pub async fn list(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(qp): Query<ListQuery>,
) -> ApiResult<Json<Value>> {
    let user_id = user.0.as_ref().map(|u| u.id);
    let is_admin = user.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);

    if let Some(fid) = qp.forum_id {
        if let Some(forum) = crate::routes::forums::find_by_id(&state.pool, fid).await? {
            gate::ensure(&state.pool, user_id, is_admin, &forum, GateKind::Browse).await?;
        }
    }

    let mut posts: Vec<Post> = match qp.q.as_deref() {
        Some(q) if !q.trim().is_empty() => {
            let like = format!("%{}%", q.trim());
            sqlx::query_as::<_, Post>(
                "SELECT * FROM posts WHERE is_removed = 0 AND title LIKE ?",
            )
            .bind(&like)
            .fetch_all(&state.pool)
            .await?
        }
        _ => match qp.forum_id {
            Some(fid) => {
                sqlx::query_as::<_, Post>(
                    "SELECT * FROM posts WHERE is_removed = 0 AND forum_id = ?",
                )
                .bind(fid)
                .fetch_all(&state.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE is_removed = 0")
                    .fetch_all(&state.pool)
                    .await?
            }
        },
    };

    let allowed = gate::allowed_forum_ids(&state.pool, user_id, is_admin).await?;
    posts.retain(|post| allowed.contains(&post.forum_id));

    order_posts(&mut posts, &qp.sort);
    let total = posts.len() as i64;
    let (slice, _) = j::slice_page(posts, qp.page, qp.size);
    let mut content = Vec::with_capacity(slice.len());
    for post in &slice {
        content.push(to_summary(&state.pool, post).await?);
    }
    Ok(j::ok(j::PageResponse::of(content, qp.page, qp.size, total)))
}

/// GET /posts/{id}
pub async fn detail(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let post = require_post(&state.pool, id).await?;
    let user_id = user.0.as_ref().map(|u| u.id);
    let is_admin = user.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    if let Some(forum) = crate::routes::forums::find_by_id(&state.pool, post.forum_id).await? {
        gate::ensure(&state.pool, user_id, is_admin, &forum, GateKind::Browse).await?;
    }
    Ok(j::ok(to_detail(&state, &post, user_id, is_admin).await?))
}

/// POST /posts
pub async fn create(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<PostCreateRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum_id = req
        .forum_id
        .ok_or_else(|| ApiError::bad_request("forumId forumId 不能为空"))?;
    if req.title.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Err(ApiError::bad_request("title 标题不能为空"));
    }
    let title = req.title.as_deref().unwrap().trim().to_string();
    if title.chars().count() > 300 {
        return Err(ApiError::bad_request("title 长度不能超过 300"));
    }
    if req.content.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Err(ApiError::bad_request("content 内容不能为空"));
    }
    let content = req.content.as_deref().unwrap().to_string();

    let forum = crate::routes::forums::find_by_id(&state.pool, forum_id)
        .await?
        .ok_or_else(|| ApiError::not_found("子论坛不存在"))?;
    gate::ensure(&state.pool, Some(me.id), me.is_admin(), &forum, GateKind::Operate).await?;

    // 帖子插入与论坛帖子数重算需原子完成（对齐 Java `PostService.create` 的 @Transactional）。
    let mut tx = state.pool.begin().await?;
    let result = sqlx::query(
        "INSERT INTO posts (forum_id, author_id, title, content, upvotes, downvotes, score, comment_count, \
         is_pinned, is_removed, created_at, updated_at) VALUES (?, ?, ?, ?, 0, 0, 0, 0, 0, 0, NOW(), NOW())",
    )
    .bind(forum.id)
    .bind(me.id)
    .bind(&title)
    .bind(&content)
    .execute(&mut *tx)
    .await?;
    let post_id = result.last_insert_id() as i64;
    recount_posts(&mut tx, forum.id).await?;
    tx.commit().await?;

    let post = require_post(&state.pool, post_id).await?;
    Ok(j::ok(to_detail(&state, &post, Some(me.id), me.is_admin()).await?))
}

/// PUT /posts/{id}
pub async fn update(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<PostUpdateRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let mut post = require_post(&state.pool, id).await?;
    assert_can_edit(&state.pool, &post, me.id).await?;
    if let Some(t) = req.title {
        if !t.trim().is_empty() {
            post.title = t;
        }
    }
    if let Some(c) = req.content {
        post.content = Some(c);
    }
    sqlx::query("UPDATE posts SET title = ?, content = ?, updated_at = NOW() WHERE id = ?")
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.id)
        .execute(&state.pool)
        .await?;
    let post = require_post(&state.pool, id).await?;
    Ok(j::ok(to_detail(&state, &post, Some(me.id), me.is_admin()).await?))
}

/// DELETE /posts/{id}
pub async fn delete(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let post = require_post(&state.pool, id).await?;
    assert_can_edit(&state.pool, &post, me.id).await?;
    // 软删除、清票与帖子数重算需原子完成（对齐 Java `PostService.delete`）。
    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE posts SET is_removed = 1, updated_at = NOW() WHERE id = ?")
        .bind(post.id)
        .execute(&mut *tx)
        .await?;
    vote::clear_votes(&mut tx, "POST", post.id).await?;
    recount_posts(&mut tx, post.forum_id).await?;
    tx.commit().await?;
    Ok(j::ok_empty())
}

/// POST /posts/{id}/vote
pub async fn post_vote(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<VoteRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let post = require_post(&state.pool, id).await?;
    let forum = crate::routes::forums::find_by_id(&state.pool, post.forum_id)
        .await?
        .ok_or_else(|| ApiError::not_found("子论坛不存在"))?;
    gate::ensure(&state.pool, Some(me.id), me.is_admin(), &forum, GateKind::Operate).await?;
    let value = req.value.unwrap_or(0);
    if !(-1..=1).contains(&value) {
        return Err(ApiError::bad_request("投票值只能是 1、-1 或 0"));
    }
    // 票记录变更与帖子计数写回需原子完成（对齐 Java `PostService.vote` 的 @Transactional）。
    let mut tx = state.pool.begin().await?;
    let outcome = vote::apply(
        &mut tx,
        me.id,
        "POST",
        post.id,
        value,
        post.upvotes,
        post.downvotes,
    )
    .await?;
    sqlx::query("UPDATE posts SET upvotes = ?, downvotes = ?, score = ? WHERE id = ?")
        .bind(outcome.upvotes)
        .bind(outcome.downvotes)
        .bind(outcome.score)
        .bind(post.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let post = require_post(&state.pool, id).await?;
    Ok(j::ok(to_detail(&state, &post, Some(me.id), me.is_admin()).await?))
}

/// POST /posts/{id}/pin
pub async fn pin(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let post = require_post(&state.pool, id).await?;
    let forum = crate::routes::forums::find_by_id(&state.pool, post.forum_id)
        .await?
        .ok_or_else(|| ApiError::not_found("子论坛不存在"))?;
    if !access::can_manage(&state.pool, &forum, Some(me.id)).await? {
        return Err(ApiError::forbidden("需要坛主或管理员权限"));
    }
    sqlx::query("UPDATE posts SET is_pinned = ? WHERE id = ?")
        .bind(!post.pinned)
        .bind(post.id)
        .execute(&state.pool)
        .await?;
    let post = require_post(&state.pool, id).await?;
    Ok(j::ok(to_detail(&state, &post, Some(me.id), me.is_admin()).await?))
}

/// 供 users 模块复用：GET /users/{id}/posts
pub async fn list_by_author(
    state: &AppState,
    author_id: i64,
    user_id: Option<i64>,
    is_admin: bool,
    sort: &str,
    page: i32,
    size: i32,
) -> ApiResult<Json<Value>> {
    let allowed = gate::allowed_forum_ids(&state.pool, user_id, is_admin).await?;
    let mut posts: Vec<Post> =
        sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE author_id = ? AND is_removed = 0")
            .bind(author_id)
            .fetch_all(&state.pool)
            .await?;
    posts.retain(|post| allowed.contains(&post.forum_id));
    order_posts(&mut posts, sort);
    let total = posts.len() as i64;
    let (slice, _) = j::slice_page(posts, page, size);
    let mut content = Vec::with_capacity(slice.len());
    for post in &slice {
        content.push(to_summary(&state.pool, post).await?);
    }
    Ok(j::ok(j::PageResponse::of(content, page, size, total)))
}

/// 供评论模块复用：重算帖子的评论数（需在调用方事务内执行）。
pub async fn recount_comments(conn: &mut sqlx::MySqlConnection, post_id: i64) -> Result<(), ApiError> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM comments WHERE post_id = ? AND is_removed = 0")
            .bind(post_id)
            .fetch_one(&mut *conn)
            .await?;
    sqlx::query("UPDATE posts SET comment_count = ? WHERE id = ?")
        .bind(count as i32)
        .bind(post_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// 保留：帖子存在性检查（供其它模块使用）。
#[allow(dead_code)]
pub async fn post_exists(pool: &MySqlPool, id: i64) -> Result<bool, ApiError> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM posts WHERE id = ? AND is_removed = 0 LIMIT 1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}
