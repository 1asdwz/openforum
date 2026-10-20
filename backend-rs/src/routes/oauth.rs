//! OAuth 2.0 authorization-code endpoint (with PKCE), token exchange, userinfo and revocation.
//!
//! Access tokens are the same HS256 JWTs used by first-party login, so they are accepted by the
//! existing authentication middleware and inherit the authorizing user's permissions.

use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{Duration, NaiveDateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::json as j;
use crate::oauth as o;
use crate::repo;
use crate::security::{create_api_token, CurrentUser};
use crate::state::AppState;

const CODE_TTL_MINUTES: i64 = 5;
const REFRESH_TTL_DAYS: i64 = 30;

fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

pub struct TokenError {
    status: StatusCode,
    error: &'static str,
    description: String,
}

impl TokenError {
    fn new(status: StatusCode, error: &'static str, description: impl Into<String>) -> Self {
        Self { status, error, description: description.into() }
    }
    fn invalid_request(description: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_request", description)
    }
    fn invalid_grant(description: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_grant", description)
    }
    fn invalid_client(description: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "invalid_client", description)
    }
    fn internal(description: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "server_error", description)
    }
}

impl From<ApiError> for TokenError {
    fn from(err: ApiError) -> Self {
        Self::internal(err.message)
    }
}

impl From<sqlx::Error> for TokenError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("oauth database error: {err}");
        Self::internal("Internal server error")
    }
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
        headers.insert(header::PRAGMA, "no-cache".parse().unwrap());
        if self.status == StatusCode::UNAUTHORIZED {
            headers.insert(
                header::WWW_AUTHENTICATE,
                "Basic realm=\"openforum-oauth\"".parse().unwrap(),
            );
        }
        let body = json!({
            "error": self.error,
            "error_description": self.description,
        });
        (self.status, headers, Json(body)).into_response()
    }
}

// ---------------------------------------------------------------------------
// Authorization (consent page)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AuthorizeInfoQuery {
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
}

