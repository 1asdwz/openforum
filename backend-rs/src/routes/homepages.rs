use std::collections::HashSet;

use axum::extract::{Path, State};
use axum::Json;
use chrono::{Duration, Local, NaiveDateTime};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{MySql, MySqlPool, QueryBuilder};

use crate::error::{ApiError, ApiResult};
use crate::gate;
use crate::json as j;
use crate::models::UserHomepage;
use crate::repo;
use crate::sanitizer;
use crate::security::CurrentUser;
use crate::state::AppState;

/// Maximum raw HTML size: 64KB.
pub const MAX_HOMEPAGE_BYTES: usize = 65_536;
/// Maximum sanitized size (including entity escape expansion): 128KB.
pub const MAX_SANITIZED_BYTES: usize = 131_072;
/// Save cooldown (seconds).
const SAVE_COOLDOWN_SECONDS: i64 = 3;
/// Report reason length range (characters).
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

/// Public view: blocked content is completely hidden from anyone other than the owner/admin.
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

/// Owner view: returns the full mode, status, and block reason.
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
        return Err(ApiError::bad_request("Invalid personal homepage mode"));
    }

    let existing = find_homepage(&state.pool, me.id).await?;
    if let Some(hp) = &existing {
        let elapsed = Local::now().naive_local() - hp.updated_at;
        if elapsed < Duration::seconds(SAVE_COOLDOWN_SECONDS) {
            return Err(ApiError::new(429, "Saving too frequently, please try again later"));
        }
    }

    let now = Local::now().naive_local();
    if mode == "NORMAL" {
        // Only switch the mode; keep the stored HTML and blocked status so it can be switched back at any time.
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
            return Err(ApiError::bad_request("Homepage content must not be empty"));
        }
        if raw_html.len() > MAX_HOMEPAGE_BYTES {
            return Err(ApiError::bad_request("Homepage content must not exceed 64KB"));
        }
        let cleaned = sanitizer::sanitize_homepage(&raw_html);
        if cleaned.trim().is_empty() {
            return Err(ApiError::bad_request(
                "Homepage content is empty after sanitization; please remove unsupported tags",
            ));
        }
        if cleaned.len() > MAX_SANITIZED_BYTES {
            return Err(ApiError::bad_request("Homepage content is too large after sanitization"));
        }
        let size = cleaned.len() as i32;

        // Leave status untouched: a blocked user's homepage still requires admin review to be restored after editing.
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
        return Err(ApiError::bad_request("You cannot report your own homepage"));
    }
    repo::require_user(&state.pool, id).await?;

    let no_custom = || ApiError::bad_request("This user has no custom homepage to report");
    let row = find_homepage(&state.pool, id).await?.ok_or_else(no_custom)?;
    if row.mode != "CUSTOM" || row.status != "ACTIVE" {
        return Err(no_custom());
    }

    let reason = req.reason.trim();
    let chars = reason.chars().count();
    if !(REPORT_MIN_CHARS..=REPORT_MAX_CHARS).contains(&chars) {
        return Err(ApiError::bad_request("Report reason must be 5-500 characters"));
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
        return Err(ApiError::bad_request("You have already reported this homepage"));
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

/// Maximum number of items returned per list.
const HOMEPAGE_LIST_LIMIT: usize = 10;
/// Maximum number of bio characters displayed.
const HOMEPAGE_BIO_CHARS: usize = 200;

#[derive(Debug, sqlx::FromRow)]
struct HomepagePostRow {
    id: i64,
    title: String,
    score: i32,
    comment_count: i32,
    created_at: NaiveDateTime,
    forum_id: i64,
    forum_name: String,
    forum_slug: String,
}

fn post_row_json(row: &HomepagePostRow) -> Value {
    json!({
        "id": row.id,
        "title": row.title,
        "forumName": row.forum_name,
        "forumSlug": row.forum_slug,
        "score": row.score,
        "commentCount": row.comment_count,
        "createdAt": j::jts(row.created_at),
    })
}

/// Filter by forums visible to the viewer first, then truncate, so hidden content does not consume list slots.
fn visible_posts(rows: Vec<HomepagePostRow>, allowed: &HashSet<i64>) -> Vec<Value> {
    rows.into_iter()
        .filter(|row| allowed.contains(&row.forum_id))
        .take(HOMEPAGE_LIST_LIMIT)
        .map(|row| post_row_json(&row))
        .collect()
}

/// Writes the visible forum set into an `IN (...)` placeholder list.
fn push_forum_filter(qb: &mut QueryBuilder<'_, MySql>, ids: &[i64]) {
    qb.push("(");
    let mut separated = qb.separated(", ");
    for id in ids {
        separated.push_bind(*id);
    }
    separated.push_unseparated(")");
}

/// Stats only cover forums the viewer can browse, preventing restricted forum content from being inferred via counts.
async fn viewer_scoped_stats(
    pool: &MySqlPool,
    user_id: i64,
    allowed: &HashSet<i64>,
) -> Result<(i64, i64, i64), ApiError> {
    if allowed.is_empty() {
        return Ok((0, 0, 0));
    }
    let mut ids: Vec<i64> = allowed.iter().copied().collect();
    ids.sort_unstable();

    let mut qb = QueryBuilder::new(
        "SELECT (SELECT COUNT(*) FROM posts p WHERE p.author_id = ",
    );
    qb.push_bind(user_id);
    qb.push(" AND p.is_removed = 0 AND p.forum_id IN ");
    push_forum_filter(&mut qb, &ids);
    qb.push("), (SELECT COUNT(*) FROM comments c JOIN posts p ON p.id = c.post_id WHERE c.author_id = ");
    qb.push_bind(user_id);
    qb.push(" AND c.is_removed = 0 AND p.is_removed = 0 AND p.forum_id IN ");
    push_forum_filter(&mut qb, &ids);
    qb.push("), (SELECT CAST(COALESCE(SUM(p.upvotes), 0) AS SIGNED) FROM posts p WHERE p.author_id = ");
    qb.push_bind(user_id);
    qb.push(" AND p.is_removed = 0 AND p.forum_id IN ");
    push_forum_filter(&mut qb, &ids);
    qb.push(") + (SELECT CAST(COALESCE(SUM(c.upvotes), 0) AS SIGNED) FROM comments c JOIN posts p ON p.id = c.post_id WHERE c.author_id = ");
    qb.push_bind(user_id);
    qb.push(" AND c.is_removed = 0 AND p.is_removed = 0 AND p.forum_id IN ");
    push_forum_filter(&mut qb, &ids);
    qb.push(")");

    let row: (i64, i64, i64) = qb.build_query_as().fetch_one(pool).await?;
    Ok(row)
}

fn truncate_bio(raw: Option<&str>) -> Option<String> {
    let text = raw?.trim();
    if text.is_empty() {
        return None;
    }
    if text.chars().count() <= HOMEPAGE_BIO_CHARS {
        return Some(text.to_string());
    }
    let mut out: String = text.chars().take(HOMEPAGE_BIO_CHARS - 1).collect();
    out.push('…');
    Some(out)
}

/// GET /users/{id}/homepage-data
///
/// For custom homepage data components: returns only public profile info, public stats, and three read-only post lists.
/// Email, authentication info, and user info fields are not returned; stats and lists are filtered by the viewer's forum browse permissions.
pub async fn homepage_data(
    State(state): State<AppState>,
    viewer: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let user = repo::require_user(&state.pool, id).await?;
    let viewer_id = viewer.0.as_ref().map(|u| u.id);
    let is_admin = viewer.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    let allowed = gate::allowed_forum_ids(&state.pool, viewer_id, is_admin).await?;

    let stats = viewer_scoped_stats(&state.pool, id, &allowed).await?;

    let authored: Vec<HomepagePostRow> = sqlx::query_as(
        "SELECT p.id, p.title, p.score, p.comment_count, p.created_at, p.forum_id, \
                f.name AS forum_name, f.slug AS forum_slug \
         FROM posts p JOIN sub_forums f ON f.id = p.forum_id \
         WHERE p.author_id = ? AND p.is_removed = 0 \
         ORDER BY p.created_at DESC, p.id DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let liked: Vec<HomepagePostRow> = sqlx::query_as(
        "SELECT p.id, p.title, p.score, p.comment_count, p.created_at, p.forum_id, \
                f.name AS forum_name, f.slug AS forum_slug \
         FROM votes v \
         JOIN posts p ON p.id = v.target_id \
         JOIN sub_forums f ON f.id = p.forum_id \
         WHERE v.user_id = ? AND v.target_type = 'POST' AND v.value = 1 AND p.is_removed = 0 \
         ORDER BY v.updated_at DESC, v.id DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let commented: Vec<HomepagePostRow> = sqlx::query_as(
        "SELECT p.id, p.title, p.score, p.comment_count, p.created_at, p.forum_id, \
                f.name AS forum_name, f.slug AS forum_slug, MAX(c.created_at) AS last_comment_at \
         FROM comments c \
         JOIN posts p ON p.id = c.post_id \
         JOIN sub_forums f ON f.id = p.forum_id \
         WHERE c.author_id = ? AND c.is_removed = 0 AND p.is_removed = 0 \
         GROUP BY p.id, p.title, p.score, p.comment_count, p.created_at, p.forum_id, f.name, f.slug \
         ORDER BY last_comment_at DESC, p.id DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(j::ok(json!({
        "profile": {
            "id": user.id,
            "username": user.username,
            "avatar": user.avatar,
            "bio": truncate_bio(user.bio.as_deref()),
            "role": user.role,
            "createdAt": j::jts(user.created_at),
        },
        "stats": {
            "posts": stats.0,
            "comments": stats.1,
            "upvotes": stats.2,
        },
        "posts": visible_posts(authored, &allowed),
        "liked": visible_posts(liked, &allowed),
        "commented": visible_posts(commented, &allowed),
    })))
}
