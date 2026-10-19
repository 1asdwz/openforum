use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

/// Mirrors Java `ApiException`: carries a business code, the HTTP status code is the code, and the body is `{code,message,data:null}`.
///
/// When `raw = true`, outputs `{code,message}` (without `data`), reproducing the 401/403 responses
/// produced by Java `SecurityConfig.writeJson` at the Spring Security filter layer.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub raw: bool,
    /// Business errors may carry structured data (e.g. unmet gate conditions).
    pub data: Option<Value>,
}

impl ApiError {
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), raw: false, data: None }
    }
    /// Filter-layer style: `{code,message}`, without a `data` field.
    pub fn raw(code: u16, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), raw: true, data: None }
    }
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
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
        ApiError::internal("Internal server error")
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!("internal error: {e}");
        ApiError::internal("Internal server error")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        // Business errors mirror Java `ApiResponse.error(code, message)` and include `data: null`;
        // filter-layer errors (raw) mirror Java `SecurityConfig.writeJson` and omit `data`.
        let body = if self.raw {
            json!({ "code": self.code, "message": self.message })
        } else {
            json!({
                "code": self.code,
                "message": self.message,
                "data": self.data.unwrap_or(Value::Null),
            })
        };
        (status, Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
