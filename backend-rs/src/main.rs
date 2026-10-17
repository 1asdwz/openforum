mod access;
mod auth_gate;
mod config;
mod db;
mod dto;
mod enums;
mod error;
mod json;
mod models;
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
        "OpenForum Rust 后端启动中，端口 {}，数据库 {}:{} / {}",
        cfg.port,
        cfg.db_host,
        cfg.db_port,
        cfg.db_name
    );

    let pool = db::connect(&cfg).await?;
    tracing::info!("MySQL 连接池就绪");

    let storage = Arc::new(Storage::from_config(&cfg)?);
    if !cfg.minio_enabled {
        tracing::warn!("MinIO 未启用（MINIO_ENABLED=false），媒体上传/读取将被拒绝");
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

    // 种子数据：仅在数据库为空时执行，失败仅告警（不阻断启动）。
    if let Err(e) = seeder::seed(&state).await {
        tracing::warn!("种子数据初始化失败（忽略）: {e}");
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 网关中间件需要独立的 `Arc<Config>`（state 随后会被移动进 `.with_state()`）。
    let gate_state = cfg.clone();

    let app = Router::new()
        .nest("/api", routes::router())
        // 未匹配路径与 Java `GlobalExceptionHandler.handleNotFound` 对齐：404 + `资源不存在`。
        .fallback(not_found)
        // 与 Java `spring.servlet.multipart.max-request-size: 60MB` 对齐。
        // axum 的 Multipart/Json 提取器默认上限仅 2MB，不放开会导致图片(10MB)/视频(50MB)上传失败。
        .layer(DefaultBodyLimit::max(60 * 1024 * 1024))
        // 复刻 Java `SecurityConfig` 的路径级鉴权（过滤器层 401/403）。
        .layer(middleware::from_fn_with_state(gate_state, spring_security_gate))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("监听 http://{addr}（API 前缀 /api）");
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

/// 未匹配任何路由时的兜底，与 Java `GlobalExceptionHandler.handleNotFound` 对齐：
/// HTTP 404，body `{code:404,message:"资源不存在",data:null}`。
async fn not_found() -> crate::error::ApiError {
    crate::error::ApiError::not_found("资源不存在")
}

/// Spring Ant `prefix/**` 语义：匹配 `prefix` 自身或以 `prefix/` 开头。
fn under_prefix(prefix: &str, path: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}/"))
}

/// 复刻 Java `SecurityConfig.PUBLIC_PATHS` 的匿名放行集合。
/// 入参为去掉 `/api` 前缀后的路径（Java 同时登记了带/不带前缀两套，这里统一按非前缀形式匹配）。
fn is_public_path(path: &str) -> bool {
    under_prefix("/auth", path)
        || under_prefix("/forums", path)
        || under_prefix("/posts", path)
        || under_prefix("/comments", path)
        || under_prefix("/users", path)
        || path == "/plugins"
        || path == "/authentications/types"
}

/// 复刻 Java `SecurityConfig` 的路径级鉴权（过滤器层），保证未认证/无权限的
/// 响应体为 `{code,message}`（不带 `data`），与 `writeJson` 完全一致。
async fn spring_security_gate(
    State(cfg): State<Arc<Config>>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    // 路由挂载在 `/api` 下，这里统一剥离前缀后按 Java 非前缀形式匹配。
    let p = path.strip_prefix("/api").unwrap_or(&path).to_string();

    // OPTIONS 预检放行（Java `OPTIONS /** permitAll`）。CorsLayer 通常已短路处理，这里兜底。
    if method == Method::OPTIONS {
        return next.run(req).await;
    }
    // GET /media/** permitAll（MinIO 媒体代理）。
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

    // `/admin/**` 需 ADMIN 角色（对齐全 `ADMIN_PATHS`）。
    if under_prefix("/admin", &p) {
        return if is_admin {
            next.run(req).await
        } else if authenticated {
            crate::error::ApiError::raw(403, "forbidden").into_response()
        } else {
            crate::error::ApiError::raw(401, "unauthorized").into_response()
        };
    }

    // 公共路径放行。
    if is_public_path(&p) {
        return next.run(req).await;
    }

    // 其余路径 `anyRequest().authenticated()`。
    if authenticated {
        next.run(req).await
    } else {
        crate::error::ApiError::raw(401, "unauthorized").into_response()
    }
}
