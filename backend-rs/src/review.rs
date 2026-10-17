use base64::Engine as _;
use chrono::Local;
use serde_json::{json, Map, Value};

use crate::enums::AuthLevel;
use crate::error::ApiError;
use crate::json::json_to_map;
use crate::models::{AuthMedia, UserAuthentication};
use crate::repo;
use crate::state::AppState;

const AI_VISION_PROMPT: &str = "你是用户认证审核助手。我会给你用户的认证类型、声明的字段以及证件/自拍照片。\n请判断材料是否真实、清晰、与声明一致，并尽量抽取关键字段。\n只返回严格 JSON，不要任何多余文字，格式：\n{\"status\":\"APPROVED|REJECTED|PENDING\",\"confidence\":0.0-1.0,\"reason\":\"简短说明\",\n \"extracted\":{\"fullName\":\"\",\"idNumber\":\"\"}}\n- 图片与声明一致且清晰：APPROVED，confidence 0.85 以上。\n- 图片模糊、遮挡、明显伪造或与声明不符：REJECTED。\n- 无法判断：PENDING。";

const MIN_DIMENSION: u32 = 300;

#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub status: &'static str,
    pub confidence: f64,
    pub reason: String,
    pub extracted: Value,
}

impl ReviewResult {
    fn new(status: &'static str, confidence: f64, reason: impl Into<String>) -> Self {
        Self {
            status,
            confidence,
            reason: reason.into(),
            extracted: Value::Object(Map::new()),
        }
    }
}

