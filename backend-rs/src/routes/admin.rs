use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::enums::AuthLevel;
use crate::error::{ApiError, ApiResult};
use crate::json as j;
use crate::models::UserAuthentication;
use crate::repo;
use crate::routes::authentications::auth_dto;
use crate::routes::{forums, users};
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
        _ => Err(ApiError::bad_request(format!("非法的状态: {raw}"))),
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
        .ok_or_else(|| ApiError::not_found("认证记录不存在"))?;
    let reason = body
        .as_ref()
        .and_then(|Json(v)| v.get("reason").and_then(|r| r.as_str()).map(|s| s.to_string()))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "管理员通过".to_string());

    auth.status = "APPROVED".to_string();
    auth.review_method = Some("MANUAL".to_string());
    auth.reviewed_by = Some(me.id);
    auth.reviewed_at = Some(chrono::Local::now().naive_local());
    auth.reason = Some(reason);
    auth.rejection_reason = None;

    // 手动通过同样要把认证结果写入用户信息项，供板块门槛判定使用。
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
        .ok_or_else(|| ApiError::not_found("认证记录不存在"))?;
    let reason = body
        .as_ref()
        .and_then(|Json(v)| v.get("reason").and_then(|r| r.as_str()).map(|s| s.to_string()))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "管理员驳回".to_string());

    auth.status = "REJECTED".to_string();
    auth.review_method = Some("MANUAL".to_string());
    auth.reviewed_by = Some(me.id);
    auth.reviewed_at = Some(chrono::Local::now().naive_local());
    auth.rejection_reason = Some(reason);
    repo::save_auth(&state.pool, &mut auth).await?;
    Ok(j::ok(auth_dto(&state, &auth).await?))
}

/// GET /admin/custom-pages
pub async fn list_custom_pages(
    state: State<AppState>,
    admin: CurrentUser,
    query: Query<forums::CustomPageListQuery>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    forums::admin_list_custom_pages(state, query).await
}

/// POST /admin/custom-pages/{id}/approve
pub async fn approve_custom_page(
    state: State<AppState>,
    admin: CurrentUser,
    path: Path<i64>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    forums::admin_approve_custom_page(state, path).await
}

/// POST /admin/custom-pages/{id}/reject
pub async fn reject_custom_page(
    state: State<AppState>,
    admin: CurrentUser,
    path: Path<i64>,
    body: Option<Json<Value>>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    forums::admin_reject_custom_page(state, path, body).await
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
