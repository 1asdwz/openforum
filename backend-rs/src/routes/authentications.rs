use axum::body::Bytes;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::enums::AuthLevel;
use crate::error::{ApiError, ApiResult};
use crate::json as j;
use crate::models::UserAuthentication;
use crate::repo;
use crate::review;
use crate::routes::media as media_route;
use crate::security::CurrentUser;
use crate::state::AppState;

const AUTH_IMAGE_TYPES: [&str; 3] = ["image/jpeg", "image/png", "image/webp"];
const AUTH_VIDEO_TYPES: [&str; 2] = ["video/webm", "video/mp4"];
const TIPS: [&str; 4] = [
    "请在光线充足、背景简洁的环境下拍摄",
    "面部保持无遮挡，请勿戴帽子、墨镜或口罩",
    "正对镜头并保持稳定，按屏幕提示依次完成动作",
    "证件需四角完整、文字清晰、无反光或遮挡",
];

#[derive(Debug, Deserialize)]
pub struct KindQuery {
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitRequest {
    #[serde(rename = "type")]
    pub auth_type: Option<String>,
    pub level: Option<String>,
    pub data: Option<Value>,
    #[serde(rename = "mediaIds")]
    pub media_ids: Option<Vec<i64>>,
    #[serde(rename = "videoMediaId")]
    pub video_media_id: Option<i64>,
    #[serde(rename = "challengeId")]
    pub challenge_id: Option<String>,
    pub liveness: Option<Value>,
    pub attestation: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct RetryRequest {
    pub level: Option<String>,
    pub data: Option<Value>,
    #[serde(rename = "mediaIds")]
    pub media_ids: Option<Vec<i64>>,
    #[serde(rename = "videoMediaId")]
    pub video_media_id: Option<i64>,
    #[serde(rename = "challengeId")]
    pub challenge_id: Option<String>,
    pub liveness: Option<Value>,
    pub attestation: Option<Value>,
}

fn short_label(t: &str) -> &'static str {
    match t {
        "REAL_NAME" => "实名",
        "ID_CARD" => "身份证",
        "GENDER" => "性别",
        "AGE" => "年龄",
        "NATIONALITY" => "国籍",
        "CITY" => "城市",
        "OCCUPATION" => "职业",
        "EDUCATION" => "学历",
        "DRIVER_LICENSE" => "驾照",
        "MARRIAGE_CERT" => "婚姻",
        "PROPERTY_CERT" => "房产",
        "ASSETS" => "资产",
        "CUSTOM" => "自定义",
        _ => "自定义",
    }
}

fn document_name(t: &str) -> &'static str {
    match t {
        "REAL_NAME" => "证件（身份证/护照）",
        "ID_CARD" => "身份证",
        "DRIVER_LICENSE" => "驾驶证",
        "MARRIAGE_CERT" => "结婚证",
        "PROPERTY_CERT" => "房产证",
        "NATIONALITY" => "国籍证明",
        "CITY" => "居住证明",
        "OCCUPATION" => "在职证明",
        "EDUCATION" => "学历证书",
        _ => "相关证明材料",
    }
}

fn type_label(t: &str) -> String {
    crate::enums::auth_type_label(t).unwrap_or(t).to_string()
}

fn levels_for(t: &str) -> Vec<Value> {
    let label = short_label(t);
    let doc = document_name(t);
    let tips: Vec<&str> = TIPS.to_vec();
    vec![
        json!({
            "level": "LOW",
            "label": "低",
            "requirements": [format!("填写{label}相关信息")],
            "tips": Vec::<String>::new(),
        }),
        json!({
            "level": "MEDIUM",
            "label": "中",
            "requirements": [
                format!("填写{label}相关信息"),
                format!("上传{doc}照片（正面清晰）"),
                "由 AI 审核材料真实性".to_string(),
            ],
            "tips": tips,
        }),
        json!({
            "level": "HIGH",
            "label": "高",
            "requirements": [
                format!("填写{label}相关信息"),
                "完成视频活体认证（随机动作序列 + 人脸分析）".to_string(),
                format!("上传{doc}照片"),
                "AI 审核通过后自动登记实名认证".to_string(),
            ],
            "tips": tips,
        }),
        json!({
            "level": "EXTREME",
            "label": "极高",
            "requirements": [
                "完成「高」等级的全部步骤".to_string(),
                "使用 OpenForum 移动 App 防伪录制视频".to_string(),
                "通过后可获得最高可信度标识".to_string(),
            ],
            "tips": tips,
        }),
    ]
}

