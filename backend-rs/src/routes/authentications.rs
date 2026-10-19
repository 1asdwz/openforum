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
    "Take the photo in a well-lit area with a plain background",
    "Keep your face unobstructed; do not wear a hat, sunglasses, or a mask",
    "Face the camera and stay still; follow the on-screen prompts to complete each action",
    "The document must show all four corners, clear text, and no glare or obstruction",
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
        "REAL_NAME" => "Real Name",
        "ID_CARD" => "ID Card",
        "GENDER" => "Gender",
        "AGE" => "Age",
        "NATIONALITY" => "Nationality",
        "CITY" => "City",
        "OCCUPATION" => "Occupation",
        "EDUCATION" => "Education",
        "DRIVER_LICENSE" => "Driver's License",
        "MARRIAGE_CERT" => "Marriage",
        "PROPERTY_CERT" => "Property",
        "ASSETS" => "Assets",
        "CUSTOM" => "Custom",
        _ => "Custom",
    }
}

fn document_name(t: &str) -> &'static str {
    match t {
        "REAL_NAME" => "Document (ID card/passport)",
        "ID_CARD" => "ID card",
        "DRIVER_LICENSE" => "Driver's license",
        "MARRIAGE_CERT" => "Marriage certificate",
        "PROPERTY_CERT" => "Property certificate",
        "NATIONALITY" => "Proof of nationality",
        "CITY" => "Proof of residence",
        "OCCUPATION" => "Employment certificate",
        "EDUCATION" => "Education certificate",
        _ => "Supporting documents",
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
            "label": "Low",
            "requirements": [format!("Fill in {label} information")],
            "tips": Vec::<String>::new(),
        }),
        json!({
            "level": "MEDIUM",
            "label": "Medium",
            "requirements": [
                format!("Fill in {label} information"),
                format!("Upload a clear photo of the front of your {doc}"),
                "AI reviews the authenticity of the materials".to_string(),
            ],
            "tips": tips,
        }),
        json!({
            "level": "HIGH",
            "label": "High",
            "requirements": [
                format!("Fill in {label} information"),
                "Complete video liveness verification (random action sequence + face analysis)".to_string(),
                format!("Upload a photo of your {doc}"),
                "Real-name authentication is registered automatically after AI approval".to_string(),
            ],
            "tips": tips,
        }),
        json!({
            "level": "EXTREME",
            "label": "Extreme",
            "requirements": [
                "Complete all steps of the High level".to_string(),
                "Record an anti-spoofing video with the OpenForum mobile app".to_string(),
                "Earn the highest trust badge after passing".to_string(),
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
        "REAL_NAME" => vec![f("fullName", "Full name", "text")],
        "ID_CARD" => vec![
            f("fullName", "Full name", "text"),
            f("idNumber", "ID number", "text"),
            sel(
                "gender",
                "Gender",
                json!([{"value": "MALE", "label": "Male"}, {"value": "FEMALE", "label": "Female"}]),
            ),
            f("birthDate", "Date of birth", "date"),
            sel(
                "country",
                "Country/Region",
                crate::info::options_value(crate::info::find_field("COUNTRY").expect("COUNTRY field")),
            ),
            f("city", "City", "text"),
        ],
        "GENDER" => vec![sel(
            "gender",
            "Gender",
            json!([{"value": "MALE", "label": "Male"}, {"value": "FEMALE", "label": "Female"}]),
        )],
        "AGE" => vec![f("birthDate", "Date of birth", "date")],
        "NATIONALITY" => vec![sel(
            "nationality",
            "Nationality",
            crate::info::options_value(crate::info::find_field("COUNTRY").expect("COUNTRY field")),
        )],
        "CITY" => vec![
            sel(
                "country",
                "Country/Region",
                crate::info::options_value(crate::info::find_field("COUNTRY").expect("COUNTRY field")),
            ),
            f("city", "City", "text"),
        ],
        "OCCUPATION" => vec![
            sel(
                "category",
                "Occupation category",
                crate::info::options_value(crate::info::find_field("OCCUPATION").expect("OCCUPATION field")),
            ),
            f("title", "Job title", "text"),
        ],
        "EDUCATION" => vec![sel(
            "education",
            "Education",
            crate::info::options_value(crate::info::find_field("EDUCATION").expect("EDUCATION field")),
        )],
        "DRIVER_LICENSE" => vec![f("licenseNumber", "License number", "text")],
        "MARRIAGE_CERT" => vec![
            f("spouseName", "Spouse name", "text"),
            f("certificateNumber", "Marriage certificate number", "text"),
        ],
        "PROPERTY_CERT" => vec![
            f("certificateNumber", "Property certificate number", "text"),
            f("propertyAddress", "Property address", "text"),
            f("cities", "Property city", "tags"),
            f("districts", "Property district", "tags"),
            f("count", "Number of properties", "number"),
        ],
        "ASSETS" => vec![f("amount", "Asset amount", "number")],
        "CUSTOM" => vec![f("description", "Description", "textarea")],
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
        _ => return Err(ApiError::bad_request("kind must not be empty")),
    };
    let up = v.to_uppercase();
    match up.as_str() {
        "DOCUMENT" | "SELFIE" | "VIDEO" => Ok(up),
        _ => Err(ApiError::bad_request(format!("Unknown file type: {v}"))),
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
        return Err(ApiError::bad_request("Uploaded file must not be empty"));
    }
    if !state.storage.is_enabled() {
        return Err(ApiError::bad_request("File storage is not enabled; cannot upload authentication materials"));
    }
    let lower = content_type.to_lowercase();
    let size = bytes.len() as i64;
    if kind == "VIDEO" {
        if !AUTH_VIDEO_TYPES.contains(&lower.as_str()) {
            return Err(ApiError::bad_request("Unsupported video format; only webm/mp4 are allowed"));
        }
        if size > state.cfg.auth_max_video_bytes {
            return Err(ApiError::bad_request("Video is too large; the limit is 50MB"));
        }
    } else {
        if !AUTH_IMAGE_TYPES.contains(&lower.as_str()) {
            return Err(ApiError::bad_request("Unsupported image format; only jpeg/png/webp are allowed"));
        }
        if size > state.cfg.auth_max_image_bytes {
            return Err(ApiError::bad_request("Image is too large; the limit is 10MB"));
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
        .ok_or_else(|| ApiError::not_found("Media file not found"))?;
    if media.user_id != me.id && !me.is_admin() {
        return Err(ApiError::forbidden("You do not have access to this media file"));
    }
    if !state.storage.is_enabled() {
        return Err(ApiError::bad_request("File storage is not enabled"));
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
        // The Java entity declares a default of LOW for the `level` field; historical records with NULL in the DB serialize as "LOW".
        "level": auth.level.clone().unwrap_or_else(|| "LOW".to_string()),
        "media": media,
        "liveness": if liveness.is_null() { json!({}) } else { liveness },
        "attestation": if attestation.is_null() { json!({}) } else { attestation },
    }))
}

fn parse_auth_type(raw: Option<&str>) -> Result<String, ApiError> {
    let v = match raw {
        Some(s) if !s.trim().is_empty() => s.trim().to_uppercase(),
        _ => return Err(ApiError::bad_request("type must not be empty")),
    };
    if crate::enums::is_known_auth_type(&v) {
        Ok(v)
    } else {
        Err(ApiError::bad_request(format!("Unknown authentication type: {v}")))
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
                "This authentication type already exists; use the retry endpoint to resubmit",
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
        .ok_or_else(|| ApiError::not_found("Authentication record not found"))?;
    if auth.user_id != me.id {
        return Err(ApiError::forbidden("You do not have permission to modify this authentication"));
    }
    let old_level = AuthLevel::parse_or_null(auth.level.as_deref()).unwrap_or(AuthLevel::Low);
    let new_level = match req.level.as_deref() {
        Some(s) if !s.trim().is_empty() => AuthLevel::parse(s)?,
        _ => old_level,
    };
    let rejected = auth.status == "REJECTED";
    let upgrade = new_level.rank() > old_level.rank();
    if !rejected && !upgrade {
        return Err(ApiError::bad_request("Only rejected authentications or level upgrades can be resubmitted"));
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
