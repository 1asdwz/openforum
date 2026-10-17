use axum::body::Bytes;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::json as j;
use crate::security::CurrentUser;
use crate::state::AppState;

const IMAGE_TYPES: [&str; 4] = ["image/jpeg", "image/png", "image/webp", "image/gif"];
const VIDEO_TYPES: [&str; 2] = ["video/mp4", "video/webm"];

pub fn extension_for(content_type: &str) -> &'static str {
    match content_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        _ => "bin",
    }
}

/// 读取 multipart 中的 `file` 字段，返回 (bytes, content_type)。
pub async fn read_file_field(multipart: &mut Multipart) -> Result<(Vec<u8>, String), ApiError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad_request("读取上传文件失败"))?
    {
        if field.name() == Some("file") {
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let bytes = field
                .bytes()
                .await
                .map_err(|_| ApiError::bad_request("读取上传文件失败"))?;
            return Ok((bytes.to_vec(), content_type));
        }
    }
    Err(ApiError::bad_request("上传文件不能为空"))
}

/// POST /media
pub async fn upload(
    State(state): State<AppState>,
    user: CurrentUser,
    mut multipart: Multipart,
) -> ApiResult<axum::Json<Value>> {
    let me = user.require()?;
    let (bytes, content_type) = read_file_field(&mut multipart).await?;
    if bytes.is_empty() {
        return Err(ApiError::bad_request("上传文件不能为空"));
    }
    if !state.storage.is_enabled() {
        return Err(ApiError::bad_request("文件存储未启用，无法上传媒体"));
    }
    let lower = content_type.to_lowercase();
    let is_video = VIDEO_TYPES.contains(&lower.as_str());
    let is_image = IMAGE_TYPES.contains(&lower.as_str());
    if !is_video && !is_image {
        return Err(ApiError::bad_request(
            "文件格式不支持，图片仅允许 jpeg/png/webp/gif，视频仅允许 mp4/webm",
        ));
    }
    let size = bytes.len() as i64;
    if is_video && size > state.cfg.auth_max_video_bytes {
        return Err(ApiError::bad_request("视频过大，上限 50MB"));
    }
    if is_image && size > state.cfg.auth_max_image_bytes {
        return Err(ApiError::bad_request("图片过大，上限 10MB"));
    }

    let ext = extension_for(&lower);
    let date = chrono::Local::now().format("%Y%m%d");
    let key = format!("post/{}/{}/{}.{}", me.id, date, uuid::Uuid::new_v4(), ext);
    state.storage.put(&key, bytes.clone(), &lower).await?;

    let result = sqlx::query(
        "INSERT INTO post_media (user_id, object_key, content_type, size_bytes, created_at) VALUES (?, ?, ?, ?, NOW())",
    )
    .bind(me.id)
    .bind(&key)
    .bind(&lower)
    .bind(size)
    .execute(&state.pool)
    .await?;
    let id = result.last_insert_id() as i64;

    Ok(j::ok(json!({
        "id": id,
        "contentType": lower,
        "sizeBytes": size,
    })))
}

/// GET /media/{id}
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Response> {
    let row: Option<(String, Option<String>, Option<i64>)> =
        sqlx::query_as("SELECT object_key, content_type, size_bytes FROM post_media WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
    let (object_key, content_type, size) =
        row.ok_or_else(|| ApiError::not_found("媒体文件不存在"))?;
    if !state.storage.is_enabled() {
        return Err(ApiError::bad_request("文件存储未启用"));
    }
    let bytes = state.storage.get(&object_key).await?;
    let ct = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let len = size.unwrap_or(bytes.len() as i64);

    let mut resp = Bytes::from(bytes).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&ct).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    if let Ok(v) = HeaderValue::from_str(&len.to_string()) {
        resp.headers_mut().insert(header::CONTENT_LENGTH, v);
    }
    Ok(resp)
}
