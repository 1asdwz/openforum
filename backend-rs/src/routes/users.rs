use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::dto;
use crate::error::ApiResult;
use crate::json as j;
use crate::repo;
use crate::routes::{comments, posts};
use crate::security::CurrentUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct UserPostsQuery {
    #[serde(default = "default_new")]
    pub sort: String,
    #[serde(default)]
    pub page: i32,
    #[serde(default = "default_size")]
    pub size: i32,
}

fn default_new() -> String {
    "new".to_string()
}
fn default_size() -> i32 {
    20
}

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    #[serde(default)]
    pub page: i32,
    #[serde(default = "default_size")]
    pub size: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

/// GET /users/{id}
pub async fn profile(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let user = repo::require_user(&state.pool, id).await?;
    // 隐私：公开资料不返回认证徽章与具体信息项。
    Ok(j::ok(dto::user_dto(&user, Vec::new(), false)))
}

/// GET /users/{id}/posts
pub async fn user_posts(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Query(q): Query<UserPostsQuery>,
) -> ApiResult<Json<Value>> {
    repo::require_user(&state.pool, id).await?;
    let user_id = user.0.as_ref().map(|u| u.id);
    let is_admin = user.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    posts::list_by_author(&state, id, user_id, is_admin, &q.sort, q.page, q.size).await
}

/// GET /users/{id}/comments
pub async fn user_comments(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<Value>> {
    repo::require_user(&state.pool, id).await?;
    let user_id = user.0.as_ref().map(|u| u.id);
    let is_admin = user.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    comments::list_by_author(&state, id, user_id, is_admin, q.page, q.size).await
}

/// PUT /users/me
pub async fn update_me(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<UpdateProfileRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let mut u = repo::require_user(&state.pool, me.id).await?;
    repo::update_user_profile(
        &state.pool,
        u.id,
        req.avatar.as_deref(),
        req.bio.as_deref(),
        None,
    )
    .await?;
    if let Some(a) = req.avatar {
        u.avatar = Some(a);
    }
    if let Some(b) = req.bio {
        u.bio = Some(b);
    }
    let auths = repo::approved_type_names(&state.pool, u.id).await?;
    Ok(j::ok(dto::user_dto(&u, auths, true)))
}

/// 供 admin 列表复用：分页返回用户（email 置 null）。
pub async fn list_users_page(
    state: &AppState,
    page: i32,
    size: i32,
) -> ApiResult<Json<Value>> {
    let rows: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM users ORDER BY created_at DESC")
            .fetch_all(&state.pool)
            .await?;
    let ids: Vec<i64> = rows.into_iter().map(|(id,)| id).collect();
    let total = ids.len() as i64;
    let (slice, _) = j::slice_page(ids, page, size);
    let mut content = Vec::with_capacity(slice.len());
    for id in slice {
        if let Some(user) = repo::find_user_by_id(&state.pool, id).await? {
            let auths = repo::approved_type_names(&state.pool, user.id).await?;
            content.push(dto::user_dto(&user, auths, false));
        }
    }
    let page_resp = j::PageResponse::of(content, page, size, total);
    Ok(j::ok(json!(page_resp)))
}