pub struct MediaRef {
    pub kind: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

/// 主审核流水线，对应 Java `AuthenticationReviewService.review`。
/// 全流程（PENDING 落库 → 规则/插件审核 → 终态写回 → 审核日志）在单个事务内完成，
/// 对齐 Java 方法上的 `@Transactional`，避免中途失败留下半成品状态。
#[allow(clippy::too_many_arguments)]
pub async fn review(
    state: &AppState,
    user_id: i64,
    auth_type: &str,
    level: AuthLevel,
    data: Value,
    media_ids: Vec<i64>,
    video_media_id: Option<i64>,
    challenge_id: Option<String>,
    liveness: Option<Value>,
    attestation: Option<Value>,
    existing: Option<UserAuthentication>,
) -> Result<UserAuthentication, ApiError> {
    let mut tx = state.pool.begin().await?;
    let result = review_inner(
        state,
        &mut tx,
        user_id,
        auth_type,
        level,
        data,
        media_ids,
        video_media_id,
        challenge_id,
        liveness,
        attestation,
        existing,
    )
    .await;
    match result {
        Ok(saved) => {
            tx.commit().await?;
            Ok(saved)
        }
        // tx 在此 drop：未提交即自动回滚。
        Err(e) => Err(e),
    }
}

#[allow(clippy::too_many_arguments)]
async fn review_inner(
    state: &AppState,
    conn: &mut sqlx::MySqlConnection,
    user_id: i64,
    auth_type: &str,
    level: AuthLevel,
    data: Value,
    media_ids: Vec<i64>,
    video_media_id: Option<i64>,
    challenge_id: Option<String>,
    liveness: Option<Value>,
    attestation: Option<Value>,
    existing: Option<UserAuthentication>,
) -> Result<UserAuthentication, ApiError> {
    let safe_data: Map<String, Value> = match data {
        Value::Object(m) => m,
        _ => Map::new(),
    };
    let liveness_map: Map<String, Value> = match liveness {
        Some(Value::Object(m)) => m,
        _ => Map::new(),
    };
    let attestation_map: Map<String, Value> = match attestation {
        Some(Value::Object(m)) => m,
        _ => Map::new(),
    };

    // 1. 结构校验（其中活体挑战校验可能返回 400，需在落库前抛出）
    let structure_error = validate_structure(
        state,
        conn,
        user_id,
        auth_type,
        level,
        &safe_data,
        &media_ids,
        video_media_id,
        challenge_id.as_deref(),
        &liveness_map,
        &attestation_map,
    )
    .await?;

    // 2. 落库为 PENDING
    let mut auth = existing.unwrap_or_default();
    auth.user_id = user_id;
    auth.auth_type = auth_type.to_string();
    auth.level = Some(level.name().to_string());
    auth.auth_data = Some(Value::Object(safe_data.clone()).to_string());
    auth.media = if media_ids.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&media_ids).unwrap_or_default())
    };
    auth.liveness = if liveness_map.is_empty() {
        None
    } else {
        Some(Value::Object(liveness_map.clone()).to_string())
    };
    auth.attestation = if attestation_map.is_empty() {
        None
    } else {
        Some(Value::Object(attestation_map.clone()).to_string())
    };
    auth.reviewed_by = None;
    auth.rejection_reason = None;
    auth.status = "PENDING".to_string();
    repo::save_auth(&mut *conn, &mut auth).await?;

    repo::insert_review_log(
        &mut *conn,
        auth.id,
        auth.level.as_deref(),
        "SUBMIT",
        auth.review_method.as_deref(),
        None,
        Some(&format!("提交 {}:{}", auth.auth_type, auth.level.clone().unwrap_or_default())),
        Some(user_id),
    )
    .await?;

    if let Some(err) = structure_error {
        return finish(conn, auth, "REJECTED", "RULE", 0.95, &err, Some(&err)).await;
    }

    // 3. LOW：直接采用规则引擎结论
    if level == AuthLevel::Low {
        let rule = rule_review(auth_type, &safe_data);
        if rule.status == "REJECTED" {
            return finish(conn, auth, "REJECTED", "RULE", rule.confidence, &rule.reason, Some(&rule.reason)).await;
        }
        let saved = finish(conn, auth, "APPROVED", "RULE", rule.confidence, &rule.reason, None).await?;
        crate::info::grant_user_info(conn, user_id, auth_type, saved.id, level.name(), &safe_data).await?;
        return Ok(saved);
    }

    // 4. 绑定媒体（带 user_id 过滤，防止越权绑定他人材料）
    if !media_ids.is_empty() {
        repo::attach_auth_media(&mut *conn, auth.id, user_id, &media_ids).await?;
    }

    // 5. 专业审核
    let documents = materialize(state, conn, &media_ids, user_id).await?;
    if !documents.iter().any(|d| d.kind == "DOCUMENT") {
        return finish(
            conn,
            auth,
            "REJECTED",
            "RULE",
            0.9,
            "至少需要上传 1 张证件照片",
            Some("缺少证件照片"),
        )
        .await;
    }
    let outcome = run_plugins(state, auth_type, level, &safe_data, &documents).await;

    // 6. 活体降级：不自动通过，转人工
    let liveness_passed = liveness_map.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
    if level.rank() >= AuthLevel::High.rank() && !liveness_passed && outcome.status != "REJECTED" {
        return finish(
            conn,
            auth,
            "PENDING",
            "MANUAL",
            0.0,
            "活体检测未完成，转人工审核",
            None,
        )
        .await;
    }

    if outcome.status == "APPROVED" {
        let saved = finish(
            conn,
            auth,
            "APPROVED",
            outcome.method,
            outcome.confidence,
            &outcome.reason,
            None,
        )
        .await?;
        real_name_register(conn, user_id, auth_type, level, &safe_data, &outcome.extracted).await?;
        crate::info::grant_user_info(conn, user_id, auth_type, saved.id, level.name(), &safe_data).await?;
        return Ok(saved);
    }
    if outcome.status == "REJECTED" {
        return finish(
            conn,
            auth,
            "REJECTED",
            outcome.method,
            outcome.confidence,
            &outcome.reason,
            Some(&outcome.reason),
        )
        .await;
    }
    finish(conn, auth, "PENDING", "MANUAL", 0.0, &outcome.reason, None).await
}

struct PluginOutcome {
    status: &'static str,
    method: &'static str,
    confidence: f64,
    reason: String,
    extracted: Value,
}

