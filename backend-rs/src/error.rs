use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// 与 Java `ApiException` 对齐：携带业务 code，HTTP 状态码即 code，body 为 `{code,message,data:null}`。
///
/// `raw = true` 时输出 `{code,message}`（不带 `data`），用于复刻 Java `SecurityConfig.writeJson`
/// 在 Spring Security 过滤器层产生的 401/403 响应。
#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub raw: bool,
}

impl ApiError {
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), raw: false }
    }
    /// 过滤器层风格：`{code,message}`，不带 `data` 字段。
    pub fn raw(code: u16, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), raw: true }
    }
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, message)
    }
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(401, message)
    }
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(403, message)
    }
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, message)
    }
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, message)
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("database error: {e}");
        ApiError::internal("服务器内部错误")
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!("internal error: {e}");
        ApiError::internal("服务器内部错误")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        // 业务错误与 Java `ApiResponse.error(code, message)` 对齐，带 `data: null`；
        // 过滤器层错误（raw）与 Java `SecurityConfig.writeJson` 对齐，不带 `data`。
        let body = if self.raw {
            json!({ "code": self.code, "message": self.message })
        } else {
            json!({ "code": self.code, "message": self.message, "data": null })
        };
        (status, Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
