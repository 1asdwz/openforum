use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::access;
use crate::error::{ApiError, ApiResult};
use crate::gate::{self, GateKind};
use crate::json as j;
use crate::models::Comment;
use crate::routes::posts;
use crate::security::CurrentUser;
use crate::state::AppState;
use crate::vote;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCreateRequest {
    pub post_id: Option<i64>,
    pub content: Option<String>,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub value: Option<i32>,
}

async fn author_fields(pool: &MySqlPool, user_id: i64) -> Result<(Option<String>, Option<String>), ApiError> {
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT username, avatar FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(match row {
        Some((u, a)) => (Some(u), a),
        None => (None, None),
    })
}

fn build_tree(
    comment: &Comment,
    by_parent: &HashMap<i64, Vec<Comment>>,
    authors: &HashMap<i64, (Option<String>, Option<String>)>,
) -> Value {
    let (name, avatar) = authors
        .get(&comment.author_id)
        .cloned()
        .unwrap_or((None, None));
    let children: Vec<Value> = by_parent
        .get(&comment.id)
        .map(|list| list.iter().map(|c| build_tree(c, by_parent, authors)).collect())
        .unwrap_or_default();
    json!({
        "id": comment.id,
        "postId": comment.post_id,
        "authorId": comment.author_id,
        "authorName": name,
        "authorAvatar": avatar,
        "parentId": comment.parent_id,
        "content": comment.content,
        "upvotes": comment.upvotes,
        "downvotes": comment.downvotes,
        "score": comment.score,
        "createdAt": j::jts(comment.created_at),
        "children": children,
        "postTitle": Value::Null,
    })
}

fn build_single(comment: &Comment, name: Option<String>, avatar: Option<String>) -> Value {
    json!({
        "id": comment.id,
        "postId": comment.post_id,
        "authorId": comment.author_id,
        "authorName": name,
        "authorAvatar": avatar,
        "parentId": comment.parent_id,
        "content": comment.content,
        "upvotes": comment.upvotes,
        "downvotes": comment.downvotes,
        "score": comment.score,
        "createdAt": j::jts(comment.created_at),
        "children": Vec::<Value>::new(),
        "postTitle": Value::Null,
    })
}

async fn load_authors(pool: &MySqlPool, comments: &[Comment]) -> Result<HashMap<i64, (Option<String>, Option<String>)>, ApiError> {
    let mut map: HashMap<i64, (Option<String>, Option<String>)> = HashMap::new();
    for c in comments {
        if map.contains_key(&c.author_id) {
            continue;
        }
        let fields = author_fields(pool, c.author_id).await?;
        map.insert(c.author_id, fields);
    }
    Ok(map)
}

async fn require_comment(pool: &MySqlPool, id: i64) -> Result<Comment, ApiError> {
    sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = ? AND is_removed = 0")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::not_found("Comment not found"))
}