#[allow(clippy::too_many_arguments)]
async fn validate_structure(
    state: &AppState,
    conn: &mut sqlx::MySqlConnection,
    user_id: i64,
    auth_type: &str,
    level: AuthLevel,
    data: &Map<String, Value>,
    media_ids: &[i64],
    video_media_id: Option<i64>,
    challenge_id: Option<&str>,
    liveness: &Map<String, Value>,
    attestation: &Map<String, Value>,
) -> Result<Option<String>, ApiError> {
    let rule = rule_review(auth_type, data);
    if rule.status == "REJECTED" {
        return Ok(Some(rule.reason));
    }
    if level.rank() < AuthLevel::Medium.rank() {
        return Ok(None);
    }
    if media_ids.is_empty() {
        return Ok(Some("该等级需要上传证件材料".to_string()));
    }
    if media_ids.len() > state.cfg.auth_max_document_count {
        return Ok(Some(format!(
            "证件材料数量超出上限（最多 {} 张）",
            state.cfg.auth_max_document_count
        )));
    }
    if level.rank() >= AuthLevel::High.rank() {
        if video_media_id.is_none() {
            return Ok(Some("该等级需要完成视频认证".to_string()));
        }
        consume_challenge(conn, challenge_id, user_id, liveness).await?;
    }
    if level == AuthLevel::Extreme && attestation.is_empty() {
        return Ok(Some("极高等级需要通过移动 App 完成防伪录制".to_string()));
    }
    Ok(None)
}

async fn consume_challenge(
    conn: &mut sqlx::MySqlConnection,
    challenge_id: Option<&str>,
    user_id: i64,
    liveness: &Map<String, Value>,
) -> Result<(), ApiError> {
    let challenge_id = match challenge_id {
        Some(c) if !c.trim().is_empty() => c,
        _ => return Err(ApiError::bad_request("缺少活体挑战标识")),
    };
    let challenge = repo::find_challenge(&mut *conn, challenge_id, user_id)
        .await?
        .ok_or_else(|| ApiError::bad_request("活体挑战无效或不属于当前用户"))?;
    if challenge.used_at.is_some() {
        return Err(ApiError::bad_request("活体挑战已使用，请重新开始视频认证"));
    }
    if challenge.expires_at < Local::now().naive_local() {
        return Err(ApiError::bad_request("活体挑战已过期，请重新开始视频认证"));
    }
    if liveness.is_empty() {
        return Err(ApiError::bad_request("缺少活体检测结果"));
    }
    let degraded = liveness.get("degraded").and_then(|v| v.as_bool()).unwrap_or(false);
    if liveness.get("passed").and_then(|v| v.as_bool()).unwrap_or(false) {
        let required = crate::json::json_to_string_list(challenge.actions.as_deref());
        validate_actions(&required, liveness)?;
    } else if !degraded {
        return Err(ApiError::bad_request("活体检测未通过，请重新进行视频认证"));
    }
    // 原子消费：仅当尚未被使用时才成功；并发下只有一个请求能抢到，避免重复消费（TOCTOU）。
    let consumed = repo::mark_challenge_used(&mut *conn, challenge_id, user_id).await?;
    if !consumed {
        return Err(ApiError::bad_request("活体挑战已使用，请重新开始视频认证"));
    }
    Ok(())
}

fn validate_actions(required: &[String], liveness: &Map<String, Value>) -> Result<(), ApiError> {
    let list = match liveness.get("actions") {
        Some(Value::Array(a)) => a,
        _ => return Err(ApiError::bad_request("活体检测结果缺少动作记录")),
    };
    for action in required {
        let passed = list.iter().any(|item| match item {
            Value::Object(obj) => {
                obj.get("action").and_then(|v| v.as_str()) == Some(action.as_str())
                    && obj.get("passed").and_then(|v| v.as_bool()).unwrap_or(false)
            }
            _ => false,
        });
        if !passed {
            return Err(ApiError::bad_request(format!("未完成指定动作: {action}")));
        }
    }
    Ok(())
}

async fn materialize(
    state: &AppState,
    conn: &mut sqlx::MySqlConnection,
    media_ids: &[i64],
    user_id: i64,
) -> Result<Vec<MediaRef>, ApiError> {
    if media_ids.is_empty() {
        return Ok(Vec::new());
    }
    let media: Vec<AuthMedia> = repo::list_auth_media_in(&mut *conn, user_id, media_ids).await?;
    if media.len() != media_ids.len() {
        return Err(ApiError::bad_request("存在无效或不属于当前用户的媒体文件"));
    }
    let mut refs = Vec::with_capacity(media.len());
    for item in media {
        let bytes = state.storage.get(&item.object_key).await?;
        refs.push(MediaRef {
            kind: item.kind.clone(),
            content_type: item.content_type.clone(),
            bytes,
        });
    }
    Ok(refs)
}

