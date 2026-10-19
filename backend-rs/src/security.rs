use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::ApiError;
use crate::models::User;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub role: String,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.role == "ADMIN"
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    role: String,
    iat: i64,
    exp: i64,
}

/// Aligned with Java's `JwtTokenProvider`: HS256, subject=id, claims=username/role.
pub fn create_token(cfg: &Config, user: &User) -> String {
    let now = Utc::now();
    let expiry = now + Duration::hours(cfg.jwt_expiration_hours);
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: user.role.clone(),
        iat: now.timestamp(),
        exp: expiry.timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .expect("jwt encode")
}

/// Parse a token; any error yields None (matching the Java filter: a failed parse means unauthenticated).
pub fn parse_token(cfg: &Config, token: &str) -> Option<AuthUser> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .ok()?;
    let id = data.claims.sub.parse::<i64>().ok()?;
    Some(AuthUser {
        id,
        role: data.claims.role,
    })
}

pub fn hash_password(raw: &str) -> Result<String, ApiError> {
    bcrypt::hash(raw, 10).map_err(|e| ApiError::internal(format!("Failed to hash password: {e}")))
}

pub fn verify_password(raw: &str, hash: &str) -> bool {
    bcrypt::verify(raw, hash).unwrap_or(false)
}

/// Request-scoped current user (may be absent). Mirrors Java's `SecurityUtils.currentUser()`.
pub struct CurrentUser(pub Option<AuthUser>);

impl CurrentUser {
    pub fn require(&self) -> Result<&AuthUser, ApiError> {
        self.0.as_ref().ok_or_else(|| ApiError::unauthorized("Please sign in first"))
    }
    pub fn require_admin(&self) -> Result<&AuthUser, ApiError> {
        match self.0.as_ref() {
            None => Err(ApiError::unauthorized("Please sign in first")),
            Some(u) if u.is_admin() => Ok(u),
            Some(_) => Err(ApiError::forbidden("Administrator privileges required")),
        }
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .and_then(|t| parse_token(&state.cfg, t.trim()));
        Ok(CurrentUser(user))
    }
}