/// GET /comments/post/{postId}
pub async fn tree(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(post_id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let user_id = user.0.as_ref().map(|u| u.id);
    let is_admin = user.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    let post: Option<(i64,)> = sqlx::query_as(
        "SELECT forum_id FROM posts WHERE id = ? AND is_removed = 0",
    )
    .bind(post_id)
    .fetch_optional(&state.pool)
    .await?;
    if let Some((forum_id,)) = post {
        if let Some(forum) = crate::routes::forums::find_by_id(&state.pool, forum_id).await? {
            gate::ensure(&state.pool, user_id, is_admin, &forum, GateKind::Browse).await?;
        }
    }
    let comments: Vec<Comment> = sqlx::query_as::<_, Comment>(
        "SELECT * FROM comments WHERE post_id = ? AND is_removed = 0 ORDER BY created_at ASC",
    )
    .bind(post_id)
    .fetch_all(&state.pool)
    .await?;

    let mut roots: Vec<Comment> = Vec::new();
    let mut by_parent: HashMap<i64, Vec<Comment>> = HashMap::new();
    for c in comments {
        match c.parent_id {
            None => roots.push(c),
            Some(pid) => by_parent.entry(pid).or_default().push(c),
        }
    }
    let authors = load_authors(&state.pool, &roots).await?;
    let mut content = Vec::with_capacity(roots.len());
    for root in &roots {
        content.push(build_tree(root, &by_parent, &authors));
    }
    Ok(j::ok(content))
}

/// POST /comments
pub async fn create(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<CommentCreateRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let post_id = req
        .post_id
        .ok_or_else(|| ApiError::bad_request("postId must not be empty"))?;
    if req.content.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Err(ApiError::bad_request("content must not be empty"));
    }
    let content = req.content.as_deref().unwrap().to_string();

    let post: Option<(i64,)> =
        sqlx::query_as("SELECT forum_id FROM posts WHERE id = ? AND is_removed = 0")
            .bind(post_id)
            .fetch_optional(&state.pool)
            .await?;
    let forum_id = match post {
        Some((forum_id,)) => forum_id,
        None => return Err(ApiError::not_found("Post not found")),
    };
    if let Some(forum) = crate::routes::forums::find_by_id(&state.pool, forum_id).await? {
        gate::ensure(&state.pool, Some(me.id), me.is_admin(), &forum, GateKind::Operate).await?;
    }

    if let Some(parent_id) = req.parent_id {
        let parent: Option<(i64, i64)> = sqlx::query_as(
            "SELECT id, post_id FROM comments WHERE id = ? AND is_removed = 0",
        )
        .bind(parent_id)
        .fetch_optional(&state.pool)
        .await?;
        match parent {
            None => return Err(ApiError::bad_request("Parent comment not found")),
            Some((_, ppost)) if ppost != post_id => {
                return Err(ApiError::bad_request("Parent comment does not belong to this post"))
            }
            _ => {}
        }
    }

    // Comment insert and post comment count recount must be atomic (matches Java `CommentService.create`).
    let mut tx = state.pool.begin().await?;
    let result = sqlx::query(
        "INSERT INTO comments (post_id, author_id, parent_id, content, upvotes, downvotes, score, is_removed, created_at, updated_at) \
         VALUES (?, ?, ?, ?, 0, 0, 0, 0, NOW(), NOW())",
    )
    .bind(post_id)
    .bind(me.id)
    .bind(req.parent_id)
    .bind(&content)
    .execute(&mut *tx)
    .await?;
    let comment_id = result.last_insert_id() as i64;
    posts::recount_comments(&mut tx, post_id).await?;
    tx.commit().await?;

    let comment = require_comment(&state.pool, comment_id).await?;
    let (name, avatar) = author_fields(&state.pool, comment.author_id).await?;
    Ok(j::ok(build_single(&comment, name, avatar)))
}

/// DELETE /comments/{id}
pub async fn delete(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let comment = require_comment(&state.pool, id).await?;

    let mut authorized = me.id == comment.author_id;
    if !authorized {
        let post: Option<(i64,)> = sqlx::query_as("SELECT forum_id FROM posts WHERE id = ?")
            .bind(comment.post_id)
            .fetch_optional(&state.pool)
            .await?;
        if let Some((forum_id,)) = post {
            if let Some(forum) = crate::routes::forums::find_by_id(&state.pool, forum_id).await? {
                authorized = access::has_permission(
                    &state.pool,
                    &forum,
                    Some(me.id),
                    access::PERM_POSTS_COMMENTS,
                )
                .await?;
            }
        }
    }
    if !authorized {
        return Err(ApiError::forbidden(
            "Only the author or a moderator with the Posts & Comments permission can delete this comment",
        ));
    }

    // Soft delete, vote cleanup, and comment count recount must be atomic (matches Java `CommentService.delete`).
    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE comments SET is_removed = 1, updated_at = NOW() WHERE id = ?")
        .bind(comment.id)
        .execute(&mut *tx)
        .await?;
    vote::clear_votes(&mut tx, "COMMENT", comment.id).await?;

    let post: Option<(i64,)> = sqlx::query_as("SELECT id FROM posts WHERE id = ?")
        .bind(comment.post_id)
        .fetch_optional(&mut *tx)
        .await?;
    if post.is_some() {
        posts::recount_comments(&mut tx, comment.post_id).await?;
    }
    tx.commit().await?;
    Ok(j::ok_empty())
}