async fn run_plugins(
    state: &AppState,
    auth_type: &str,
    level: AuthLevel,
    data: &Map<String, Value>,
    documents: &[MediaRef],
) -> PluginOutcome {
    let plugins = state.plugins.enabled_document_reviews();
    if plugins.is_empty() {
        return PluginOutcome {
            status: "PENDING",
            method: "MANUAL",
            confidence: 0.0,
            reason: "等待人工审核".to_string(),
            extracted: Value::Object(Map::new()),
        };
    }

    let mut best_approved: Option<PluginOutcome> = None;
    let mut best_rejected: Option<PluginOutcome> = None;
    let mut extracted = Value::Object(Map::new());
    let threshold = state.cfg.auth_auto_approve_threshold;

    for plugin in plugins {
        let (method, result) = match plugin.key {
            "ai-vision-review" => ("AI", ai_vision_review(state, auth_type, level, data, documents).await),
            "heuristic-document" => ("RULE", heuristic_document(documents)),
            _ => continue,
        };
        if let Value::Object(m) = &result.extracted {
            if !m.is_empty() {
                extracted = result.extracted.clone();
            }
        }
        let outcome = PluginOutcome {
            status: result.status,
            method,
            confidence: result.confidence,
            reason: result.reason,
            extracted: result.extracted,
        };
        if outcome.status == "REJECTED" {
            let better = best_rejected.as_ref().map(|b| outcome.confidence > b.confidence).unwrap_or(true);
            if better {
                best_rejected = Some(outcome);
            }
        } else if outcome.status == "APPROVED" {
            let better = best_approved.as_ref().map(|b| outcome.confidence > b.confidence).unwrap_or(true);
            if better {
                best_approved = Some(outcome);
            }
        }
    }

    if let Some(rejected) = best_rejected {
        return rejected;
    }
    if let Some(approved) = &best_approved {
        if approved.confidence >= threshold {
            return best_approved.unwrap();
        }
    }
    PluginOutcome {
        status: "PENDING",
        method: "MANUAL",
        confidence: 0.0,
        reason: "材料已收到，等待人工复核".to_string(),
        extracted,
    }
}

async fn finish(
    conn: &mut sqlx::MySqlConnection,
    mut auth: UserAuthentication,
    status: &str,
    method: &str,
    confidence: f64,
    reason: &str,
    rejection_reason: Option<&str>,
) -> Result<UserAuthentication, ApiError> {
    auth.status = status.to_string();
    auth.review_method = Some(method.to_string());
    auth.confidence = Some(confidence);
    auth.reason = Some(reason.to_string());
    auth.rejection_reason = rejection_reason.map(|s| s.to_string());
    if status != "PENDING" {
        auth.reviewed_at = Some(Local::now().naive_local());
    }
    repo::save_auth(&mut *conn, &mut auth).await?;

    let action = match status {
        "APPROVED" => "APPROVE",
        "REJECTED" => "REJECT",
        _ => "SUBMIT",
    };
    repo::insert_review_log(
        &mut *conn,
        auth.id,
        auth.level.as_deref(),
        action,
        Some(method),
        Some(confidence),
        Some(reason),
        None,
    )
    .await?;
    Ok(auth)
}