fn fields_for(t: &str) -> Vec<Value> {
    let f = |name: &str, label: &str, ty: &str| json!({ "name": name, "label": label, "type": ty, "required": true });
    let sel = |name: &str, label: &str, options: Value| json!({
        "name": name,
        "label": label,
        "type": "select",
        "required": true,
        "options": options,
    });
    match t {
        "REAL_NAME" => vec![f("fullName", "真实姓名", "text")],
        "ID_CARD" => vec![
            f("fullName", "真实姓名", "text"),
            f("idNumber", "身份证号", "text"),
            sel(
                "gender",
                "性别",
                json!([{"value": "MALE", "label": "男"}, {"value": "FEMALE", "label": "女"}]),
            ),
            f("birthDate", "出生日期", "date"),
            sel(
                "country",
                "国家/地区",
                crate::info::options_value(crate::info::find_field("COUNTRY").expect("COUNTRY field")),
            ),
            f("city", "城市", "text"),
        ],
        "GENDER" => vec![sel(
            "gender",
            "性别",
            json!([{"value": "MALE", "label": "男"}, {"value": "FEMALE", "label": "女"}]),
        )],
        "AGE" => vec![f("birthDate", "出生日期", "date")],
        "NATIONALITY" => vec![sel(
            "nationality",
            "国籍",
            crate::info::options_value(crate::info::find_field("COUNTRY").expect("COUNTRY field")),
        )],
        "CITY" => vec![
            sel(
                "country",
                "国家/地区",
                crate::info::options_value(crate::info::find_field("COUNTRY").expect("COUNTRY field")),
            ),
            f("city", "城市", "text"),
        ],
        "OCCUPATION" => vec![
            sel(
                "category",
                "职业分类",
                crate::info::options_value(crate::info::find_field("OCCUPATION").expect("OCCUPATION field")),
            ),
            f("title", "职位名称", "text"),
        ],
        "EDUCATION" => vec![sel(
            "education",
            "学历",
            crate::info::options_value(crate::info::find_field("EDUCATION").expect("EDUCATION field")),
        )],
        "DRIVER_LICENSE" => vec![f("licenseNumber", "驾照号", "text")],
        "MARRIAGE_CERT" => vec![
            f("spouseName", "配偶姓名", "text"),
            f("certificateNumber", "结婚证号", "text"),
        ],
        "PROPERTY_CERT" => vec![
            f("certificateNumber", "产权证号", "text"),
            f("propertyAddress", "房产地址", "text"),
            f("cities", "房产所在城市", "tags"),
            f("districts", "房产所在区县", "tags"),
            f("count", "房产数量", "number"),
        ],
        "ASSETS" => vec![f("amount", "资产金额", "number")],
        "CUSTOM" => vec![f("description", "说明", "textarea")],
        _ => Vec::new(),
    }
}

/// GET /authentications/types
pub async fn types() -> ApiResult<Json<Value>> {
    let items: Vec<Value> = crate::enums::AUTH_TYPES
        .iter()
        .map(|(t, label)| {
            json!({
                "type": t,
                "label": label,
                "fields": fields_for(t),
                "levels": levels_for(t),
            })
        })
        .collect();
    Ok(j::ok(items))
}

/// GET /authentications/challenge
pub async fn challenge(
    State(state): State<AppState>,
    user: CurrentUser,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    use rand::seq::SliceRandom;
    use rand::RngCore;

    let mut pool: Vec<String> = crate::enums::CHALLENGE_ACTIONS
        .iter()
        .map(|s| s.to_string())
        .collect();
    pool.shuffle(&mut rand::thread_rng());
    let actions: Vec<String> = pool.into_iter().take(3).collect();
    let actions_json = serde_json::to_string(&actions).unwrap_or_default();

    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    let nonce = hex::encode(buf);

    let id = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Local::now().naive_local()
        + chrono::Duration::minutes(state.cfg.auth_challenge_ttl_minutes);
    repo::insert_challenge(&state.pool, &id, me.id, &actions_json, &nonce, expires_at).await?;

    Ok(j::ok(json!({
        "challengeId": id,
        "actions": actions,
        "nonce": nonce,
        "expiresAt": expires_at,
    })))
}

