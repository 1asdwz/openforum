use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{Duration, Local, NaiveDateTime};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::access;
use crate::auth_gate;
use crate::error::{ApiError, ApiResult};
use crate::json as j;
use crate::models::SubForum;
use crate::sanitizer;
use crate::security::CurrentUser;
use crate::state::AppState;

const DEFAULT_THEME_COLOR: &str = "#FF4500";

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub page: i32,
    #[serde(default = "default_size")]
    pub size: i32,
    #[serde(default = "default_hot")]
    pub sort: String,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LimitQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_size() -> i32 {
    20
}
fn default_hot() -> String {
    "hot".to_string()
}
fn default_limit() -> i32 {
    10
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForumCreateRequest {
    pub slug: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub theme_color: Option<String>,
    pub template_type: Option<String>,
    pub template_config: Option<String>,
    pub rules: Option<String>,
    pub required_auth_types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForumUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub theme_color: Option<String>,
    pub template_type: Option<String>,
    pub template_config: Option<String>,
    pub rules: Option<String>,
    pub required_auth_types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CustomPageUpdateRequest {
    pub html: Option<String>,
}

fn auth_type_list(csv: Option<&str>) -> Vec<String> {
    match csv {
        Some(s) if !s.trim().is_empty() => s
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn join_auth_types(list: Option<&Vec<String>>) -> Option<String> {
    let list = list?;
    let mut out: Vec<String> = Vec::new();
    for item in list {
        let v = item.trim().to_uppercase();
        if !v.is_empty() && !out.contains(&v) {
            out.push(v);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out.join(","))
    }
}

fn parse_template_type(value: Option<&str>) -> String {
    match value {
        Some(v) if !v.trim().is_empty() => {
            let up = v.trim().to_uppercase();
            if up == "CUSTOM" {
                "CUSTOM".to_string()
            } else {
                "DEFAULT".to_string()
            }
        }
        _ => "DEFAULT".to_string(),
    }
}

fn slug_valid(slug: &str) -> bool {
    let len = slug.chars().count();
    if !(2..=80).contains(&len) {
        return false;
    }
    slug.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

async fn load_forum_posts(pool: &MySqlPool) -> Result<HashMap<i64, Vec<(i32, NaiveDateTime)>>, ApiError> {
    let rows: Vec<(i64, i32, NaiveDateTime)> =
        sqlx::query_as("SELECT forum_id, score, created_at FROM posts WHERE is_removed = 0")
            .fetch_all(pool)
            .await?;
    let mut map: HashMap<i64, Vec<(i32, NaiveDateTime)>> = HashMap::new();
    for (fid, score, created) in rows {
        map.entry(fid).or_default().push((score, created));
    }
    Ok(map)
}

fn plugin_score(
    forum: &SubForum,
    posts: Option<&Vec<(i32, NaiveDateTime)>>,
    hot_enabled: bool,
) -> f64 {
    let base = forum.member_count as f64 * 2.0 + forum.post_count as f64 * 3.0;
    if !hot_enabled {
        // 与 Java 一致：推荐插件全部关闭时，回退到基础分。
        return base;
    }
    let mut recent = 0i64;
    let mut vote_sum = 0i64;
    if let Some(list) = posts {
        let cutoff = Local::now().naive_local() - Duration::days(7);
        for (score, created) in list {
            if *created > cutoff {
                recent += 1;
            }
            vote_sum += *score as i64;
        }
    }
    let raw = base + recent as f64 * 5.0 + vote_sum as f64 * 0.1;
    if raw <= 0.0 {
        base
    } else {
        raw
    }
}

async fn build_summary(
    _state: &AppState,
    forum: &SubForum,
    member_ids: &std::collections::HashSet<i64>,
) -> ApiResult<Value> {
    Ok(json!({
        "id": forum.id,
        "slug": forum.slug,
        "name": forum.name,
        "description": forum.description,
        "icon": forum.icon,
        "themeColor": forum.theme_color,
        "requiredAuthTypes": auth_type_list(forum.required_auth_types.as_deref()),
        "memberCount": forum.member_count,
        "postCount": forum.post_count,
        "isMember": member_ids.contains(&forum.id),
        "createdAt": j::jts(forum.created_at),
    }))
}

async fn build_detail(
    state: &AppState,
    forum: &SubForum,
    user_id: Option<i64>,
) -> ApiResult<Value> {
    let is_member = access::is_member(&state.pool, forum.id, user_id).await?;
    let mut can_post = false;
    if let Some(uid) = user_id {
        let gate = auth_gate::evaluate(&state.pool, Some(uid), forum.required_auth_types.as_deref()).await?;
        can_post = gate.passed;
    }
    let creator_name: Option<(String,)> =
        sqlx::query_as("SELECT username FROM users WHERE id = ?")
            .bind(forum.creator_id)
            .fetch_optional(&state.pool)
            .await?;
    let template_type = if forum.template_type.trim().is_empty() {
        "DEFAULT".to_string()
    } else {
        forum.template_type.clone()
    };
    let custom_page_html = if forum.custom_page_status.as_deref() == Some("APPROVED") {
        forum.custom_page_html.clone()
    } else {
        None
    };
    Ok(json!({
        "id": forum.id,
        "slug": forum.slug,
        "name": forum.name,
        "description": forum.description,
        "icon": forum.icon,
        "themeColor": forum.theme_color,
        "rules": forum.rules,
        "requiredAuthTypes": auth_type_list(forum.required_auth_types.as_deref()),
        "templateType": template_type,
        "templateConfig": forum.template_config,
        "customPageStatus": forum.custom_page_status,
        "customPageHtml": custom_page_html,
        "memberCount": forum.member_count,
        "postCount": forum.post_count,
        "isMember": is_member,
        "canPost": can_post,
        "isPublic": forum.public,
        "creatorId": forum.creator_id,
        "creatorName": creator_name.map(|(n,)| n),
        "createdAt": j::jts(forum.created_at),
    }))
}

fn custom_page_dto(forum: &SubForum) -> Value {
    json!({
        "forumId": forum.id,
        "forumSlug": forum.slug,
        "forumName": forum.name,
        "html": forum.custom_page_html,
        "status": forum.custom_page_status,
        "reviewNote": forum.custom_page_review_note,
    })
}

async fn find_forum_by_id(pool: &MySqlPool, id: i64) -> Result<Option<SubForum>, ApiError> {
    Ok(sqlx::query_as::<_, SubForum>("SELECT * FROM sub_forums WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

async fn require_forum(pool: &MySqlPool, id: i64) -> Result<SubForum, ApiError> {
    find_forum_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::not_found("子论坛不存在"))
}

/// GET /forums
pub async fn list(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(qp): Query<ListQuery>,
) -> ApiResult<Json<Value>> {
    let user_id = user.0.as_ref().map(|u| u.id);
    let mut forums: Vec<SubForum> = match qp.q.as_deref() {
        Some(q) if !q.trim().is_empty() => {
            let like = format!("%{}%", q.trim());
            sqlx::query_as::<_, SubForum>(
                "SELECT * FROM sub_forums WHERE name LIKE ? OR slug LIKE ?",
            )
            .bind(&like)
            .bind(&like)
            .fetch_all(&state.pool)
            .await?
        }
        _ => sqlx::query_as::<_, SubForum>("SELECT * FROM sub_forums")
            .fetch_all(&state.pool)
            .await?,
    };

    if qp.sort.eq_ignore_ascii_case("new") {
        forums.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    } else {
        let hot = state.plugins.is_enabled_key("hot-score-recommendation");
        let post_map = load_forum_posts(&state.pool).await?;
        forums.sort_by(|a, b| {
            let sa = plugin_score(a, post_map.get(&a.id), hot);
            let sb = plugin_score(b, post_map.get(&b.id), hot);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let total = forums.len() as i64;
    let (slice, _) = j::slice_page(forums, qp.page, qp.size);
    let member_ids = access::member_forum_ids(&state.pool, user_id).await?;
    let mut content = Vec::with_capacity(slice.len());
    for forum in &slice {
        content.push(build_summary(&state, forum, &member_ids).await?);
    }
    Ok(j::ok(j::PageResponse::of(content, qp.page, qp.size, total)))
}

/// GET /forums/popular
pub async fn popular(
    State(state): State<AppState>,
    Query(qp): Query<LimitQuery>,
) -> ApiResult<Json<Value>> {
    let mut forums: Vec<SubForum> = sqlx::query_as::<_, SubForum>("SELECT * FROM sub_forums")
        .fetch_all(&state.pool)
        .await?;
    forums.sort_by(|a, b| {
        b.member_count
            .cmp(&a.member_count)
            .then(b.post_count.cmp(&a.post_count))
    });
    forums.truncate(qp.limit.max(0) as usize);
    let empty = std::collections::HashSet::new();
    let mut content = Vec::with_capacity(forums.len());
    for forum in &forums {
        content.push(build_summary(&state, forum, &empty).await?);
    }
    Ok(j::ok(content))
}

/// GET /forums/recommended
pub async fn recommended(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(qp): Query<LimitQuery>,
) -> ApiResult<Json<Value>> {
    let user_id = user.0.as_ref().map(|u| u.id);
    let mut forums: Vec<SubForum> = sqlx::query_as::<_, SubForum>("SELECT * FROM sub_forums")
        .fetch_all(&state.pool)
        .await?;
    let post_map = load_forum_posts(&state.pool).await?;
    let member_ids = access::member_forum_ids(&state.pool, user_id).await?;
    let hot = state.plugins.is_enabled_key("hot-score-recommendation");

    let score_of = |f: &SubForum| -> f64 {
        let mut s = plugin_score(f, post_map.get(&f.id), hot);
        if member_ids.contains(&f.id) {
            s += 1000.0;
        }
        s
    };
    forums.sort_by(|a, b| {
        score_of(b)
            .partial_cmp(&score_of(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    forums.truncate(qp.limit.max(0) as usize);

    let mut content = Vec::with_capacity(forums.len());
    for forum in &forums {
        content.push(build_summary(&state, forum, &member_ids).await?);
    }
    Ok(j::ok(content))
}

/// POST /forums
pub async fn create(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<ForumCreateRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let slug = req.slug.as_deref().map(|s| s.trim().to_lowercase()).unwrap_or_default();
    if slug.is_empty() {
        return Err(ApiError::bad_request("slug slug 不能为空"));
    }
    if req.name.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Err(ApiError::bad_request("name 名称不能为空"));
    }
    if !slug_valid(&slug) {
        return Err(ApiError::bad_request(
            "slug 只能包含小写字母、数字、-、_（2-80 位）",
        ));
    }
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM sub_forums WHERE slug = ? LIMIT 1")
            .bind(&slug)
            .fetch_optional(&state.pool)
            .await?;
    if exists.is_some() {
        return Err(ApiError::bad_request("slug 已存在"));
    }

    let name = req.name.as_deref().unwrap().trim().to_string();
    let theme_color = req
        .theme_color
        .clone()
        .unwrap_or_else(|| DEFAULT_THEME_COLOR.to_string());
    let template_type = parse_template_type(req.template_type.as_deref());
    let required = join_auth_types(req.required_auth_types.as_ref());

    let result = sqlx::query(
        "INSERT INTO sub_forums (slug, name, description, icon, theme_color, rules, required_auth_types, \
         template_type, template_config, custom_page_html, custom_page_status, custom_page_review_note, \
         creator_id, member_count, post_count, is_public, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, ?, 0, 0, 1, NOW(), NOW())",
    )
    .bind(&slug)
    .bind(&name)
    .bind(req.description.as_deref())
    .bind(req.icon.as_deref())
    .bind(&theme_color)
    .bind(req.rules.as_deref())
    .bind(&required)
    .bind(&template_type)
    .bind(req.template_config.as_deref())
    .bind(me.id)
    .execute(&state.pool)
    .await?;
    let forum_id = result.last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO forum_members (forum_id, user_id, role, joined_at) VALUES (?, ?, 'OWNER', NOW())",
    )
    .bind(forum_id)
    .bind(me.id)
    .execute(&state.pool)
    .await?;
    sqlx::query("UPDATE sub_forums SET member_count = 1 WHERE id = ?")
        .bind(forum_id)
        .execute(&state.pool)
        .await?;

    let forum = require_forum(&state.pool, forum_id).await?;
    Ok(j::ok(build_detail(&state, &forum, Some(me.id)).await?))
}

/// GET /forums/{slug}
pub async fn detail(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(slug): Path<String>,
) -> ApiResult<Json<Value>> {
    let forum = sqlx::query_as::<_, SubForum>("SELECT * FROM sub_forums WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError::not_found("子论坛不存在"))?;
    let user_id = user.0.as_ref().map(|u| u.id);
    Ok(j::ok(build_detail(&state, &forum, user_id).await?))
}

/// PUT /forums/{id}
pub async fn update(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<ForumUpdateRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let mut forum = require_forum(&state.pool, id).await?;
    if !access::can_manage(&state.pool, &forum, Some(me.id)).await? {
        return Err(ApiError::forbidden("需要坛主或管理员权限"));
    }
    if let Some(v) = req.name {
        forum.name = v;
    }
    if let Some(v) = req.description {
        forum.description = Some(v);
    }
    if let Some(v) = req.icon {
        forum.icon = Some(v);
    }
    if let Some(v) = req.theme_color {
        forum.theme_color = Some(v);
    }
    if let Some(v) = req.template_type {
        forum.template_type = parse_template_type(Some(&v));
    }
    if let Some(v) = req.template_config {
        forum.template_config = Some(v);
    }
    if let Some(v) = req.rules {
        forum.rules = Some(v);
    }
    if let Some(list) = req.required_auth_types {
        forum.required_auth_types = join_auth_types(Some(&list));
    }

    sqlx::query(
        "UPDATE sub_forums SET name=?, description=?, icon=?, theme_color=?, template_type=?, \
         template_config=?, rules=?, required_auth_types=?, updated_at=NOW() WHERE id=?",
    )
    .bind(&forum.name)
    .bind(&forum.description)
    .bind(&forum.icon)
    .bind(&forum.theme_color)
    .bind(&forum.template_type)
    .bind(&forum.template_config)
    .bind(&forum.rules)
    .bind(&forum.required_auth_types)
    .bind(forum.id)
    .execute(&state.pool)
    .await?;

    let forum = require_forum(&state.pool, id).await?;
    Ok(j::ok(build_detail(&state, &forum, Some(me.id)).await?))
}

/// PUT /forums/{id}/custom-page
pub async fn save_custom_page(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<CustomPageUpdateRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    if !access::can_manage(&state.pool, &forum, Some(me.id)).await? {
        return Err(ApiError::forbidden("需要坛主或管理员权限"));
    }
    let html = req.html.unwrap_or_default();
    if html.trim().is_empty() {
        return Err(ApiError::bad_request("html 不能为空"));
    }
    let clean = sanitizer::sanitize(&html);
    sqlx::query(
        "UPDATE sub_forums SET custom_page_html=?, custom_page_status='DRAFT', custom_page_review_note=NULL, updated_at=NOW() WHERE id=?",
    )
    .bind(&clean)
    .bind(forum.id)
    .execute(&state.pool)
    .await?;
    let forum = require_forum(&state.pool, id).await?;
    Ok(j::ok(custom_page_dto(&forum)))
}

/// POST /forums/{id}/custom-page/submit
pub async fn submit_custom_page(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    if !access::can_manage(&state.pool, &forum, Some(me.id)).await? {
        return Err(ApiError::forbidden("需要坛主或管理员权限"));
    }
    if forum.custom_page_html.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Err(ApiError::bad_request("请先保存自定义页面 HTML"));
    }
    sqlx::query(
        "UPDATE sub_forums SET custom_page_status='PENDING', custom_page_review_note=NULL, updated_at=NOW() WHERE id=?",
    )
    .bind(forum.id)
    .execute(&state.pool)
    .await?;
    let forum = require_forum(&state.pool, id).await?;
    Ok(j::ok(custom_page_dto(&forum)))
}

/// GET /forums/{id}/custom-page
pub async fn get_custom_page(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    if !access::can_manage(&state.pool, &forum, Some(me.id)).await? {
        return Err(ApiError::forbidden("需要坛主或管理员权限"));
    }
    Ok(j::ok(custom_page_dto(&forum)))
}

/// POST /forums/{id}/join
pub async fn join(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    // 入会（成员插入 + member_count 重算）需原子完成（对齐 Java `SubForumService.join` 的 @Transactional）。
    let mut tx = state.pool.begin().await?;
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM forum_members WHERE forum_id=? AND user_id=? LIMIT 1")
            .bind(forum.id)
            .bind(me.id)
            .fetch_optional(&mut *tx)
            .await?;
    if exists.is_none() {
        sqlx::query(
            "INSERT INTO forum_members (forum_id, user_id, role, joined_at) VALUES (?, ?, 'MEMBER', NOW())",
        )
        .bind(forum.id)
        .bind(me.id)
        .execute(&mut *tx)
        .await?;
    }
    recount_members(&mut tx, forum.id).await?;
    tx.commit().await?;
    Ok(j::ok_empty())
}

/// POST /forums/{id}/leave
pub async fn leave(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    if me.id == forum.creator_id {
        return Err(ApiError::bad_request("坛主不能退出自己的子论坛"));
    }
    // 退会（成员删除 + member_count 重算）需原子完成（对齐 Java `SubForumService.leave` 的 @Transactional）。
    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM forum_members WHERE forum_id=? AND user_id=?")
        .bind(forum.id)
        .bind(me.id)
        .execute(&mut *tx)
        .await?;
    recount_members(&mut tx, forum.id).await?;
    tx.commit().await?;
    Ok(j::ok_empty())
}

async fn recount_members(conn: &mut sqlx::MySqlConnection, forum_id: i64) -> Result<(), ApiError> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM forum_members WHERE forum_id = ?")
        .bind(forum_id)
        .fetch_one(&mut *conn)
        .await?;
    sqlx::query("UPDATE sub_forums SET member_count = ? WHERE id = ?")
        .bind(count as i32)
        .bind(forum_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// GET /forums/{id}/members
pub async fn members(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let forum = require_forum(&state.pool, id).await?;
    let rows: Vec<(i64, i64, String, NaiveDateTime)> = sqlx::query_as(
        "SELECT id, user_id, role, joined_at FROM forum_members WHERE forum_id = ? ORDER BY joined_at ASC",
    )
    .bind(forum.id)
    .fetch_all(&state.pool)
    .await?;
    let mut content = Vec::with_capacity(rows.len());
    for (_id, user_id, role, joined_at) in rows {
        let user: Option<(String, Option<String>)> =
            sqlx::query_as("SELECT username, avatar FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(&state.pool)
                .await?;
        let (username, avatar) = match user {
            Some((u, a)) => (Some(u), a),
            None => (None, None),
        };
        content.push(json!({
            "userId": user_id,
            "username": username,
            "avatar": avatar,
            "role": role,
            "joinedAt": j::jts(joined_at),
        }));
    }
    Ok(j::ok(content))
}

// -------- 管理端自定义页 --------

#[derive(Debug, Deserialize)]
pub struct CustomPageListQuery {
    #[serde(default = "default_pending")]
    pub status: String,
    #[serde(default)]
    pub page: i32,
    #[serde(default = "default_size")]
    pub size: i32,
}

fn default_pending() -> String {
    "PENDING".to_string()
}

fn parse_status(value: &str) -> Result<String, ApiError> {
    if value.trim().is_empty() {
        return Ok("PENDING".to_string());
    }
    let up = value.trim().to_uppercase();
    match up.as_str() {
        "PENDING" | "APPROVED" | "REJECTED" | "DRAFT" => Ok(up),
        _ => Err(ApiError::bad_request(format!("非法的状态: {value}"))),
    }
}

/// GET /admin/custom-pages
pub async fn admin_list_custom_pages(
    State(state): State<AppState>,
    Query(qp): Query<CustomPageListQuery>,
) -> ApiResult<Json<Value>> {
    let status = parse_status(&qp.status)?;
    let forums: Vec<SubForum> = sqlx::query_as::<_, SubForum>(
        "SELECT * FROM sub_forums WHERE custom_page_status = ? ORDER BY updated_at DESC",
    )
    .bind(&status)
    .fetch_all(&state.pool)
    .await?;
    let total = forums.len() as i64;
    let (slice, _) = j::slice_page(forums, qp.page, qp.size);
    let content: Vec<Value> = slice.iter().map(custom_page_dto).collect();
    Ok(j::ok(j::PageResponse::of(content, qp.page, qp.size, total)))
}

/// POST /admin/custom-pages/{id}/approve
pub async fn admin_approve_custom_page(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let forum = require_forum(&state.pool, id).await?;
    sqlx::query(
        "UPDATE sub_forums SET custom_page_status='APPROVED', custom_page_review_note=NULL, template_type='CUSTOM', updated_at=NOW() WHERE id=?",
    )
    .bind(forum.id)
    .execute(&state.pool)
    .await?;
    let forum = require_forum(&state.pool, id).await?;
    Ok(j::ok(custom_page_dto(&forum)))
}

/// POST /admin/custom-pages/{id}/reject
pub async fn admin_reject_custom_page(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    body: Option<Json<serde_json::Value>>,
) -> ApiResult<Json<Value>> {
    let forum = require_forum(&state.pool, id).await?;
    let reason = body
        .as_ref()
        .and_then(|Json(v)| v.get("reason").and_then(|r| r.as_str()).map(|s| s.to_string()))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "管理员驳回".to_string());
    sqlx::query(
        "UPDATE sub_forums SET custom_page_status='REJECTED', custom_page_review_note=?, updated_at=NOW() WHERE id=?",
    )
    .bind(&reason)
    .bind(forum.id)
    .execute(&state.pool)
    .await?;
    let forum = require_forum(&state.pool, id).await?;
    Ok(j::ok(custom_page_dto(&forum)))
}

/// 供 posts 模块复用。
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<SubForum>, ApiError> {
    find_forum_by_id(pool, id).await
}