/// 规则引擎：校验各认证类型的必填字段与格式。
pub fn rule_review(auth_type: &str, data: &Map<String, Value>) -> ReviewResult {
    let mut errors: Vec<String> = Vec::new();
    match auth_type {
        "REAL_NAME" => require_text(data, "fullName", "真实姓名", 2, 50, &mut errors),
        "ID_CARD" => validate_id_card(data, &mut errors),
        "GENDER" => validate_gender(data, &mut errors),
        "AGE" => validate_age(data, &mut errors),
        "NATIONALITY" => require_text(data, "nationality", "国籍", 1, 50, &mut errors),
        "CITY" => require_text(data, "city", "城市", 1, 40, &mut errors),
        "OCCUPATION" => {
            let category = text(data, "category").unwrap_or_default().to_ascii_uppercase();
            if !crate::info::is_valid_occupation(&category) {
                errors.push("请选择职业分类".to_string());
            }
            if text(data, "title").map(|t| t.chars().count() > 50).unwrap_or(false) {
                errors.push("职位名称不能超过 50 字".to_string());
            }
        }
        "EDUCATION" => {
            let raw = text(data, "education").unwrap_or_default().to_ascii_uppercase();
            if crate::info::education_rank(&raw).is_none() {
                errors.push("请选择学历".to_string());
            }
        }
        "DRIVER_LICENSE" => require_text(data, "licenseNumber", "驾照号", 6, 40, &mut errors),
        "MARRIAGE_CERT" => {
            require_text(data, "spouseName", "配偶姓名", 1, 50, &mut errors);
            require_text(data, "certificateNumber", "结婚证号", 4, 40, &mut errors);
        }
        "PROPERTY_CERT" => {
            require_text(data, "certificateNumber", "产权证号", 4, 60, &mut errors);
            require_text(data, "propertyAddress", "房产地址", 2, 200, &mut errors);
        }
        "ASSETS" => validate_assets(data, &mut errors),
        "CUSTOM" => require_text(data, "description", "说明", 2, 500, &mut errors),
        _ => {}
    }
    if errors.is_empty() {
        ReviewResult::new("APPROVED", 0.95, "规则校验通过")
    } else {
        ReviewResult::new("REJECTED", 0.95, errors.join("；"))
    }
}

fn validate_id_card(data: &Map<String, Value>, errors: &mut Vec<String>) {
    match text(data, "idNumber") {
        None => errors.push("身份证号不能为空".to_string()),
        Some(v) if v.is_empty() => errors.push("身份证号不能为空".to_string()),
        Some(v) => {
            let bytes = v.as_bytes();
            let valid_len = v.chars().count() == 18;
            let first17 = bytes.len() >= 17 && bytes[..17].iter().all(|b| b.is_ascii_digit());
            let last_ok = bytes
                .get(17)
                .map(|b| b.is_ascii_digit() || *b == b'X' || *b == b'x')
                .unwrap_or(false);
            if !(valid_len && first17 && last_ok) {
                errors.push("身份证号必须为 18 位（末位可为 X）".to_string());
            }
        }
    }
}

fn validate_gender(data: &Map<String, Value>, errors: &mut Vec<String>) {
    const GENDERS: [&str; 6] = ["male", "female", "other", "男", "女", "其他"];
    match text(data, "gender") {
        None => errors.push("性别不能为空".to_string()),
        Some(v) if v.is_empty() => errors.push("性别不能为空".to_string()),
        Some(v) => {
            if !GENDERS.contains(&v.to_lowercase().as_str()) {
                errors.push("性别取值不合法".to_string());
            }
        }
    }
}

fn validate_age(data: &Map<String, Value>, errors: &mut Vec<String>) {
    if let Some(birth) = text(data, "birthDate") {
        if !birth.is_empty() {
            if chrono::NaiveDate::parse_from_str(&birth, "%Y-%m-%d").is_err() {
                errors.push("出生日期格式应为 YYYY-MM-DD".to_string());
            }
            return;
        }
    }
    match text(data, "age") {
        None => errors.push("年龄不能为空".to_string()),
        Some(v) if v.is_empty() => errors.push("年龄不能为空".to_string()),
        Some(v) => match v.parse::<i32>() {
            Ok(age) if (0..=120).contains(&age) => {}
            Ok(_) => errors.push("年龄需在 0-120 之间".to_string()),
            Err(_) => errors.push("年龄必须为数字".to_string()),
        },
    }
}

fn validate_assets(data: &Map<String, Value>, errors: &mut Vec<String>) {
    match text(data, "amount") {
        None => errors.push("资产金额不能为空".to_string()),
        Some(v) if v.is_empty() => errors.push("资产金额不能为空".to_string()),
        Some(v) => match v.parse::<f64>() {
            Ok(amount) if amount >= 0.0 => {}
            Ok(_) => errors.push("资产金额不能为负数".to_string()),
            Err(_) => errors.push("资产金额必须为数字".to_string()),
        },
    }
}

