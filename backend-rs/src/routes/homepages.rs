use axum::extract::{Path, State};
use axum::Json;
use chrono::{Duration, Local};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::error::{ApiError, ApiResult};
use crate::json as j;
use crate::models::UserHomepage;
use crate::repo;
use crate::sanitizer;
use crate::security::CurrentUser;
use crate::state::AppState;

/// 原始 HTML 上限 64KB。
pub const MAX_HOMEPAGE_BYTES: usize = 65_536;
/// 净化后（含实体转义膨胀）上限 128KB。
pub const MAX_SANITIZED_BYTES: usize = 131_072;
/// 保存冷却（秒）。
const SAVE_COOLDOWN_SECONDS: i64 = 3;
/// 举报原因长度区间（字符数）。
const REPORT_MIN_CHARS: usize = 5;
const REPORT_MAX_CHARS: usize = 500;

pub async fn find_homepage(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Option<UserHomepage>, ApiError> {
    let row = sqlx::query_as::<_, UserHomepage>(
        "SELECT mode, html, status, block_reason, size_bytes, updated_at \
         FROM user_homepages WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// 公开视角：被下架内容对非本人/非管理员完全隐藏。
fn public_view(row: Option<&UserHomepage>, privileged: bool) -> Value {
    let Some(hp) = row else {
        return json!({ "mode": "NORMAL" });
    };
    if hp.mode != "CUSTOM" {
        return json!({ "mode": "NORMAL" });
    }
    let blocked = hp.status != "ACTIVE";
    if blocked && !privileged {
        return json!({ "mode": "NORMAL" });
    }
    json!({
        "mode": "CUSTOM",
        "html": hp.html.clone().unwrap_or_default(),
        "blocked": blocked,
        "blockReason": if blocked { hp.block_reason.clone() } else { None },
        "updatedAt": j::jts(hp.updated_at),
    })
}

/// 本人视角：完整返回模式、状态与下架原因。
fn me_view(row: Option<&UserHomepage>) -> Value {
    match row {
        None => json!({
            "mode": "NORMAL",
            "html": Value::Null,
            "status": "ACTIVE",
            "blockReason": Value::Null,
            "sizeBytes": 0,
            "updatedAt": Value::Null,
        }),
        Some(hp) => json!({
            "mode": hp.mode,
            "html": hp.html,
            "status": hp.status,
            "blockReason": hp.block_reason,
            "sizeBytes": hp.size_bytes,
            "updatedAt": j::jts(hp.updated_at),
        }),
    }
}

/// GET /users/{id}/homepage
pub async fn public_get(
    State(state): State<AppState>,
    viewer: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    repo::require_user(&state.pool, id).await?;
    let row = find_homepage(&state.pool, id).await?;
    let privileged = viewer
        .0
        .as_ref()
        .map(|u| u.id == id || u.is_admin())
        .unwrap_or(false);
    Ok(j::ok(public_view(row.as_ref(), privileged)))
}

/// GET /users/me/homepage
pub async fn my_get(State(state): State<AppState>, user: CurrentUser) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let row = find_homepage(&state.pool, me.id).await?;
    Ok(j::ok(me_view(row.as_ref())))
}

#[derive(Debug, Deserialize)]
pub struct SaveHomepageRequest {
    pub mode: String,
    #[serde(default)]
    pub html: Option<String>,
}

/// PUT /users/me/homepage
pub async fn save(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<SaveHomepageRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let mode = req.mode.trim().to_ascii_uppercase();
    if mode != "NORMAL" && mode != "CUSTOM" {
        return Err(ApiError::bad_request("非法的个人主页模式"));
    }

    let existing = find_homepage(&state.pool, me.id).await?;
    if let Some(hp) = &existing {
        let elapsed = Local::now().naive_local() - hp.updated_at;
        if elapsed < Duration::seconds(SAVE_COOLDOWN_SECONDS) {
            return Err(ApiError::new(429, "保存过于频繁，请稍后再试"));
        }
    }

    let now = Local::now().naive_local();
    if mode == "NORMAL" {
        // 只切换模式，保留已存 HTML 与下架状态，便于随时切回。
        sqlx::query(
            "INSERT INTO user_homepages (user_id, mode, html, status, size_bytes, created_at, updated_at) \
             VALUES (?, 'NORMAL', NULL, 'ACTIVE', 0, ?, ?) \
             ON DUPLICATE KEY UPDATE mode = 'NORMAL', updated_at = VALUES(updated_at)",
        )
        .bind(me.id)
        .bind(now)
        .bind(now)
        .execute(&state.pool)
        .await?;
    } else {
        let raw_html = req.html.unwrap_or_default();
        if raw_html.trim().is_empty() {
            return Err(ApiError::bad_request("主页内容不能为空"));
        }
        if raw_html.len() > MAX_HOMEPAGE_BYTES {
            return Err(ApiError::bad_request("主页内容不能超过 64KB"));
        }
        let cleaned = sanitizer::sanitize_homepage(&raw_html);
        if cleaned.trim().is_empty() {
            return Err(ApiError::bad_request(
                "主页内容净化后为空，请移除不支持的标签",
            ));
        }
        if cleaned.len() > MAX_SANITIZED_BYTES {
            return Err(ApiError::bad_request("主页内容净化后过大"));
        }
        let size = cleaned.len() as i32;

        // 不触碰 status：已被下架的用户修改内容后仍需管理员复核恢复。
        sqlx::query(
            "INSERT INTO user_homepages (user_id, mode, html, status, size_bytes, created_at, updated_at) \
             VALUES (?, 'CUSTOM', ?, 'ACTIVE', ?, ?, ?) \
             ON DUPLICATE KEY UPDATE mode = 'CUSTOM', html = VALUES(html), \
             size_bytes = VALUES(size_bytes), updated_at = VALUES(updated_at)",
        )
        .bind(me.id)
        .bind(&cleaned)
        .bind(size)
        .bind(now)
        .bind(now)
        .execute(&state.pool)
        .await?;
    }

    let row = find_homepage(&state.pool, me.id).await?;
    Ok(j::ok(me_view(row.as_ref())))
}

/// POST /users/me/homepage/reset
pub async fn reset(State(state): State<AppState>, user: CurrentUser) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM user_homepage_reports WHERE user_id = ?")
        .bind(me.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM user_homepages WHERE user_id = ?")
        .bind(me.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(j::ok(me_view(None)))
}

#[derive(Debug, Deserialize)]
pub struct ReportHomepageRequest {
    pub reason: String,
}

/// POST /users/{id}/homepage/report
pub async fn report(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<ReportHomepageRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    if me.id == id {
        return Err(ApiError::bad_request("不能举报自己的主页"));
    }
    repo::require_user(&state.pool, id).await?;

    let no_custom = || ApiError::bad_request("该用户没有可举报的自定义主页");
    let row = find_homepage(&state.pool, id).await?.ok_or_else(no_custom)?;
    if row.mode != "CUSTOM" || row.status != "ACTIVE" {
        return Err(no_custom());
    }

    let reason = req.reason.trim();
    let chars = reason.chars().count();
    if !(REPORT_MIN_CHARS..=REPORT_MAX_CHARS).contains(&chars) {
        return Err(ApiError::bad_request("举报原因需为 5-500 个字符"));
    }

    let since = Local::now().naive_local() - Duration::hours(24);
    let dup: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM user_homepage_reports \
         WHERE user_id = ? AND reporter_id = ? AND created_at >= ?",
    )
    .bind(id)
    .bind(me.id)
    .bind(since)
    .fetch_one(&state.pool)
    .await?;
    if dup.0 > 0 {
        return Err(ApiError::bad_request("你已举报过该主页"));
    }

    sqlx::query(
        "INSERT INTO user_homepage_reports (user_id, reporter_id, reason, created_at) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(me.id)
    .bind(reason)
    .bind(Local::now().naive_local())
    .execute(&state.pool)
    .await?;

    Ok(j::ok(json!({ "ok": true })))
}
