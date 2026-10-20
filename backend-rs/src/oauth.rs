//! Shared OAuth 2.0 primitives: client records, token generation, PKCE and redirect helpers.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::NaiveDateTime;
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

use crate::error::ApiError;

pub const DEFAULT_SCOPE: &str = "api";
pub const MAX_APPS_PER_USER: i64 = 10;
pub const MAX_REDIRECT_URIS: usize = 5;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OAuthClient {
    pub id: i64,
    pub client_id: String,
    pub secret_hash: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    pub redirect_uris: String,
    pub client_type: String,
    pub scopes: String,
    pub owner_id: i64,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl OAuthClient {
    pub fn is_public(&self) -> bool {
        self.client_type == "PUBLIC"
    }

    pub fn redirects(&self) -> Vec<String> {
        serde_json::from_str::<Vec<String>>(&self.redirect_uris).unwrap_or_default()
    }
}

/// Random URL-safe token with a human-readable prefix (used for client ids, secrets, codes and refresh tokens).
pub fn random_token(prefix: &str) -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    format!("{prefix}{}", URL_SAFE_NO_PAD.encode(buf))
}

/// Tokens are stored hashed so a database leak cannot be replayed.
pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub async fn find_client(pool: &MySqlPool, client_id: &str) -> Result<Option<OAuthClient>, ApiError> {
    let client = sqlx::query_as::<_, OAuthClient>(
        "SELECT * FROM oauth_clients WHERE client_id = ? AND status = 'ACTIVE' LIMIT 1",
    )
    .bind(client_id)
    .fetch_optional(pool)
    .await?;
    Ok(client)
}

pub async fn require_client(pool: &MySqlPool, client_id: &str) -> Result<OAuthClient, ApiError> {
    find_client(pool, client_id)
        .await?
        .ok_or_else(|| ApiError::bad_request("Unknown or disabled OAuth client"))
}

pub fn validate_redirect(client: &OAuthClient, uri: &str) -> bool {
    client.redirects().iter().any(|registered| registered == uri)
}

/// A redirect URI must be https, except for loopback hosts used during local development.
pub fn is_valid_redirect_uri(raw: &str) -> bool {
    let uri = raw.trim();
    if uri.is_empty() || uri.len() > 500 || uri.contains('#') {
        return false;
    }
    if uri.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    if uri.starts_with("https://") {
        return uri.len() > "https://".len();
    }
    uri.starts_with("http://localhost") || uri.starts_with("http://127.0.0.1")
}

/// Requested scope must be a subset of the scopes granted to the client (currently a single `api` scope).
pub fn resolve_scope(client: &OAuthClient, requested: Option<&str>) -> Result<String, ApiError> {
    let requested = requested.map(str::trim).filter(|s| !s.is_empty());
    let Some(requested) = requested else {
        return Ok(client.scopes.clone());
    };
    let allowed: Vec<&str> = client.scopes.split_whitespace().collect();
    for scope in requested.split_whitespace() {
        if !allowed.contains(&scope) {
            return Err(ApiError::bad_request("Requested scope is not allowed for this application"));
        }
    }
    Ok(requested.to_string())
}

/// PKCE S256: BASE64URL(SHA256(verifier)) without padding.
pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub fn valid_pkce_verifier(verifier: &str) -> bool {
    let len = verifier.chars().count();
    (43..=128).contains(&len)
        && verifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~'))
}

/// Appends query parameters, percent-encoding keys and values per RFC 3986.
pub fn append_query(base: &str, params: &[(&str, &str)]) -> String {
    let mut url = base.to_string();
    for (key, value) in params.iter() {
        if value.is_empty() {
            continue;
        }
        url.push(if url.contains('?') { '&' } else { '?' });
        url.push_str(&percent_encode(key));
        url.push('=');
        url.push_str(&percent_encode(value));
    }
    url
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_uri_rules() {
        assert!(is_valid_redirect_uri("https://example.com/callback"));
        assert!(is_valid_redirect_uri("http://localhost:3000/cb"));
        assert!(is_valid_redirect_uri("http://127.0.0.1:8080/cb"));
        assert!(!is_valid_redirect_uri("http://example.com/cb"));
        assert!(!is_valid_redirect_uri("https://example.com/cb#fragment"));
        assert!(!is_valid_redirect_uri("javascript:alert(1)"));
    }

    #[test]
    fn pkce_matches_rfc_vector() {
        // RFC 7636 Appendix B
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(pkce_challenge(verifier), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
        assert!(valid_pkce_verifier(verifier));
    }

    #[test]
    fn query_appending() {
        assert_eq!(
            append_query("https://app.example/cb", &[("code", "a b"), ("state", "x/y")]),
            "https://app.example/cb?code=a%20b&state=x%2Fy"
        );
        assert_eq!(
            append_query("https://app.example/cb?mode=1", &[("code", "abc")]),
            "https://app.example/cb?mode=1&code=abc"
        );
    }
}
