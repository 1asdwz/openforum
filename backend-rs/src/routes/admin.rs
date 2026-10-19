use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::enums::AuthLevel;
use crate::error::{ApiError, ApiResult};
use crate::json as j;
use crate::models::UserAuthentication;
use crate::repo;
use crate::routes::authentications::auth_dto;
use crate::routes::{forum_homepages, homepages, users};
use crate::security::CurrentUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AuthListQuery {
    #[serde(default = "default_pending")]
    pub status: String,
    pub level: Option<String>,
    #[serde(default)]
    pub page: i32,
    #[serde(default = "default_size")]
    pub size: i32,
}

fn default_pending() -> String {
    "PENDING".to_string()
}
fn default_size() -> i32 {
    20
}

fn parse_status(raw: &str) -> Result<String, ApiError> {
    if raw.trim().is_empty() {
        return Ok("PENDING".to_string());
    }
    let up = raw.trim().to_uppercase();
    match up.as_str() {
        "PENDING" | "APPROVED" | "REJECTED" => Ok(up),
        _ => Err(ApiError::bad_request(format!("Invalid status: {raw}"))),
    }
}

/// GET /admin/authentications
pub async fn list_authentications(
    State(state): State<AppState>,
    admin: CurrentUser,
    Query(qp): Query<AuthListQuery>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    let status = parse_status(&qp.status)?;
    let level = AuthLevel::parse_or_null(qp.level.as_deref());

    let mut list: Vec<UserAuthentication> = match level {
        Some(l) => {
            sqlx::query_as::<_, UserAuthentication>(
                "SELECT * FROM user_authentications WHERE status = ? AND level = ? ORDER BY created_at ASC",
            )
            .bind(&status)
            .bind(l.name())
            .fetch_all(&state.pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, UserAuthentication>(
                "SELECT * FROM user_authentications WHERE status = ? ORDER BY created_at ASC",
            )
            .bind(&status)
            .fetch_all(&state.pool)
            .await?
        }
    };
    let total = list.len() as i64;
    let (slice, _) = j::slice_page(std::mem::take(&mut list), qp.page, qp.size);
    let mut content = Vec::with_capacity(slice.len());
    for auth in &slice {
        content.push(auth_dto(&state, auth).await?);
    }
    Ok(j::ok(j::PageResponse::of(content, qp.page, qp.size, total)))
}

/// POST /admin/authentications/{id}/approve
pub async fn approve_authentication(
    State(state): State<AppState>,
    admin: CurrentUser,
    Path(id): Path<i64>,
    body: Option<Json<Value>>,
) -> ApiResult<Json<Value>> {
    let me = admin.require_admin()?;
    let mut auth = repo::find_auth_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::not_found("Authentication record not found"))?;
    let reason = body
        .as_ref()
        .and_then(|Json(v)| v.get("reason").and_then(|r| r.as_str()).map(|s| s.to_string()))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Approved by admin".to_string());

    auth.status = "APPROVED".to_string();
    auth.review_method = Some("MANUAL".to_string());
    auth.reviewed_by = Some(me.id);
    auth.reviewed_at = Some(chrono::Local::now().naive_local());
    auth.reason = Some(reason);
    auth.rejection_reason = None;

    // Manual approval must also write the authentication result into user info fields for forum gate checks.
    let data = j::json_value(auth.auth_data.as_deref());
    let data_map = data.as_object().cloned().unwrap_or_default();
    let level_name = auth.level.clone().unwrap_or_else(|| "LOW".to_string());
    let mut tx = state.pool.begin().await?;
    repo::save_auth(&mut *tx, &mut auth).await?;
    crate::info::grant_user_info(
        &mut *tx,
        auth.user_id,
        &auth.auth_type,
        auth.id,
        &level_name,
        &data_map,
    )
    .await?;
    tx.commit().await?;
    Ok(j::ok(auth_dto(&state, &auth).await?))
}

/// POST /admin/authentications/{id}/reject
pub async fn reject_authentication(
    State(state): State<AppState>,
    admin: CurrentUser,
    Path(id): Path<i64>,
    body: Option<Json<Value>>,
) -> ApiResult<Json<Value>> {
    let me = admin.require_admin()?;
    let mut auth = repo::find_auth_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::not_found("Authentication record not found"))?;
    let reason = body
        .as_ref()
        .and_then(|Json(v)| v.get("reason").and_then(|r| r.as_str()).map(|s| s.to_string()))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Rejected by admin".to_string());

    auth.status = "REJECTED".to_string();
    auth.review_method = Some("MANUAL".to_string());
    auth.reviewed_by = Some(me.id);
    auth.reviewed_at = Some(chrono::Local::now().naive_local());
    auth.rejection_reason = Some(reason);
    repo::save_auth(&state.pool, &mut auth).await?;
    Ok(j::ok(auth_dto(&state, &auth).await?))
}

