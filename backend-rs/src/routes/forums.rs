use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{Duration, Local, NaiveDateTime};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::access;
use crate::error::{ApiError, ApiResult};
use crate::gate::{self, GateKind};
use crate::json as j;
use crate::models::SubForum;
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
    pub browse_rule: Option<Value>,
    pub operate_rule: Option<Value>,
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
    pub browse_rule: Option<Value>,
    pub operate_rule: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddModeratorRequest {
    pub username: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateModeratorRequest {
    #[serde(default)]
    pub permissions: Vec<String>,
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

fn rule_storage(rule: Option<&gate::GateRule>) -> Option<String> {
    rule.filter(|r| !r.is_empty()).map(|r| r.to_json_string())
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
        // Same as Java: fall back to the base score when all recommendation plugins are disabled.
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
    is_admin: bool,
) -> ApiResult<Value> {
    let is_member = access::is_member(&state.pool, forum.id, user_id).await?;
    let mut can_post = false;
    if user_id.is_some() {
        let gate_result =
            gate::check(&state.pool, user_id, is_admin, forum, GateKind::Operate).await?;
        can_post = gate_result.passed;
    }
    let rules = gate::load_rules(&state.pool, forum.id).await?;
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
    let my_permissions = access::permissions_of(&state.pool, forum, user_id).await?;
    let can_manage = !my_permissions.is_empty();
    let is_creator = user_id == Some(forum.creator_id);
    Ok(json!({
        "id": forum.id,
        "slug": forum.slug,
        "name": forum.name,
        "description": forum.description,
        "icon": forum.icon,
        "themeColor": forum.theme_color,
        "rules": forum.rules,
        "requiredAuthTypes": auth_type_list(forum.required_auth_types.as_deref()),
        "legacyRequiredAuthTypes": auth_type_list(forum.required_auth_types.as_deref()),
        "browseRule": rules.browse_json(),
        "operateRule": rules.operate_json(),
        "templateType": template_type,
        "templateConfig": forum.template_config,
        "memberCount": forum.member_count,
        "postCount": forum.post_count,
        "isMember": is_member,
        "canPost": can_post,
        "canOperate": can_post,
        "canManage": can_manage,
        "myPermissions": my_permissions,
        "isCreator": is_creator,
        "isPublic": forum.public,
        "creatorId": forum.creator_id,
        "creatorName": creator_name.map(|(n,)| n),
        "createdAt": j::jts(forum.created_at),
    }))
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
        .ok_or_else(|| ApiError::not_found("Forum not found"))
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
        return Err(ApiError::bad_request("slug must not be empty"));
    }
    if req.name.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Err(ApiError::bad_request("name must not be empty"));
    }
    if !slug_valid(&slug) {
        return Err(ApiError::bad_request(
            "slug may only contain lowercase letters, digits, -, and _ (2-80 characters)",
        ));
    }
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM sub_forums WHERE slug = ? LIMIT 1")
            .bind(&slug)
            .fetch_optional(&state.pool)
            .await?;
    if exists.is_some() {
        return Err(ApiError::bad_request("slug already exists"));
    }

    let name = req.name.as_deref().unwrap().trim().to_string();
    let theme_color = req
        .theme_color
        .clone()
        .unwrap_or_else(|| DEFAULT_THEME_COLOR.to_string());
    let template_type = parse_template_type(req.template_type.as_deref());
    let required = join_auth_types(req.required_auth_types.as_ref());
    let browse_rule = gate::normalize_rule(req.browse_rule.as_ref())?;
    let operate_rule = gate::normalize_rule(req.operate_rule.as_ref())?;
    let browse_json = rule_storage(browse_rule.as_ref());
    let operate_json = rule_storage(operate_rule.as_ref());

    let result = sqlx::query(
        "INSERT INTO sub_forums (slug, name, description, icon, theme_color, rules, required_auth_types, \
         template_type, template_config, creator_id, member_count, post_count, is_public, browse_rule, \
         operate_rule, gate_version, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 1, ?, ?, 1, NOW(), NOW())",
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
    .bind(&browse_json)
    .bind(&operate_json)
    .execute(&state.pool)
    .await?;
    let forum_id = result.last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO forum_members (forum_id, user_id, role, permissions, promoted_at, joined_at) \
         VALUES (?, ?, 'OWNER', 'EVERYTHING', NOW(), NOW())",
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
    Ok(j::ok(build_detail(&state, &forum, Some(me.id), me.is_admin()).await?))
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
        .ok_or_else(|| ApiError::not_found("Forum not found"))?;
    let user_id = user.0.as_ref().map(|u| u.id);
    let is_admin = user.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    gate::ensure(&state.pool, user_id, is_admin, &forum, GateKind::Browse).await?;
    Ok(j::ok(build_detail(&state, &forum, user_id, is_admin).await?))
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
    access::require_permission(
        &state.pool,
        &forum,
        Some(me.id),
        access::PERM_CONFIG,
    )
    .await?;
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

    if let Some(raw) = req.browse_rule {
        let rule = gate::normalize_rule(Some(&raw))?;
        sqlx::query("UPDATE sub_forums SET browse_rule=?, gate_version=1 WHERE id=?")
            .bind(rule_storage(rule.as_ref()))
            .bind(forum.id)
            .execute(&state.pool)
            .await?;
    }
    if let Some(raw) = req.operate_rule {
        let rule = gate::normalize_rule(Some(&raw))?;
        sqlx::query("UPDATE sub_forums SET operate_rule=?, gate_version=1 WHERE id=?")
            .bind(rule_storage(rule.as_ref()))
            .bind(forum.id)
            .execute(&state.pool)
            .await?;
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
    Ok(j::ok(build_detail(&state, &forum, Some(me.id), me.is_admin()).await?))
}

/// POST /forums/{id}/join
pub async fn join(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    // Joining (member insert + member_count recount) must be atomic (matches @Transactional in Java `SubForumService.join`).
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
        return Err(ApiError::bad_request("The forum owner cannot leave their own forum"));
    }
    // Leaving (member delete + member_count recount) must be atomic (matches @Transactional in Java `SubForumService.leave`).
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
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let forum = require_forum(&state.pool, id).await?;
    let user_id = user.0.as_ref().map(|u| u.id);
    let is_admin = user.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    gate::ensure(&state.pool, user_id, is_admin, &forum, GateKind::Browse).await?;
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

async fn build_moderator(
    pool: &MySqlPool,
    forum: &SubForum,
    user_id: i64,
    role: &str,
    permissions_raw: Option<&str>,
    joined_at: NaiveDateTime,
    promoted_at: Option<NaiveDateTime>,
) -> ApiResult<Value> {
    let user: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT username, avatar FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    let (username, avatar) = match user {
        Some((name, avatar)) => (name, avatar),
        None => (String::new(), None),
    };
    let is_creator = user_id == forum.creator_id;
    let permissions = access::effective_permissions(role, permissions_raw, is_creator);
    Ok(json!({
        "userId": user_id,
        "username": username,
        "avatar": avatar,
        "role": role,
        "isCreator": is_creator,
        "permissions": permissions,
        "joinedAt": j::jts(joined_at),
        "promotedAt": promoted_at.map(j::jts),
    }))
}

type ModeratorRow = (
    i64,
    String,
    Option<String>,
    String,
    Option<String>,
    NaiveDateTime,
    Option<NaiveDateTime>,
);

/// GET /forums/{id}/moderators
pub async fn list_moderators(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    if !access::can_manage(&state.pool, &forum, Some(me.id)).await? {
        return Err(ApiError::forbidden("Moderator permission required"));
    }

    let rows: Vec<ModeratorRow> = sqlx::query_as(
        "SELECT fm.user_id, u.username, u.avatar, fm.role, fm.permissions, fm.joined_at, fm.promoted_at \
         FROM forum_members fm JOIN users u ON u.id = fm.user_id \
         WHERE fm.forum_id = ? AND fm.role IN ('OWNER', 'MODERATOR') \
         ORDER BY FIELD(fm.role, 'OWNER', 'MODERATOR'), fm.joined_at ASC",
    )
    .bind(forum.id)
    .fetch_all(&state.pool)
    .await?;

    let mut content: Vec<Value> = Vec::with_capacity(rows.len() + 1);
    let mut has_creator = false;
    for (user_id, username, avatar, role, raw_permissions, joined_at, promoted_at) in rows {
        let is_creator = user_id == forum.creator_id;
        if is_creator {
            has_creator = true;
        }
        let permissions = access::effective_permissions(&role, raw_permissions.as_deref(), is_creator);
        content.push(json!({
            "userId": user_id,
            "username": username,
            "avatar": avatar,
            "role": role,
            "isCreator": is_creator,
            "permissions": permissions,
            "joinedAt": j::jts(joined_at),
            "promotedAt": promoted_at.map(j::jts),
        }));
    }

    // Legacy data compatibility: still show the creator when their member row is missing.
    if !has_creator {
        content.insert(
            0,
            build_moderator(
                &state.pool,
                &forum,
                forum.creator_id,
                "OWNER",
                Some(access::PERM_EVERYTHING),
                forum.created_at,
                None,
            )
            .await?,
        );
    }

    Ok(j::ok(content))
}

/// POST /forums/{id}/moderators
pub async fn add_moderator(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<AddModeratorRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    access::require_permission(
        &state.pool,
        &forum,
        Some(me.id),
        access::PERM_EVERYTHING,
    )
    .await?;

    let username = req
        .username
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| ApiError::bad_request("Please enter a username"))?;
    let target: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM users WHERE username = ? LIMIT 1")
            .bind(username)
            .fetch_optional(&state.pool)
            .await?;
    let target_id = target
        .ok_or_else(|| ApiError::not_found("User not found"))?
        .0;
    if target_id == forum.creator_id {
        return Err(ApiError::bad_request("The creator already has all permissions"));
    }

    let permissions = access::normalize_permissions(req.permissions);
    if permissions.is_empty() {
        return Err(ApiError::bad_request("Please select at least one permission"));
    }
    let stored = access::serialize_permissions(&permissions);

    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT role FROM forum_members WHERE forum_id = ? AND user_id = ? LIMIT 1",
    )
    .bind(forum.id)
    .bind(target_id)
    .fetch_optional(&state.pool)
    .await?;
    if matches!(
        existing.as_ref().map(|(role,)| role.as_str()),
        Some("OWNER")
    ) {
        return Err(ApiError::bad_request("The creator's permissions cannot be changed"));
    }

    let mut tx = state.pool.begin().await?;
    if existing.is_some() {
        sqlx::query(
            "UPDATE forum_members SET role = 'MODERATOR', permissions = ?, \
             promoted_at = NOW(), promoted_by = ? WHERE forum_id = ? AND user_id = ?",
        )
        .bind(&stored)
        .bind(me.id)
        .bind(forum.id)
        .bind(target_id)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO forum_members (forum_id, user_id, role, permissions, promoted_at, promoted_by, joined_at) \
             VALUES (?, ?, 'MODERATOR', ?, NOW(), ?, NOW())",
        )
        .bind(forum.id)
        .bind(target_id)
        .bind(&stored)
        .bind(me.id)
        .execute(&mut *tx)
        .await?;
        recount_members(&mut tx, forum.id).await?;
    }
    tx.commit().await?;

    let row: (String, Option<String>, NaiveDateTime, Option<NaiveDateTime>) = sqlx::query_as(
        "SELECT role, permissions, joined_at, promoted_at \
         FROM forum_members WHERE forum_id = ? AND user_id = ?",
    )
    .bind(forum.id)
    .bind(target_id)
    .fetch_one(&state.pool)
    .await?;
    Ok(j::ok(
        build_moderator(
            &state.pool,
            &forum,
            target_id,
            &row.0,
            row.1.as_deref(),
            row.2,
            row.3,
        )
        .await?,
    ))
}