fn parse_kind(raw: Option<&str>) -> Result<String, ApiError> {
    let v = match raw {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => return Err(ApiError::bad_request("文件类型 kind 不能为空")),
    };
    let up = v.to_uppercase();
    match up.as_str() {
        "DOCUMENT" | "SELFIE" | "VIDEO" => Ok(up),
        _ => Err(ApiError::bad_request(format!("未知的文件类型: {v}"))),
    }
}

/// POST /authentications/media
pub async fn upload_media(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(q): Query<KindQuery>,
    mut multipart: Multipart,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let kind = parse_kind(q.kind.as_deref())?;
    let (bytes, content_type) = media_route::read_file_field(&mut multipart).await?;
    if bytes.is_empty() {
        return Err(ApiError::bad_request("上传文件不能为空"));
    }
    if !state.storage.is_enabled() {
        return Err(ApiError::bad_request("文件存储未启用，无法上传认证材料"));
    }
    let lower = content_type.to_lowercase();
    let size = bytes.len() as i64;
    if kind == "VIDEO" {
        if !AUTH_VIDEO_TYPES.contains(&lower.as_str()) {
            return Err(ApiError::bad_request("视频格式不支持，仅允许 webm/mp4"));
        }
        if size > state.cfg.auth_max_video_bytes {
            return Err(ApiError::bad_request("视频过大，上限 50MB"));
        }
    } else {
        if !AUTH_IMAGE_TYPES.contains(&lower.as_str()) {
            return Err(ApiError::bad_request("图片格式不支持，仅允许 jpeg/png/webp"));
        }
        if size > state.cfg.auth_max_image_bytes {
            return Err(ApiError::bad_request("图片过大，上限 10MB"));
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256 = hex::encode(hasher.finalize());

    let ext = media_route::extension_for(&lower);
    let date = chrono::Local::now().format("%Y%m%d");
    let key = format!("auth/{}/{}/{}.{}", me.id, date, uuid::Uuid::new_v4(), ext);
    state.storage.put(&key, bytes, &lower).await?;

    let id = repo::insert_auth_media(&state.pool, me.id, &kind, &key, &lower, size, &sha256).await?;
    Ok(j::ok(json!({
        "mediaId": id,
        "kind": kind,
        "contentType": lower,
        "sizeBytes": size,
    })))
}

/// GET /authentications/media/{mediaId}
pub async fn get_media(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(media_id): Path<i64>,
) -> ApiResult<Response> {
    let me = user.require()?;
    let media = repo::find_auth_media(&state.pool, media_id)
        .await?
        .ok_or_else(|| ApiError::not_found("媒体文件不存在"))?;
    if media.user_id != me.id && !me.is_admin() {
        return Err(ApiError::forbidden("无权访问该媒体文件"));
    }
    if !state.storage.is_enabled() {
        return Err(ApiError::bad_request("文件存储未启用"));
    }
    let bytes = state.storage.get(&media.object_key).await?;
    let ct = media
        .content_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let len = media.size_bytes.unwrap_or(bytes.len() as i64);

    let mut resp = Bytes::from(bytes).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&ct).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    resp.headers_mut().insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("inline"));
    if let Ok(v) = HeaderValue::from_str(&len.to_string()) {
        resp.headers_mut().insert(header::CONTENT_LENGTH, v);
    }
    Ok(resp)
}

pub async fn auth_dto(state: &AppState, auth: &UserAuthentication) -> ApiResult<Value> {
    let auth_data = j::json_value(auth.auth_data.as_deref());
    let liveness = j::json_value(auth.liveness.as_deref());
    let attestation = j::json_value(auth.attestation.as_deref());
    let media_ids = j::json_to_long_list(auth.media.as_deref());
    let mut media = Vec::new();
    for id in media_ids {
        if let Some(m) = repo::find_auth_media(&state.pool, id).await? {
            media.push(json!({
                "id": m.id,
                "kind": m.kind,
                "contentType": m.content_type,
                "sizeBytes": m.size_bytes,
            }));
        }
    }
    Ok(json!({
        "id": auth.id,
        "userId": auth.user_id,
        "type": auth.auth_type,
        "typeLabel": type_label(&auth.auth_type),
        "status": auth.status,
        "reviewMethod": auth.review_method,
        "confidence": auth.confidence,
        "reason": auth.reason,
        "rejectionReason": auth.rejection_reason,
        "authData": auth_data,
        "createdAt": j::jts(auth.created_at),
        "reviewedAt": j::jts_opt(auth.reviewed_at),
        // Java 实体 `level` 字段声明默认值为 LOW，DB 中为 NULL 的历史记录序列化后即为 "LOW"。
        "level": auth.level.clone().unwrap_or_else(|| "LOW".to_string()),
        "media": media,
        "liveness": if liveness.is_null() { json!({}) } else { liveness },
        "attestation": if attestation.is_null() { json!({}) } else { attestation },
    }))
}