/// GET /admin/users
pub async fn list_users(
    State(state): State<AppState>,
    admin: CurrentUser,
    Query(qp): Query<users::PageQuery>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    users::list_users_page(&state, qp.page, qp.size).await
}

#[derive(Debug, Deserialize)]
pub struct HomepageListQuery {
    #[serde(default = "default_all")]
    pub status: String,
    #[serde(default)]
    pub page: i32,
    #[serde(default = "default_size")]
    pub size: i32,
}

fn default_all() -> String {
    "ALL".to_string()
}

#[derive(Debug, sqlx::FromRow)]
struct AdminHomepageRow {
    user_id: i64,
    username: String,
    mode: String,
    status: String,
    size_bytes: i32,
    block_reason: Option<String>,
    updated_at: chrono::NaiveDateTime,
    report_count: i64,
}

/// GET /admin/user-homepages
pub async fn list_user_homepages(
    State(state): State<AppState>,
    admin: CurrentUser,
    Query(qp): Query<HomepageListQuery>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    let status = qp.status.trim().to_ascii_uppercase();
    if !matches!(status.as_str(), "ALL" | "ACTIVE" | "BLOCKED") {
        return Err(ApiError::bad_request(format!("Invalid status: {}", qp.status)));
    }

    let mut rows: Vec<AdminHomepageRow> = sqlx::query_as(
        "SELECT h.user_id, u.username, h.mode, h.status, h.size_bytes, h.block_reason, h.updated_at, \
         (SELECT COUNT(*) FROM user_homepage_reports r WHERE r.user_id = h.user_id) AS report_count \
         FROM user_homepages h JOIN users u ON u.id = h.user_id \
         WHERE h.mode = 'CUSTOM' ORDER BY h.updated_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    if status != "ALL" {
        rows.retain(|r| r.status == status);
    }

    let total = rows.len() as i64;
    let (slice, _) = j::slice_page(rows, qp.page, qp.size);
    let content: Vec<Value> = slice
        .iter()
        .map(|r| {
            json!({
                "userId": r.user_id,
                "username": r.username,
                "mode": r.mode,
                "status": r.status,
                "sizeBytes": r.size_bytes,
                "blockReason": r.block_reason,
                "reportCount": r.report_count,
                "updatedAt": j::jts(r.updated_at),
            })
        })
        .collect();
    Ok(j::ok(j::PageResponse::of(content, qp.page, qp.size, total)))
}

/// POST /admin/user-homepages/{userId}/block
pub async fn block_user_homepage(
    State(state): State<AppState>,
    admin: CurrentUser,
    Path(user_id): Path<i64>,
    body: Option<Json<Value>>,
) -> ApiResult<Json<Value>> {
    let me = admin.require_admin()?;
    let reason = body
        .as_ref()
        .and_then(|Json(v)| v.get("reason").and_then(|r| r.as_str()))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::bad_request("Please provide a block reason"))?;
    let chars = reason.chars().count();
    if !(2..=200).contains(&chars) {
        return Err(ApiError::bad_request("Block reason must be 2-200 characters"));
    }

    homepages::find_homepage(&state.pool, user_id)
        .await?
        .filter(|r| r.mode == "CUSTOM")
        .ok_or_else(|| ApiError::not_found("This user has no custom homepage"))?;

    sqlx::query(
        "UPDATE user_homepages SET status = 'BLOCKED', block_reason = ?, blocked_by = ?, \
         blocked_at = ? WHERE user_id = ?",
    )
    .bind(&reason)
    .bind(me.id)
    .bind(chrono::Local::now().naive_local())
    .bind(user_id)
    .execute(&state.pool)
    .await?;
    Ok(j::ok(json!({ "ok": true })))
}

#[derive(Debug, sqlx::FromRow)]
struct AdminForumHomepageRow {
    forum_id: i64,
    forum_name: String,
    forum_slug: String,
    creator_name: String,
    status: String,
    size_bytes: i32,
    block_reason: Option<String>,
    updated_at: chrono::NaiveDateTime,
    report_count: i64,
}