fn require_text(
    data: &Map<String, Value>,
    key: &str,
    label: &str,
    min: usize,
    max: usize,
    errors: &mut Vec<String>,
) {
    match text(data, key) {
        None => errors.push(format!("{label}不能为空")),
        Some(v) if v.is_empty() => errors.push(format!("{label}不能为空")),
        Some(v) => {
            let len = v.chars().count();
            if len < min || len > max {
                errors.push(format!("{label}长度需在 {min}-{max} 之间"));
            }
        }
    }
}

fn text(data: &Map<String, Value>, key: &str) -> Option<String> {
    match data.get(key) {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.trim().to_string()),
        Some(other) => Some(other.to_string().trim().to_string()),
    }
}

/// 启发式证件审核：只能判 REJECTED 或 PENDING。
pub fn heuristic_document(documents: &[MediaRef]) -> ReviewResult {
    let mut problems: Vec<String> = Vec::new();
    if documents.is_empty() {
        problems.push("缺少证件图片".to_string());
    }
    for document in documents {
        if document.bytes.is_empty() {
            problems.push("证件图片内容为空".to_string());
            continue;
        }
        match image::ImageReader::new(std::io::Cursor::new(&document.bytes))
            .with_guessed_format()
            .ok()
            .and_then(|r| r.into_dimensions().ok())
        {
            None => problems.push("证件图片无法解码".to_string()),
            Some((w, h)) => {
                if w < MIN_DIMENSION || h < MIN_DIMENSION {
                    problems.push(format!("证件图片分辨率过低（需 ≥ {MIN_DIMENSION}px）"));
                }
            }
        }
    }
    if problems.is_empty() {
        ReviewResult::new("PENDING", 0.3, "证件材料结构完整，转人工/其他插件复核")
    } else {
        ReviewResult::new("REJECTED", 0.6, problems.join("；"))
    }
}

async fn ai_vision_review(
    state: &AppState,
    auth_type: &str,
    level: AuthLevel,
    data: &Map<String, Value>,
    documents: &[MediaRef],
) -> ReviewResult {
    if !state.cfg.ai_configured() {
        return ReviewResult::new("PENDING", 0.0, "AI 未配置，跳过");
    }
    let mut content: Vec<Value> = Vec::new();
    content.push(json!({
        "type": "text",
        "text": format!("认证类型: {}\n认证等级: {}\n声明字段: {}", auth_type, level.name(), Value::Object(data.clone()))
    }));
    for (index, document) in documents.iter().enumerate() {
        content.push(json!({
            "type": "text",
            "text": format!("证件图片 #{}（{}）", index + 1, document.content_type.clone().unwrap_or_default())
        }));
        let mime = document
            .content_type
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "image/jpeg".to_string());
        let data_url = format!(
            "data:{};base64,{}",
            mime,
            base64::engine::general_purpose::STANDARD.encode(&document.bytes)
        );
        content.push(json!({ "type": "image_url", "image_url": { "url": data_url } }));
    }
    let body = json!({
        "model": state.cfg.ai_model,
        "temperature": 0,
        "messages": [
            { "role": "system", "content": AI_VISION_PROMPT },
            { "role": "user", "content": content }
        ]
    });
    match post_chat(state, &body).await {
        Ok(Some(raw)) => parse_ai_review(&raw, "AI 视觉审核"),
        Ok(None) => ReviewResult::new("PENDING", 0.0, "AI 返回内容为空"),
        Err(e) => {
            tracing::warn!("AI vision review failed: {e}");
            ReviewResult::new("PENDING", 0.0, "AI 视觉审核不可用")
        }
    }
}

async fn post_chat(state: &AppState, body: &Value) -> anyhow::Result<Option<String>> {
    let url = format!("{}/chat/completions", state.cfg.ai_api_base.trim_end_matches('/'));
    let response = state
        .http
        .post(url)
        .header("Authorization", format!("Bearer {}", state.cfg.ai_api_key))
        .json(body)
        .send()
        .await?;
    let value: Value = response.json().await?;
    let content = value
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"));
    Ok(match content {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Null) | None => None,
        Some(other) => Some(other.to_string()),
    })
}