/// PUT /forums/{id}/moderators/{userId}
pub async fn update_moderator(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((id, target_id)): Path<(i64, i64)>,
    Json(req): Json<UpdateModeratorRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    access::require_permission(
        &state.pool,
        &forum,
        Some(me.id),
        access::PERM_EVERYTHING,
    )
    .await?;
    if target_id == forum.creator_id {
        return Err(ApiError::bad_request("The creator's permissions cannot be changed"));
    }

    let permissions = access::normalize_permissions(req.permissions);
    if permissions.is_empty() {
        return Err(ApiError::bad_request("Please select at least one permission"));
    }
    let stored = access::serialize_permissions(&permissions);

    let result = sqlx::query(
        "UPDATE forum_members SET permissions = ? \
         WHERE forum_id = ? AND user_id = ? AND role = 'MODERATOR'",
    )
    .bind(&stored)
    .bind(forum.id)
    .bind(target_id)
    .execute(&state.pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("This user is not a moderator"));
    }

    let row: (String, Option<String>, NaiveDateTime, Option<NaiveDateTime>) = sqlx::query_as(
        "SELECT role, permissions, joined_at, promoted_at \
         FROM forum_members WHERE forum_id = ? AND user_id = ?",
    )
    .bind(forum.id)
    .bind(target_id)
    .fetch_one(&state.pool)
    .await?;
    Ok(j::ok(
        build_moderator(
            &state.pool,
            &forum,
            target_id,
            &row.0,
            row.1.as_deref(),
            row.2,
            row.3,
        )
        .await?,
    ))
}

/// DELETE /forums/{id}/moderators/{userId}
pub async fn remove_moderator(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((id, target_id)): Path<(i64, i64)>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let forum = require_forum(&state.pool, id).await?;
    access::require_permission(
        &state.pool,
        &forum,
        Some(me.id),
        access::PERM_EVERYTHING,
    )
    .await?;
    if target_id == forum.creator_id {
        return Err(ApiError::bad_request("The creator cannot be removed"));
    }

    let result = sqlx::query(
        "UPDATE forum_members SET role = 'MEMBER', permissions = '', \
         promoted_at = NULL, promoted_by = NULL \
         WHERE forum_id = ? AND user_id = ? AND role = 'MODERATOR'",
    )
    .bind(forum.id)
    .bind(target_id)
    .execute(&state.pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("This user is not a moderator"));
    }
    Ok(j::ok_empty())
}

/// Reused by the posts module.
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<SubForum>, ApiError> {
    find_forum_by_id(pool, id).await
}