fn parse_auth_type(raw: Option<&str>) -> Result<String, ApiError> {
    let v = match raw {
        Some(s) if !s.trim().is_empty() => s.trim().to_uppercase(),
        _ => return Err(ApiError::bad_request("type type 不能为空")),
    };
    if crate::enums::is_known_auth_type(&v) {
        Ok(v)
    } else {
        Err(ApiError::bad_request(format!("未知认证类型: {v}")))
    }
}

fn parse_level_opt(raw: Option<&str>) -> Result<AuthLevel, ApiError> {
    match raw {
        Some(s) if !s.trim().is_empty() => AuthLevel::parse(s),
        _ => Ok(AuthLevel::Low),
    }
}

fn map_of(v: Option<Value>) -> Value {
    match v {
        Some(val @ Value::Object(_)) => val,
        _ => Value::Object(Map::new()),
    }
}

/// POST /authentications
pub async fn submit(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<SubmitRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let auth_type = parse_auth_type(req.auth_type.as_deref())?;
    let level = parse_level_opt(req.level.as_deref())?;
    let existing = repo::find_auth_by_user_and_type(&state.pool, me.id, &auth_type).await?;
    if let Some(ref e) = existing {
        let old = AuthLevel::parse_or_null(e.level.as_deref()).unwrap_or(AuthLevel::Low);
        let can_upgrade = e.status == "REJECTED" || old.rank() < level.rank();
        if !can_upgrade {
            return Err(ApiError::bad_request(
                "该类型认证已存在，如需重新提交请使用重试接口",
            ));
        }
    }

    let saved = review::review(
        &state,
        me.id,
        &auth_type,
        level,
        map_of(req.data),
        req.media_ids.unwrap_or_default(),
        req.video_media_id,
        req.challenge_id,
        req.liveness,
        req.attestation,
        existing,
    )
    .await?;
    Ok(j::ok(auth_dto(&state, &saved).await?))
}

/// GET /authentications/me
pub async fn my(
    State(state): State<AppState>,
    user: CurrentUser,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let list = repo::list_auth_by_user(&state.pool, me.id).await?;
    let mut content = Vec::with_capacity(list.len());
    for auth in &list {
        content.push(auth_dto(&state, auth).await?);
    }
    Ok(j::ok(content))
}

/// GET /authentications/me/approved
pub async fn approved(
    State(state): State<AppState>,
    user: CurrentUser,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let names = repo::approved_type_names(&state.pool, me.id).await?;
    Ok(j::ok(names))
}

/// POST /authentications/{id}/retry
pub async fn retry(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<RetryRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let auth = repo::find_auth_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::not_found("认证记录不存在"))?;
    if auth.user_id != me.id {
        return Err(ApiError::forbidden("无权操作该认证"));
    }
    let old_level = AuthLevel::parse_or_null(auth.level.as_deref()).unwrap_or(AuthLevel::Low);
    let new_level = match req.level.as_deref() {
        Some(s) if !s.trim().is_empty() => AuthLevel::parse(s)?,
        _ => old_level,
    };
    let rejected = auth.status == "REJECTED";
    let upgrade = new_level.rank() > old_level.rank();
    if !rejected && !upgrade {
        return Err(ApiError::bad_request("仅被拒绝的认证或提升等级时可以重新提交"));
    }
    let data = match req.data {
        Some(v @ Value::Object(_)) => v,
        _ => j::json_value(auth.auth_data.as_deref()),
    };
    let media_ids = req
        .media_ids
        .unwrap_or_else(|| j::json_to_long_list(auth.media.as_deref()));

    let saved = review::review(
        &state,
        me.id,
        &auth.auth_type.clone(),
        new_level,
        data,
        media_ids,
        req.video_media_id,
        req.challenge_id,
        req.liveness,
        req.attestation,
        Some(auth),
    )
    .await?;
    Ok(j::ok(auth_dto(&state, &saved).await?))
}