/// GET /admin/forum-homepages
pub async fn list_forum_homepages(
    State(state): State<AppState>,
    admin: CurrentUser,
    Query(qp): Query<HomepageListQuery>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    let status = qp.status.trim().to_ascii_uppercase();
    if !matches!(status.as_str(), "ALL" | "ACTIVE" | "BLOCKED") {
        return Err(ApiError::bad_request(format!("Invalid status: {}", qp.status)));
    }

    let mut rows: Vec<AdminForumHomepageRow> = sqlx::query_as(
        "SELECT h.forum_id, f.name AS forum_name, f.slug AS forum_slug, \
         u.username AS creator_name, h.status, h.size_bytes, h.block_reason, h.updated_at, \
         (SELECT COUNT(*) FROM forum_homepage_reports r WHERE r.forum_id = h.forum_id) AS report_count \
         FROM forum_homepages h \
         JOIN sub_forums f ON f.id = h.forum_id \
         JOIN users u ON u.id = f.creator_id \
         WHERE h.mode = 'CUSTOM' ORDER BY h.updated_at DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;
    if status != "ALL" {
        rows.retain(|r| r.status == status);
    }

    let total = rows.len() as i64;
    let (slice, _) = j::slice_page(rows, qp.page, qp.size);
    let content: Vec<Value> = slice
        .iter()
        .map(|r| {
            json!({
                "forumId": r.forum_id,
                "forumName": r.forum_name,
                "forumSlug": r.forum_slug,
                "creatorName": r.creator_name,
                "status": r.status,
                "sizeBytes": r.size_bytes,
                "blockReason": r.block_reason,
                "reportCount": r.report_count,
                "updatedAt": j::jts(r.updated_at),
            })
        })
        .collect();
    Ok(j::ok(j::PageResponse::of(content, qp.page, qp.size, total)))
}

/// POST /admin/forum-homepages/{forumId}/block
pub async fn block_forum_homepage(
    State(state): State<AppState>,
    admin: CurrentUser,
    Path(forum_id): Path<i64>,
    body: Option<Json<Value>>,
) -> ApiResult<Json<Value>> {
    let me = admin.require_admin()?;
    let reason = body
        .as_ref()
        .and_then(|Json(v)| v.get("reason").and_then(|r| r.as_str()))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| ApiError::bad_request("Please provide a block reason"))?;
    let chars = reason.chars().count();
    if !(5..=500).contains(&chars) {
        return Err(ApiError::bad_request("Block reason must be 5-500 characters"));
    }

    forum_homepages::find_homepage(&state.pool, forum_id)
        .await?
        .filter(|r| r.mode == "CUSTOM")
        .ok_or_else(|| ApiError::not_found("This forum has no custom homepage"))?;

    sqlx::query(
        "UPDATE forum_homepages SET status = 'BLOCKED', block_reason = ?, blocked_by = ?, \
         blocked_at = ? WHERE forum_id = ?",
    )
    .bind(&reason)
    .bind(me.id)
    .bind(chrono::Local::now().naive_local())
    .bind(forum_id)
    .execute(&state.pool)
    .await?;
    Ok(j::ok(json!({ "ok": true })))
}

/// POST /admin/forum-homepages/{forumId}/unblock
pub async fn unblock_forum_homepage(
    State(state): State<AppState>,
    admin: CurrentUser,
    Path(forum_id): Path<i64>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    forum_homepages::find_homepage(&state.pool, forum_id)
        .await?
        .filter(|r| r.mode == "CUSTOM")
        .ok_or_else(|| ApiError::not_found("This forum has no custom homepage"))?;

    sqlx::query(
        "UPDATE forum_homepages SET status = 'ACTIVE', block_reason = NULL, blocked_by = NULL, \
         blocked_at = NULL WHERE forum_id = ?",
    )
    .bind(forum_id)
    .execute(&state.pool)
    .await?;
    Ok(j::ok(json!({ "ok": true })))
}

/// POST /admin/user-homepages/{userId}/unblock
pub async fn unblock_user_homepage(
    State(state): State<AppState>,
    admin: CurrentUser,
    Path(user_id): Path<i64>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    homepages::find_homepage(&state.pool, user_id)
        .await?
        .filter(|r| r.mode == "CUSTOM")
        .ok_or_else(|| ApiError::not_found("This user has no custom homepage"))?;

    sqlx::query(
        "UPDATE user_homepages SET status = 'ACTIVE', block_reason = NULL, blocked_by = NULL, \
         blocked_at = NULL WHERE user_id = ?",
    )
    .bind(user_id)
    .execute(&state.pool)
    .await?;
    Ok(j::ok(json!({ "ok": true })))
}
