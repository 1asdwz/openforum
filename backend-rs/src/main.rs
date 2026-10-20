mod access;
mod auth_gate;
mod config;
mod db;
mod dto;
mod enums;
mod error;
mod gate;
mod info;
mod json;
mod migrate;
mod models;
mod oauth;
mod plugins;
mod repo;
mod review;
mod routes;
mod sanitizer;
mod security;
mod seeder;
mod state;
mod storage;
mod vote;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::Method;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::plugins::PluginRegistry;
use crate::security::parse_token;
use crate::state::AppState;
use crate::storage::Storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cfg = Arc::new(Config::from_env());
    cfg.validate()?;
    tracing::info!(
        "Starting OpenForum Rust backend on port {}, database {}:{} / {}",
        cfg.port,
        cfg.db_host,
        cfg.db_port,
        cfg.db_name
    );

    let pool = db::connect(&cfg).await?;
    tracing::info!("MySQL connection pool ready");

    let storage = Arc::new(Storage::from_config(&cfg)?);
    if !cfg.minio_enabled {
        tracing::warn!("MinIO is disabled (MINIO_ENABLED=false); media uploads and reads will be rejected");
    }

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(cfg.ai_timeout_seconds.max(5)))
        .build()?;

    let plugins = Arc::new(PluginRegistry::new());

    let state = AppState {
        cfg: cfg.clone(),
        pool: pool.clone(),
        http,
        storage,
        plugins,
    };

    // Seed data: runs only when the database is empty; failures are logged as warnings (startup is not blocked).
    if let Err(e) = seeder::seed(&state).await {
        tracing::warn!("Failed to initialize seed data (ignored): {e}");
    }

    // Info field and forum gate migrations: idempotent; failures are logged as warnings (gate checks fall back to legacy logic).
    if let Err(e) = migrate::run(&state).await {
        tracing::warn!("Data migration failed (ignored): {e}");
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // The gate middleware needs its own `Arc<Config>` (state is moved into `.with_state()` afterwards).
    let gate_state = cfg.clone();

    let app = Router::new()
        .nest("/api", routes::router())
        // Unmatched paths match Java's `GlobalExceptionHandler.handleNotFound`: 404 + `Resource not found`.
        .fallback(not_found)
        // Matches Java's `spring.servlet.multipart.max-request-size: 60MB`.
        // axum's Multipart/Json extractors default to only 2MB; without raising this, image (10MB) and video (50MB) uploads fail.
        .layer(DefaultBodyLimit::max(60 * 1024 * 1024))
        // Mirrors Java `SecurityConfig` path-level authorization (filter layer 401/403).
        .layer(middleware::from_fn_with_state(gate_state, spring_security_gate))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on http://{addr} (API prefix /api)");
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

/// Fallback when no route matches, aligned with Java's `GlobalExceptionHandler.handleNotFound`:
/// HTTP 404 with body `{code:404,message:"Resource not found",data:null}`.
async fn not_found() -> crate::error::ApiError {
    crate::error::ApiError::not_found("Resource not found")
}

/// Spring Ant `prefix/**` semantics: matches `prefix` itself or anything starting with `prefix/`.
fn under_prefix(prefix: &str, path: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}/"))
}

/// Mirrors Java `SecurityConfig.PUBLIC_PATHS`, the set of paths allowed anonymously.
/// The input is the path with the `/api` prefix removed (Java registers both prefixed and unprefixed sets; here we match uniformly without the prefix).
fn is_public_path(path: &str) -> bool {
    under_prefix("/auth", path)
        || under_prefix("/forums", path)
        || under_prefix("/posts", path)
        || under_prefix("/comments", path)
        || under_prefix("/users", path)
        || path == "/plugins"
        || path == "/authentications/types"
        // OAuth endpoints are reachable anonymously; handlers that need a user enforce it themselves
        // (`CurrentUser` inside `authorize` / `userinfo`).
        || under_prefix("/oauth", path)
        // Public API reference download.
        || path == "/developers/docs"
}

/// Mirrors Java `SecurityConfig` path-level authorization (filter layer), ensuring unauthenticated/unauthorized
/// responses have the body `{code,message}` (without `data`), exactly like `writeJson`.
async fn spring_security_gate(
    State(cfg): State<Arc<Config>>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    // Routes are mounted under `/api`; strip the prefix here to match Java's unprefixed form.
    let p = path.strip_prefix("/api").unwrap_or(&path).to_string();

    // Allow OPTIONS preflight (Java `OPTIONS /** permitAll`). CorsLayer usually short-circuits this; this is a fallback.
    if method == Method::OPTIONS {
        return next.run(req).await;
    }
    // GET /media/** permitAll (MinIO media proxy).
    if method == Method::GET && under_prefix("/media", &p) {
        return next.run(req).await;
    }

    let user = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|t| parse_token(&cfg, t.trim()));
    let authenticated = user.is_some();
    let is_admin = user.as_ref().map(|u| u.is_admin()).unwrap_or(false);

    // `/admin/**` requires the ADMIN role (matching the `ADMIN_PATHS` set).
    if under_prefix("/admin", &p) {
        return if is_admin {
            next.run(req).await
        } else if authenticated {
            crate::error::ApiError::raw(403, "forbidden").into_response()
        } else {
            crate::error::ApiError::raw(401, "unauthorized").into_response()
        };
    }

    // Public paths are allowed.
    if is_public_path(&p) {
        return next.run(req).await;
    }

    // All other paths use `anyRequest().authenticated()`.
    if authenticated {
        next.run(req).await
    } else {
        crate::error::ApiError::raw(401, "unauthorized").into_response()
    }
}