pub async fn authorize_info(
    State(state): State<AppState>,
    Query(query): Query<AuthorizeInfoQuery>,
) -> ApiResult<Json<Value>> {
    let client = o::require_client(&state.pool, query.client_id.as_deref().unwrap_or("")).await?;
    let redirect = query.redirect_uri.unwrap_or_default();
    if !redirect.is_empty() && !o::validate_redirect(&client, &redirect) {
        return Err(ApiError::bad_request("redirect_uri is not registered for this application"));
    }
    let scope = o::resolve_scope(&client, query.scope.as_deref())?;
    let owner = repo::find_user_by_id(&state.pool, client.owner_id)
        .await?
        .map(|user| user.username)
        .unwrap_or_default();
    let fallback_redirect = client.redirects().into_iter().next().unwrap_or_default();

    Ok(j::ok(json!({
        "clientId": client.client_id,
        "name": client.name,
        "description": client.description,
        "homepageUrl": client.homepage_url,
        "ownerUsername": owner,
        "clientType": client.client_type,
        "scope": scope,
        "redirectUri": if redirect.is_empty() { fallback_redirect } else { redirect },
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeRequest {
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub scope: Option<String>,
    pub approve: Option<bool>,
}

pub async fn authorize(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<AuthorizeRequest>,
) -> ApiResult<Json<Value>> {
    let user = user.require()?;
    let client = o::require_client(&state.pool, req.client_id.as_deref().unwrap_or("")).await?;
    let redirect_uri = req.redirect_uri.unwrap_or_default();
    if !o::validate_redirect(&client, &redirect_uri) {
        return Err(ApiError::bad_request("redirect_uri is not registered for this application"));
    }
    let state_param = req.state.unwrap_or_default();
    if state_param.chars().count() > 512 {
        return Err(ApiError::bad_request("state is too long"));
    }
    let scope = o::resolve_scope(&client, req.scope.as_deref())?;
    let approved = req.approve.unwrap_or(false);

    if !approved {
        let redirect_to = o::append_query(
            &redirect_uri,
            &[("error", "access_denied"), ("state", &state_param)],
        );
        return Ok(j::ok(json!({ "redirectTo": redirect_to })));
    }

    let challenge = req
        .code_challenge
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(challenge) = challenge.as_deref() {
        let method = req.code_challenge_method.as_deref().unwrap_or("S256");
        if !method.eq_ignore_ascii_case("S256") {
            return Err(ApiError::bad_request("Only S256 PKCE is supported"));
        }
        if !(43..=128).contains(&challenge.chars().count()) {
            return Err(ApiError::bad_request("Invalid PKCE code_challenge"));
        }
    } else if client.is_public() {
        return Err(ApiError::bad_request("PKCE code_challenge is required for public clients"));
    }

    let code = o::random_token("ofac_");
    sqlx::query(
        "INSERT INTO oauth_authorization_codes \
         (code_hash, client_id, user_id, redirect_uri, scope, code_challenge, code_challenge_method, expires_at, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(o::hash_token(&code))
    .bind(&client.client_id)
    .bind(user.id)
    .bind(&redirect_uri)
    .bind(&scope)
    .bind(challenge.as_deref())
    .bind(challenge.as_ref().map(|_| "S256"))
    .bind(now() + Duration::minutes(CODE_TTL_MINUTES))
    .bind(now())
    .execute(&state.pool)
    .await?;

    let redirect_to =
        o::append_query(&redirect_uri, &[("code", &code), ("state", &state_param)]);
    Ok(j::ok(json!({ "redirectTo": redirect_to })))
}

// ---------------------------------------------------------------------------
// Token endpoint
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: Option<String>,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
}

fn required(value: Option<String>, name: &str) -> Result<String, TokenError> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| TokenError::invalid_request(format!("{name} is required")))
}

fn basic_credentials(headers: &HeaderMap) -> Option<(String, String)> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?.trim();
    let encoded = raw
        .strip_prefix("Basic ")
        .or_else(|| raw.strip_prefix("basic "))?;
    let decoded = STANDARD.decode(encoded.trim()).ok()?;
    let text = String::from_utf8(decoded).ok()?;
    let (id, secret) = text.split_once(':')?;
    Some((id.to_string(), secret.to_string()))
}

async fn authenticate_client(
    state: &AppState,
    headers: &HeaderMap,
    form_client_id: Option<String>,
    form_client_secret: Option<String>,
) -> Result<o::OAuthClient, TokenError> {
    let (basic_id, basic_secret) = match basic_credentials(headers) {
        Some((id, secret)) => (Some(id), Some(secret)),
        None => (None, None),
    };
    let client_id = basic_id.or(form_client_id).unwrap_or_default();
    if client_id.trim().is_empty() {
        return Err(TokenError::invalid_client("client_id is required"));
    }
    let client = o::find_client(&state.pool, client_id.trim())
        .await?
        .ok_or_else(|| TokenError::invalid_client("Unknown or disabled client"))?;
    if !client.is_public() {
        let secret = basic_secret
            .or(form_client_secret)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| TokenError::invalid_client("Client authentication required"))?;
        let expected = client.secret_hash.clone().unwrap_or_default();
        if expected.is_empty() || o::hash_token(&secret) != expected {
            return Err(TokenError::invalid_client("Invalid client credentials"));
        }
    }
    Ok(client)
}

async fn create_refresh_token(
    state: &AppState,
    client_id: &str,
    user_id: i64,
    scope: &str,
) -> Result<String, TokenError> {
    let token = o::random_token("ofrt_");
    sqlx::query(
        "INSERT INTO oauth_refresh_tokens \
         (token_hash, client_id, user_id, scope, expires_at, created_at) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(o::hash_token(&token))
    .bind(client_id)
    .bind(user_id)
    .bind(scope)
    .bind(now() + Duration::days(REFRESH_TTL_DAYS))
    .bind(now())
    .execute(&state.pool)
    .await?;
    Ok(token)
}

fn token_payload(access_token: String, expires_in: i64, refresh_token: String, scope: &str) -> Value {
    json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": expires_in,
        "refresh_token": refresh_token,
        "scope": scope,
    })
}

fn no_store_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    headers.insert(header::PRAGMA, "no-cache".parse().unwrap());
    headers
}

#[derive(Debug, sqlx::FromRow)]
struct CodeRow {
    client_id: String,
    user_id: i64,
    redirect_uri: String,
    scope: String,
    code_challenge: Option<String>,
    expires_at: NaiveDateTime,
    used_at: Option<NaiveDateTime>,
}

