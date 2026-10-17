use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::dto;
use crate::error::{ApiError, ApiResult};
use crate::json as j;
use crate::repo;
use crate::security::{create_token, hash_password, verify_password, CurrentUser};
use crate::state::AppState;

const MAX_USERNAME_LENGTH: usize = 50;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username_or_email: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleLoginRequest {
    pub credential: Option<String>,
}

fn blank(s: &Option<String>) -> bool {
    s.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true)
}

fn value_of(s: &Option<String>) -> String {
    s.clone().unwrap_or_default()
}

fn is_email(v: &str) -> bool {
    let at = match v.find('@') {
        Some(i) => i,
        None => return false,
    };
    let (local, domain) = v.split_at(at);
    let domain = &domain[1..];
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

/// 对应 Java `AuthService.register`。
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<Json<Value>> {
    if blank(&req.username) {
        return Err(ApiError::bad_request("username 用户名不能为空"));
    }
    let username = req.username.as_deref().unwrap().trim().to_string();
    let ulen = username.chars().count();
    if !(3..=50).contains(&ulen) {
        return Err(ApiError::bad_request("username 长度需在 3-50 之间"));
    }
    if blank(&req.email) {
        return Err(ApiError::bad_request("email 邮箱不能为空"));
    }
    let email = req.email.as_deref().unwrap().trim().to_string();
    if !is_email(&email) {
        return Err(ApiError::bad_request("email 格式不正确"));
    }
    if blank(&req.password) {
        return Err(ApiError::bad_request("password 密码不能为空"));
    }
    let password = req.password.as_deref().unwrap().to_string();
    let plen = password.chars().count();
    if !(6..=64).contains(&plen) {
        return Err(ApiError::bad_request("password 长度需在 6-64 之间"));
    }

    if repo::exists_username(&state.pool, &username).await? {
        return Err(ApiError::bad_request("用户名已被占用"));
    }
    if repo::exists_email(&state.pool, &email).await? {
        return Err(ApiError::bad_request("邮箱已被占用"));
    }

    let hash = hash_password(&password)?;
    let user = repo::insert_user(
        &state.pool,
        &username,
        Some(&email),
        &hash,
        None,
        None,
        "USER",
        None,
    )
    .await?;

    let token = create_token(&state.cfg, &user);
    let auths = repo::approved_type_names(&state.pool, user.id).await?;
    let user_dto = dto::user_dto(&user, auths, true);
    Ok(j::ok(json!({ "token": token, "user": user_dto })))
}

/// 对应 Java `AuthService.login`。
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<Value>> {
    if blank(&req.username_or_email) {
        return Err(ApiError::bad_request("usernameOrEmail 账号不能为空"));
    }
    if blank(&req.password) {
        return Err(ApiError::bad_request("password 密码不能为空"));
    }
    let value = req.username_or_email.as_deref().unwrap().trim().to_string();
    let password = req.password.as_deref().unwrap();

    let user = repo::find_user_by_username_or_email(&state.pool, &value)
        .await?
        .ok_or_else(|| ApiError::unauthorized("账号或密码错误"))?;
    if !user.active {
        return Err(ApiError::forbidden("账号已被禁用"));
    }
    if !verify_password(password, &user.password) {
        return Err(ApiError::unauthorized("账号或密码错误"));
    }

    let token = create_token(&state.cfg, &user);
    let auths = repo::approved_type_names(&state.pool, user.id).await?;
    let user_dto = dto::user_dto(&user, auths, true);
    Ok(j::ok(json!({ "token": token, "user": user_dto })))
}

struct GoogleUser {
    google_id: String,
    email: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}

/// 对应 Java `GoogleTokenVerifier.verify`。
async fn verify_google(state: &AppState, credential: &str) -> Result<GoogleUser, ApiError> {
    if !state.cfg.google_configured() {
        return Err(ApiError::bad_request("Google 登录未配置"));
    }
    if credential.trim().is_empty() {
        return Err(ApiError::unauthorized("Google 登录凭证无效"));
    }
    let url = "https://oauth2.googleapis.com/tokeninfo";
    let resp = state
        .http
        .get(url)
        .query(&[("id_token", credential)])
        .send()
        .await
        .map_err(|e| {
            tracing::warn!("Google tokeninfo request failed: {e}");
            ApiError::unauthorized("Google 登录凭证无效")
        })?;
    let body = resp.text().await.map_err(|_| ApiError::unauthorized("Google 登录凭证无效"))?;
    if body.trim().is_empty() {
        return Err(ApiError::unauthorized("Google 登录凭证无效"));
    }
    let claims: Value =
        serde_json::from_str(&body).map_err(|_| ApiError::unauthorized("Google 登录凭证无效"))?;
    if !claims.is_object() {
        return Err(ApiError::unauthorized("Google 登录凭证无效"));
    }

    // audience 校验
    let allowed: Vec<String> = state
        .cfg
        .google_client_id
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let aud_ok = if allowed.is_empty() {
        false
    } else {
        match claims.get("aud") {
            Some(Value::Array(arr)) => arr
                .iter()
                .any(|v| v.as_str().map(|s| allowed.iter().any(|a| a == s)).unwrap_or(false)),
            Some(Value::String(s)) => allowed.iter().any(|a| a == s),
            _ => false,
        }
    };
    if !aud_ok {
        return Err(ApiError::unauthorized("Google 登录凭证无效"));
    }

    // email_verified
    let email_verified = match claims.get("email_verified") {
        Some(Value::Bool(b)) => *b,
        Some(other) => other.as_str().map(|s| s.eq_ignore_ascii_case("true")).unwrap_or(false),
        None => false,
    };
    if !email_verified {
        return Err(ApiError::bad_request("Google 账号邮箱未验证"));
    }

    let sub = claims.get("sub").and_then(|v| v.as_str()).unwrap_or("").trim();
    if sub.is_empty() {
        return Err(ApiError::unauthorized("Google 登录凭证无效"));
    }

    let str_claim = |key: &str| -> Option<String> {
        claims.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };
    Ok(GoogleUser {
        google_id: sub.to_string(),
        email: str_claim("email"),
        name: str_claim("name"),
        picture: str_claim("picture"),
    })
}

async fn available_username(
    state: &AppState,
    email: Option<&str>,
    name: Option<&str>,
) -> Result<String, ApiError> {
    let raw = email.unwrap_or("");
    let base_src = raw.split('@').next().unwrap_or("");
    let fallback = name.unwrap_or("");
    let mut base: String = if base_src.is_empty() { fallback } else { base_src }
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if base.is_empty() {
        base = "user".to_string();
    }
    if base.chars().count() > MAX_USERNAME_LENGTH {
        base = base.chars().take(MAX_USERNAME_LENGTH).collect();
    }
    let mut candidate = base.clone();
    let mut suffix = 0;
    while repo::exists_username(&state.pool, &candidate).await? {
        suffix += 1;
        let suffix_text = suffix.to_string();
        let max_base = MAX_USERNAME_LENGTH.saturating_sub(suffix_text.len());
        let prefix: String = base.chars().take(max_base).collect();
        candidate = format!("{prefix}{suffix_text}");
    }
    Ok(candidate)
}

/// 对应 Java `AuthService.loginWithGoogle`。
pub async fn google(
    State(state): State<AppState>,
    Json(req): Json<GoogleLoginRequest>,
) -> ApiResult<Json<Value>> {
    if !state.cfg.google_configured() {
        return Err(ApiError::bad_request("Google 登录未配置"));
    }
    if blank(&req.credential) {
        return Err(ApiError::bad_request("credential Google 登录凭证不能为空"));
    }
    let credential = value_of(&req.credential);
    let gu = verify_google(&state, &credential).await?;

    let mut user = match repo::find_user_by_google_id(&state.pool, &gu.google_id).await? {
        Some(u) => Some(u),
        None => match gu.email.as_deref() {
            Some(email) if !email.trim().is_empty() => {
                let found = repo::find_user_by_email(&state.pool, email).await?;
                if let Some(mut u) = found {
                    repo::update_user_profile(&state.pool, u.id, None, None, Some(&gu.google_id))
                        .await?;
                    u.google_id = Some(gu.google_id.clone());
                    Some(u)
                } else {
                    None
                }
            }
            _ => None,
        },
    };

    if let Some(ref existing) = user {
        if !existing.active {
            return Err(ApiError::forbidden("账号已被禁用"));
        }
    } else {
        let username = available_username(&state, gu.email.as_deref(), gu.name.as_deref()).await?;
        let random_pw = uuid::Uuid::new_v4().to_string();
        let hash = hash_password(&random_pw)?;
        let created = repo::insert_user(
            &state.pool,
            &username,
            gu.email.as_deref(),
            &hash,
            gu.picture.as_deref(),
            Some(&gu.google_id),
            "USER",
            None,
        )
        .await?;
        user = Some(created);
    }

    let user = user.unwrap();
    let token = create_token(&state.cfg, &user);
    let auths = repo::approved_type_names(&state.pool, user.id).await?;
    let user_dto = dto::user_dto(&user, auths, true);
    Ok(j::ok(json!({ "token": token, "user": user_dto })))
}

/// 对应 Java `AuthController.me`。
pub async fn me(
    State(state): State<AppState>,
    user: CurrentUser,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let u = repo::require_user(&state.pool, me.id).await?;
    let auths = repo::approved_type_names(&state.pool, u.id).await?;
    Ok(j::ok(dto::user_dto(&u, auths, true)))
}