fn parse_ai_review(raw: &str, default_reason: &str) -> ReviewResult {
    let cleaned = strip_code_fence(raw);
    let parsed: Value = serde_json::from_str(&cleaned).unwrap_or(Value::Null);
    let status = match parsed
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("PENDING")
        .to_uppercase()
        .as_str()
    {
        "APPROVED" => "APPROVED",
        "REJECTED" => "REJECTED",
        _ => "PENDING",
    };
    let confidence = parsed.get("confidence").and_then(value_to_f64).unwrap_or(0.0);
    let reason = parsed
        .get("reason")
        .map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string()))
        .unwrap_or_else(|| default_reason.to_string());
    let extracted = parsed
        .get("extracted")
        .filter(|v| v.is_object())
        .cloned()
        .unwrap_or(Value::Object(Map::new()));
    ReviewResult {
        status,
        confidence,
        reason,
        extracted,
    }
}

fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn strip_code_fence(content: &str) -> String {
    let trimmed = content.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        let rest = match rest.find('\n') {
            Some(idx) => &rest[idx + 1..],
            None => rest,
        };
        if let Some(last) = rest.rfind("```") {
            return rest[..last].trim().to_string();
        }
        return rest.trim().to_string();
    }
    trimmed.to_string()
}

const REAL_NAME_TRIGGERS: [&str; 5] = [
    "ID_CARD",
    "DRIVER_LICENSE",
    "MARRIAGE_CERT",
    "PROPERTY_CERT",
    "NATIONALITY",
];

async fn real_name_register(
    conn: &mut sqlx::MySqlConnection,
    user_id: i64,
    source_type: &str,
    level: AuthLevel,
    data: &Map<String, Value>,
    extracted: &Value,
) -> Result<(), ApiError> {
    if !REAL_NAME_TRIGGERS.contains(&source_type) || level.rank() < AuthLevel::High.rank() {
        return Ok(());
    }

    let full_name = first_non_blank(&[
        extracted.get("fullName").and_then(as_string),
        data.get("fullName").and_then(as_string),
        extracted.get("name").and_then(as_string),
        data.get("name").and_then(as_string),
    ]);
    let full_name = match full_name {
        Some(v) => v,
        None => {
            tracing::debug!("自动实名登记跳过：源认证 {source_type} 未提供 fullName");
            return Ok(());
        }
    };

    let existing = repo::find_auth_by_user_and_type(&mut *conn, user_id, "REAL_NAME").await?;
    if let Some(ref record) = existing {
        let lvl = AuthLevel::parse_or_null(record.level.as_deref()).unwrap_or(AuthLevel::Low);
        if record.status == "APPROVED" && lvl.rank() >= AuthLevel::High.rank() {
            return Ok(());
        }
    }

    let mut target = existing.unwrap_or_default();
    target.user_id = user_id;
    target.auth_type = "REAL_NAME".to_string();
    let mut real_name_data = json_to_map(target.auth_data.as_deref());
    real_name_data.insert("fullName".to_string(), Value::String(full_name));
    if let Some(id_number) = first_non_blank(&[
        extracted.get("idNumber").and_then(as_string),
        data.get("idNumber").and_then(as_string),
    ]) {
        real_name_data.insert("idNumber".to_string(), Value::String(id_number));
    }
    target.auth_data = Some(Value::Object(real_name_data).to_string());
    target.level = Some(AuthLevel::High.name().to_string());
    target.status = "APPROVED".to_string();
    target.review_method = Some("SYSTEM".to_string());
    target.confidence = Some(1.0);
    target.reason = Some(format!("由 {}:{} 认证自动登记", source_type, level.name()));
    target.rejection_reason = None;
    target.reviewed_at = Some(Local::now().naive_local());
    repo::save_auth(&mut *conn, &mut target).await?;

    repo::insert_review_log(
        &mut *conn,
        target.id,
        Some(AuthLevel::High.name()),
        "APPROVE",
        Some("SYSTEM"),
        Some(1.0),
        Some(&format!("由 {}:{} 认证自动登记实名", source_type, level.name())),
        None,
    )
    .await?;
    Ok(())
}

fn as_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        other => {
            let text = other.to_string();
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
    }
}

fn first_non_blank(values: &[Option<String>]) -> Option<String> {
    values.iter().flatten().find(|v| !v.is_empty()).cloned()
}