/// POST /comments/{id}/vote
pub async fn comment_vote(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<VoteRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let comment = require_comment(&state.pool, id).await?;
    let forum_id: Option<(i64,)> =
        sqlx::query_as("SELECT forum_id FROM posts WHERE id = ?")
            .bind(comment.post_id)
            .fetch_optional(&state.pool)
            .await?;
    if let Some((forum_id,)) = forum_id {
        if let Some(forum) = crate::routes::forums::find_by_id(&state.pool, forum_id).await? {
            gate::ensure(&state.pool, Some(me.id), me.is_admin(), &forum, GateKind::Operate)
                .await?;
        }
    }
    let value = req.value.unwrap_or(0);
    if !(-1..=1).contains(&value) {
        return Err(ApiError::bad_request("Vote value must be 1, -1, or 0"));
    }
    // Vote record changes and comment count write-back must be atomic (matches Java `CommentService.vote`).
    let mut tx = state.pool.begin().await?;
    let outcome = vote::apply(
        &mut tx,
        me.id,
        "COMMENT",
        comment.id,
        value,
        comment.upvotes,
        comment.downvotes,
    )
    .await?;
    sqlx::query("UPDATE comments SET upvotes = ?, downvotes = ?, score = ? WHERE id = ?")
        .bind(outcome.upvotes)
        .bind(outcome.downvotes)
        .bind(outcome.score)
        .bind(comment.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let comment = require_comment(&state.pool, id).await?;
    let (name, avatar) = author_fields(&state.pool, comment.author_id).await?;
    Ok(j::ok(build_single(&comment, name, avatar)))
}

/// Reused by the users module: GET /users/{id}/comments
pub async fn list_by_author(
    state: &AppState,
    author_id: i64,
    user_id: Option<i64>,
    is_admin: bool,
    page: i32,
    size: i32,
) -> ApiResult<Json<Value>> {
    let allowed = gate::allowed_forum_ids(&state.pool, user_id, is_admin).await?;
    let comments: Vec<Comment> = sqlx::query_as::<_, Comment>(
        "SELECT * FROM comments WHERE author_id = ? AND is_removed = 0 ORDER BY created_at DESC",
    )
    .bind(author_id)
    .fetch_all(&state.pool)
    .await?;

    let mut visible: Vec<(Comment, Option<String>)> = Vec::new();
    for comment in comments {
        let post: Option<(String, i64)> =
            sqlx::query_as("SELECT title, forum_id FROM posts WHERE id = ?")
                .bind(comment.post_id)
                .fetch_optional(&state.pool)
                .await?;
        let (title, forum_id) = match post {
            Some(v) => v,
            None => continue,
        };
        if !allowed.contains(&forum_id) {
            continue;
        }
        visible.push((comment, Some(title)));
    }
    let total = visible.len() as i64;
    let (slice, _) = j::slice_page(visible, page, size);

    let mut content = Vec::with_capacity(slice.len());
    for (comment, post_title) in &slice {
        let (name, avatar) = author_fields(&state.pool, comment.author_id).await?;
        content.push(json!({
            "id": comment.id,
            "postId": comment.post_id,
            "authorId": comment.author_id,
            "authorName": name,
            "authorAvatar": avatar,
            "parentId": comment.parent_id,
            "content": comment.content,
            "upvotes": comment.upvotes,
            "downvotes": comment.downvotes,
            "score": comment.score,
            "createdAt": j::jts(comment.created_at),
            "children": Vec::<Value>::new(),
            "postTitle": post_title,
        }));
    }
    Ok(j::ok(j::PageResponse::of(content, page, size, total)))
}
