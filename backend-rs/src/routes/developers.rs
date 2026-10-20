//! Developer center: self-service OAuth application management.
//!
//! Any signed-in user can register up to [`o::MAX_APPS_PER_USER`] applications and immediately use
//! them with the OAuth endpoints. Secrets are only returned on creation and on reset.

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap};
use axum::response::IntoResponse;
use axum::Json;
use chrono::{NaiveDateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::json as j;
use crate::oauth as o;
use crate::security::CurrentUser;
use crate::state::AppState;

const OPENAPI_SPEC: &str = include_str!("../../api/openapi.yaml");

fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

fn app_dto(client: &o::OAuthClient, secret_once: Option<String>) -> Value {
    json!({
        "id": client.id,
        "clientId": client.client_id,
        "name": client.name,
        "description": client.description,
        "homepageUrl": client.homepage_url,
        "redirectUris": client.redirects(),
        "clientType": client.client_type,
        "scopes": client.scopes,
        "status": client.status,
        "hasSecret": client.secret_hash.is_some(),
        "clientSecret": secret_once,
        "createdAt": j::jts(client.created_at),
        "updatedAt": j::jts(client.updated_at),
    })
}

fn validate_name(raw: Option<&str>) -> Result<String, ApiError> {
    let name = raw.unwrap_or_default().trim();
    let len = name.chars().count();
    if !(2..=80).contains(&len) {
        return Err(ApiError::bad_request("Application name must be between 2 and 80 characters"));
    }
    Ok(name.to_string())
}

fn validate_description(raw: Option<&str>) -> Result<Option<String>, ApiError> {
    let description = raw.map(str::trim).filter(|value| !value.is_empty());
    if let Some(value) = description {
        if value.chars().count() > 500 {
            return Err(ApiError::bad_request("Description must be at most 500 characters"));
        }
    }
    Ok(description.map(str::to_string))
}

fn validate_homepage(raw: Option<&str>) -> Result<Option<String>, ApiError> {
    let Some(value) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let valid = value.len() <= 300
        && (value.starts_with("https://") || value.starts_with("http://"))
        && !value.chars().any(char::is_whitespace);
    if !valid {
        return Err(ApiError::bad_request(
            "Homepage URL must be a valid http(s) URL of at most 300 characters",
        ));
    }
    Ok(Some(value.to_string()))
}

fn normalize_redirects(raw: Option<Vec<String>>) -> Result<Vec<String>, ApiError> {
    let mut out: Vec<String> = Vec::new();
    for uri in raw.unwrap_or_default() {
        let trimmed = uri.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        if !o::is_valid_redirect_uri(&trimmed) {
            return Err(ApiError::bad_request(format!(
                "Invalid redirect URI: {trimmed} (use https, or http for localhost)"
            )));
        }
        if !out.contains(&trimmed) {
            out.push(trimmed);
        }
    }
    if out.is_empty() {
        return Err(ApiError::bad_request("At least one redirect URI is required"));
    }
    if out.len() > o::MAX_REDIRECT_URIS {
        return Err(ApiError::bad_request(format!(
            "At most {} redirect URIs are allowed",
            o::MAX_REDIRECT_URIS
        )));
    }
    Ok(out)
}

fn encode_redirects(uris: &[String]) -> String {
    serde_json::to_string(uris).unwrap_or_else(|_| "[]".to_string())
}

async fn require_owned_app(
    state: &AppState,
    user_id: i64,
    id: i64,
) -> Result<o::OAuthClient, ApiError> {
    let client = sqlx::query_as::<_, o::OAuthClient>(
        "SELECT * FROM oauth_clients WHERE id = ? AND owner_id = ? LIMIT 1",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::not_found("Application not found"))?;
    Ok(client)
}

/// GET /developers/apps
pub async fn list_apps(State(state): State<AppState>, user: CurrentUser) -> ApiResult<Json<Value>> {
    let user = user.require()?;
    let apps = sqlx::query_as::<_, o::OAuthClient>(
        "SELECT * FROM oauth_clients WHERE owner_id = ? ORDER BY id DESC",
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    let content: Vec<Value> = apps.iter().map(|app| app_dto(app, None)).collect();
    Ok(j::ok(json!({
        "content": content,
        "maxApps": o::MAX_APPS_PER_USER,
        "maxRedirectUris": o::MAX_REDIRECT_URIS,
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAppRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub client_type: Option<String>,
}

/// POST /developers/apps
pub async fn create_app(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<CreateAppRequest>,
) -> ApiResult<Json<Value>> {
    let user = user.require()?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM oauth_clients WHERE owner_id = ?")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await?;
    if count >= o::MAX_APPS_PER_USER {
        return Err(ApiError::bad_request(format!(
            "You can register at most {} applications",
            o::MAX_APPS_PER_USER
        )));
    }

    let name = validate_name(req.name.as_deref())?;
    let description = validate_description(req.description.as_deref())?;
    let homepage = validate_homepage(req.homepage_url.as_deref())?;
    let redirects = normalize_redirects(req.redirect_uris)?;
    let client_type = match req.client_type.as_deref().map(str::trim).unwrap_or("CONFIDENTIAL") {
        value if value.eq_ignore_ascii_case("PUBLIC") => "PUBLIC",
        value if value.eq_ignore_ascii_case("CONFIDENTIAL") => "CONFIDENTIAL",
        _ => return Err(ApiError::bad_request("client_type must be CONFIDENTIAL or PUBLIC")),
    };

    let client_id = o::random_token("ofc_");
    let secret = (client_type == "CONFIDENTIAL").then(|| o::random_token("ofs_"));
    let secret_hash = secret.as_deref().map(o::hash_token);
    let timestamp = now();

    let id = sqlx::query(
        "INSERT INTO oauth_clients \
         (client_id, secret_hash, name, description, homepage_url, redirect_uris, \
          client_type, scopes, owner_id, status, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'ACTIVE', ?, ?)",
    )
    .bind(&client_id)
    .bind(secret_hash.as_deref())
    .bind(&name)
    .bind(description.as_deref())
    .bind(homepage.as_deref())
    .bind(encode_redirects(&redirects))
    .bind(client_type)
    .bind(o::DEFAULT_SCOPE)
    .bind(user.id)
    .bind(timestamp)
    .bind(timestamp)
    .execute(&state.pool)
    .await?
    .last_insert_id() as i64;

    let client = require_owned_app(&state, user.id, id).await?;
    Ok(j::ok(app_dto(&client, secret)))
}

/// GET /developers/apps/{id}
pub async fn get_app(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let user = user.require()?;
    let client = require_owned_app(&state, user.id, id).await?;
    Ok(j::ok(app_dto(&client, None)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub status: Option<String>,
}

/// PUT /developers/apps/{id}
pub async fn update_app(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAppRequest>,
) -> ApiResult<Json<Value>> {
    let user = user.require()?;
    let client = require_owned_app(&state, user.id, id).await?;

    let name = match req.name.as_deref() {
        Some(value) => validate_name(Some(value))?,
        None => client.name.clone(),
    };
    let description = match req.description.as_deref() {
        Some(value) => validate_description(Some(value))?,
        None => client.description.clone(),
    };
    let homepage = match req.homepage_url.as_deref() {
        Some(value) => validate_homepage(Some(value))?,
        None => client.homepage_url.clone(),
    };
    let redirects = match req.redirect_uris {
        Some(value) => normalize_redirects(Some(value))?,
        None => client.redirects(),
    };
    let status = match req.status.as_deref().map(str::trim) {
        Some(value) if value.eq_ignore_ascii_case("ACTIVE") => "ACTIVE",
        Some(value) if value.eq_ignore_ascii_case("DISABLED") => "DISABLED",
        Some(_) => return Err(ApiError::bad_request("status must be ACTIVE or DISABLED")),
        None => client.status.as_str(),
    };

    sqlx::query(
        "UPDATE oauth_clients SET name = ?, description = ?, homepage_url = ?, \
         redirect_uris = ?, status = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(description.as_deref())
    .bind(homepage.as_deref())
    .bind(encode_redirects(&redirects))
    .bind(status)
    .bind(now())
    .bind(client.id)
    .execute(&state.pool)
    .await?;

    let updated = require_owned_app(&state, user.id, client.id).await?;
    Ok(j::ok(app_dto(&updated, None)))
}

/// DELETE /developers/apps/{id}
pub async fn delete_app(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let user = user.require()?;
    let client = require_owned_app(&state, user.id, id).await?;

    sqlx::query("DELETE FROM oauth_authorization_codes WHERE client_id = ?")
        .bind(&client.client_id)
        .execute(&state.pool)
        .await?;
    sqlx::query("DELETE FROM oauth_refresh_tokens WHERE client_id = ?")
        .bind(&client.client_id)
        .execute(&state.pool)
        .await?;
    sqlx::query("DELETE FROM oauth_clients WHERE id = ?")
        .bind(client.id)
        .execute(&state.pool)
        .await?;

    Ok(j::ok_empty())
}

/// POST /developers/apps/{id}/secret — rotates the client secret; the new value is shown once.
pub async fn reset_secret(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let user = user.require()?;
    let client = require_owned_app(&state, user.id, id).await?;
    if client.is_public() {
        return Err(ApiError::bad_request(
            "Public clients use PKCE instead of a client secret",
        ));
    }

    let secret = o::random_token("ofs_");
    sqlx::query("UPDATE oauth_clients SET secret_hash = ?, updated_at = ? WHERE id = ?")
        .bind(o::hash_token(&secret))
        .bind(now())
        .bind(client.id)
        .execute(&state.pool)
        .await?;

    let updated = require_owned_app(&state, user.id, client.id).await?;
    Ok(j::ok(app_dto(&updated, Some(secret))))
}

/// GET /developers/docs — public OpenAPI document download.
pub async fn openapi_spec() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/yaml; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "inline; filename=\"openforum-openapi.yaml\"".parse().unwrap(),
    );
    headers.insert(header::CACHE_CONTROL, "public, max-age=300".parse().unwrap());
    (headers, OPENAPI_SPEC)
}