pub async fn token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(req): Form<TokenRequest>,
) -> Result<(HeaderMap, Json<Value>), TokenError> {
    let grant_type = required(req.grant_type.clone(), "grant_type")?;
    match grant_type.as_str() {
        "authorization_code" => {
            let code = required(req.code.clone(), "code")?;
            let redirect_uri = required(req.redirect_uri.clone(), "redirect_uri")?;
            let client = authenticate_client(
                &state,
                &headers,
                req.client_id.clone(),
                req.client_secret.clone(),
            )
            .await?;

            let code_hash = o::hash_token(&code);
            let row = sqlx::query_as::<_, CodeRow>(
                "SELECT client_id, user_id, redirect_uri, scope, code_challenge, expires_at, used_at \
                 FROM oauth_authorization_codes WHERE code_hash = ? LIMIT 1",
            )
            .bind(&code_hash)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| TokenError::invalid_grant("Authorization code is invalid"))?;

            if row.used_at.is_some() {
                return Err(TokenError::invalid_grant("Authorization code was already used"));
            }
            if row.expires_at < now() {
                return Err(TokenError::invalid_grant("Authorization code has expired"));
            }
            if row.client_id != client.client_id || row.redirect_uri != redirect_uri {
                return Err(TokenError::invalid_grant("Authorization code does not match this request"));
            }

            if let Some(challenge) = row.code_challenge.as_deref() {
                let verifier = required(req.code_verifier.clone(), "code_verifier")?;
                if !o::valid_pkce_verifier(&verifier)
                    || o::pkce_challenge(&verifier) != challenge
                {
                    return Err(TokenError::invalid_grant("Invalid PKCE code_verifier"));
                }
            } else if client.is_public() {
                return Err(TokenError::invalid_grant("PKCE is required for public clients"));
            }

            let consumed = sqlx::query(
                "UPDATE oauth_authorization_codes SET used_at = ? \
                 WHERE code_hash = ? AND used_at IS NULL",
            )
            .bind(now())
            .bind(&code_hash)
            .execute(&state.pool)
            .await?;
            if consumed.rows_affected() != 1 {
                return Err(TokenError::invalid_grant("Authorization code was already used"));
            }

            let user = repo::require_user(&state.pool, row.user_id).await?;
            let (access_token, expires_in) =
                create_api_token(&state.cfg, &user, &client.client_id, &row.scope);
            let refresh_token =
                create_refresh_token(&state, &client.client_id, user.id, &row.scope).await?;
            Ok((
                no_store_headers(),
                Json(token_payload(access_token, expires_in, refresh_token, &row.scope)),
            ))
        }
        "refresh_token" => {
            let refresh_token = required(req.refresh_token.clone(), "refresh_token")?;
            let client = authenticate_client(
                &state,
                &headers,
                req.client_id.clone(),
                req.client_secret.clone(),
            )
            .await?;
            let token_hash = o::hash_token(&refresh_token);
            let row: Option<(String, i64, String, NaiveDateTime, Option<NaiveDateTime>)> =
                sqlx::query_as(
                    "SELECT client_id, user_id, scope, expires_at, revoked_at \
                     FROM oauth_refresh_tokens WHERE token_hash = ? LIMIT 1",
                )
                .bind(&token_hash)
                .fetch_optional(&state.pool)
                .await?;
            let (stored_client, user_id, scope, expires_at, revoked_at) =
                row.ok_or_else(|| TokenError::invalid_grant("Refresh token is invalid"))?;
            if stored_client != client.client_id {
                return Err(TokenError::invalid_grant("Refresh token does not belong to this client"));
            }
            if revoked_at.is_some() {
                return Err(TokenError::invalid_grant("Refresh token was revoked"));
            }
            if expires_at < now() {
                return Err(TokenError::invalid_grant("Refresh token has expired"));
            }
            let rotated = sqlx::query(
                "UPDATE oauth_refresh_tokens SET revoked_at = ? \
                 WHERE token_hash = ? AND revoked_at IS NULL",
            )
            .bind(now())
            .bind(&token_hash)
            .execute(&state.pool)
            .await?;
            if rotated.rows_affected() != 1 {
                return Err(TokenError::invalid_grant("Refresh token was revoked"));
            }

            let user = repo::require_user(&state.pool, user_id).await?;
            let (access_token, expires_in) =
                create_api_token(&state.cfg, &user, &client.client_id, &scope);
            let new_refresh =
                create_refresh_token(&state, &client.client_id, user.id, &scope).await?;
            Ok((
                no_store_headers(),
                Json(token_payload(access_token, expires_in, new_refresh, &scope)),
            ))
        }
        other => Err(TokenError::new(
            StatusCode::BAD_REQUEST,
            "unsupported_grant_type",
            format!("Unsupported grant_type: {other}"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Userinfo & revocation
// ---------------------------------------------------------------------------

/// GET /oauth/userinfo — returns the profile of the token's user in the plain OAuth style
/// (no response envelope, since third-party clients consume this directly).
pub async fn userinfo(
    State(state): State<AppState>,
    user: CurrentUser,
) -> ApiResult<Json<Value>> {
    let auth = user.require()?;
    let record = repo::require_user(&state.pool, auth.id).await?;
    Ok(Json(json!({
        "sub": record.id.to_string(),
        "preferred_username": record.username,
        "name": record.username,
        "picture": record.avatar,
        "role": record.role,
    })))
}

#[derive(Debug, Deserialize)]
pub struct RevokeRequest {
    pub token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

pub async fn revoke(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(req): Form<RevokeRequest>,
) -> Result<StatusCode, TokenError> {
    let client = authenticate_client(
        &state,
        &headers,
        req.client_id.clone(),
        req.client_secret.clone(),
    )
    .await?;
    if let Some(token) = req.token.filter(|value| !value.trim().is_empty()) {
        sqlx::query(
            "UPDATE oauth_refresh_tokens SET revoked_at = ? \
             WHERE token_hash = ? AND client_id = ? AND revoked_at IS NULL",
        )
        .bind(now())
        .bind(o::hash_token(token.trim()))
        .bind(&client.client_id)
        .execute(&state.pool)
        .await?;
    }
    Ok(StatusCode::OK)
}
