use axum::extract::{Path, State};
use axum::Json;
use chrono::{Duration, Local, NaiveDateTime};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::access;
use crate::error::{ApiError, ApiResult};
use crate::gate::{self, GateKind};
use crate::json as j;
use crate::models::{ForumHomepage, SubForum};
use crate::routes::forums;
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
/// Maximum number of items returned per list.
const LIST_LIMIT: i64 = 10;
/// Maximum number of description characters displayed.
const DESCRIPTION_CHARS: usize = 200;

pub async fn find_homepage(
    pool: &MySqlPool,
    forum_id: i64,
) -> Result<Option<ForumHomepage>, ApiError> {
    let row = sqlx::query_as::<_, ForumHomepage>(
        "SELECT mode, html, status, block_reason, size_bytes, updated_at \
         FROM forum_homepages WHERE forum_id = ?",
    )
    .bind(forum_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

async fn require_forum(pool: &MySqlPool, id: i64) -> Result<SubForum, ApiError> {
    forums::find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::not_found("Forum not found"))
}

/// Public view: blocked content is completely hidden from non-managers/non-admins.
fn public_view(row: Option<&ForumHomepage>, privileged: bool) -> Value {
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

/// Manager view: returns the full mode, status, and block reason.
fn manage_view(row: Option<&ForumHomepage>) -> Value {
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

async fn can_manage_forum(
    pool: &MySqlPool,
    forum: &SubForum,
    viewer: &CurrentUser,
) -> Result<bool, ApiError> {
    let Some(me) = viewer.0.as_ref() else {
        return Ok(false);
    };
    access::has_permission(
        pool,
        forum,
        Some(me.id),
        access::PERM_CONFIG,
    )
    .await
}

async fn ensure_manage(
    state: &AppState,
    forum: &SubForum,
    viewer: &CurrentUser,
) -> ApiResult<()> {
    if !can_manage_forum(&state.pool, forum, viewer).await? {
        return Err(ApiError::forbidden("Missing permission: Config"));
    }
    Ok(())
}

/// GET /forums/{id}/homepage
pub async fn public_get(
    State(state): State<AppState>,
    viewer: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let forum = require_forum(&state.pool, id).await?;
    let viewer_id = viewer.0.as_ref().map(|u| u.id);
    let is_admin = viewer.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    gate::ensure(&state.pool, viewer_id, is_admin, &forum, GateKind::Browse).await?;

    let privileged = is_admin || can_manage_forum(&state.pool, &forum, &viewer).await?;
    let row = find_homepage(&state.pool, forum.id).await?;
    Ok(j::ok(public_view(row.as_ref(), privileged)))
}

/// GET /forums/{id}/homepage/mine
pub async fn my_get(
    State(state): State<AppState>,
    viewer: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let forum = require_forum(&state.pool, id).await?;
    ensure_manage(&state, &forum, &viewer).await?;
    let row = find_homepage(&state.pool, forum.id).await?;
    Ok(j::ok(manage_view(row.as_ref())))
}

#[derive(Debug, Deserialize)]
pub struct SaveForumHomepageRequest {
    pub mode: String,
    #[serde(default)]
    pub html: Option<String>,
}

/// PUT /forums/{id}/homepage
pub async fn save(
    State(state): State<AppState>,
    viewer: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<SaveForumHomepageRequest>,
) -> ApiResult<Json<Value>> {
    let forum = require_forum(&state.pool, id).await?;
    ensure_manage(&state, &forum, &viewer).await?;

    let mode = req.mode.trim().to_ascii_uppercase();
    if mode != "NORMAL" && mode != "CUSTOM" {
        return Err(ApiError::bad_request("Invalid forum homepage mode"));
    }

    let existing = find_homepage(&state.pool, forum.id).await?;
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
            "INSERT INTO forum_homepages (forum_id, mode, html, status, size_bytes, created_at, updated_at) \
             VALUES (?, 'NORMAL', NULL, 'ACTIVE', 0, ?, ?) \
             ON DUPLICATE KEY UPDATE mode = 'NORMAL', updated_at = VALUES(updated_at)",
        )
        .bind(forum.id)
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
        let cleaned = sanitizer::sanitize_forum_homepage(&raw_html);
        if cleaned.trim().is_empty() {
            return Err(ApiError::bad_request(
                "Homepage content is empty after sanitization; please remove unsupported tags",
            ));
        }
        if cleaned.len() > MAX_SANITIZED_BYTES {
            return Err(ApiError::bad_request("Homepage content is too large after sanitization"));
        }
        let size = cleaned.len() as i32;

        // Leave status untouched: a blocked homepage still requires admin review to be restored after editing.
        sqlx::query(
            "INSERT INTO forum_homepages (forum_id, mode, html, status, size_bytes, created_at, updated_at) \
             VALUES (?, 'CUSTOM', ?, 'ACTIVE', ?, ?, ?) \
             ON DUPLICATE KEY UPDATE mode = 'CUSTOM', html = VALUES(html), \
             size_bytes = VALUES(size_bytes), updated_at = VALUES(updated_at)",
        )
        .bind(forum.id)
        .bind(&cleaned)
        .bind(size)
        .bind(now)
        .bind(now)
        .execute(&state.pool)
        .await?;
    }

    let row = find_homepage(&state.pool, forum.id).await?;
    Ok(j::ok(manage_view(row.as_ref())))
}

/// POST /forums/{id}/homepage/reset
pub async fn reset(
    State(state): State<AppState>,
    viewer: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let forum = require_forum(&state.pool, id).await?;
    ensure_manage(&state, &forum, &viewer).await?;

    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM forum_homepage_reports WHERE forum_id = ?")
        .bind(forum.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM forum_homepages WHERE forum_id = ?")
        .bind(forum.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(j::ok(manage_view(None)))
}

#[derive(Debug, Deserialize)]
pub struct ReportForumHomepageRequest {
    pub reason: String,
}

/// POST /forums/{id}/homepage/report
pub async fn report(
    State(state): State<AppState>,
    viewer: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<ReportForumHomepageRequest>,
) -> ApiResult<Json<Value>> {
    let me = viewer.require()?;
    let forum = require_forum(&state.pool, id).await?;
    if access::can_manage(&state.pool, &forum, Some(me.id)).await? {
        return Err(ApiError::bad_request("You cannot report a forum homepage you manage"));
    }

    let no_custom = || ApiError::bad_request("This forum has no custom homepage to report");
    let row = find_homepage(&state.pool, forum.id)
        .await?
        .ok_or_else(no_custom)?;
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
        "SELECT COUNT(*) FROM forum_homepage_reports \
         WHERE forum_id = ? AND reporter_id = ? AND created_at >= ?",
    )
    .bind(forum.id)
    .bind(me.id)
    .bind(since)
    .fetch_one(&state.pool)
    .await?;
    if dup.0 > 0 {
        return Err(ApiError::bad_request("You have already reported this homepage"));
    }

    sqlx::query(
        "INSERT INTO forum_homepage_reports (forum_id, reporter_id, reason, created_at) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(forum.id)
    .bind(me.id)
    .bind(reason)
    .bind(Local::now().naive_local())
    .execute(&state.pool)
    .await?;

    Ok(j::ok(json!({ "ok": true })))
}

#[derive(Debug, sqlx::FromRow)]
struct ForumPostRow {
    id: i64,
    title: String,
    score: i32,
    comment_count: i32,
    created_at: NaiveDateTime,
    author_name: Option<String>,
}

fn post_json(row: &ForumPostRow) -> Value {
    json!({
        "id": row.id,
        "title": row.title,
        "score": row.score,
        "commentCount": row.comment_count,
        "authorName": row.author_name,
        "createdAt": j::jts(row.created_at),
    })
}

#[derive(Debug, sqlx::FromRow)]
struct ForumMemberRow {
    user_id: i64,
    username: String,
    avatar: Option<String>,
    role: String,
    joined_at: NaiveDateTime,
}

fn truncate_description(raw: Option<&str>) -> Option<String> {
    let text = raw?.trim();
    if text.is_empty() {
        return None;
    }
    if text.chars().count() <= DESCRIPTION_CHARS {
        return Some(text.to_string());
    }
    let mut out: String = text.chars().take(DESCRIPTION_CHARS - 1).collect();
    out.push('…');
    Some(out)
}

/// GET /forums/{id}/homepage-data
///
/// For custom homepage data components: returns only public forum info, public stats, and read-only lists.
pub async fn homepage_data(
    State(state): State<AppState>,
    viewer: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let forum = require_forum(&state.pool, id).await?;
    let viewer_id = viewer.0.as_ref().map(|u| u.id);
    let is_admin = viewer.0.as_ref().map(|u| u.is_admin()).unwrap_or(false);
    gate::ensure(&state.pool, viewer_id, is_admin, &forum, GateKind::Browse).await?;

    let stats: (i64, i64, i64) = sqlx::query_as(
        "SELECT \
           (SELECT COUNT(*) FROM forum_members WHERE forum_id = ?), \
           (SELECT COUNT(*) FROM posts WHERE forum_id = ? AND is_removed = 0), \
           (SELECT COUNT(*) FROM comments c JOIN posts p ON p.id = c.post_id \
            WHERE p.forum_id = ? AND p.is_removed = 0 AND c.is_removed = 0)",
    )
    .bind(forum.id)
    .bind(forum.id)
    .bind(forum.id)
    .fetch_one(&state.pool)
    .await?;

    let posts: Vec<ForumPostRow> = sqlx::query_as(
        "SELECT p.id, p.title, p.score, p.comment_count, p.created_at, \
                u.username AS author_name \
         FROM posts p LEFT JOIN users u ON u.id = p.author_id \
         WHERE p.forum_id = ? AND p.is_removed = 0 \
         ORDER BY p.created_at DESC, p.id DESC LIMIT ?",
    )
    .bind(forum.id)
    .bind(LIST_LIMIT)
    .fetch_all(&state.pool)
    .await?;

    let hot_posts: Vec<ForumPostRow> = sqlx::query_as(
        "SELECT p.id, p.title, p.score, p.comment_count, p.created_at, \
                u.username AS author_name \
         FROM posts p LEFT JOIN users u ON u.id = p.author_id \
         WHERE p.forum_id = ? AND p.is_removed = 0 \
         ORDER BY p.score DESC, p.comment_count DESC, p.created_at DESC, p.id DESC LIMIT ?",
    )
    .bind(forum.id)
    .bind(LIST_LIMIT)
    .fetch_all(&state.pool)
    .await?;

    let members: Vec<ForumMemberRow> = sqlx::query_as(
        "SELECT fm.user_id, u.username, u.avatar, fm.role, fm.joined_at \
         FROM forum_members fm JOIN users u ON u.id = fm.user_id \
         WHERE fm.forum_id = ? \
         ORDER BY FIELD(fm.role, 'OWNER', 'MODERATOR', 'MEMBER'), fm.joined_at ASC \
         LIMIT ?",
    )
    .bind(forum.id)
    .bind(LIST_LIMIT)
    .fetch_all(&state.pool)
    .await?;

    Ok(j::ok(json!({
        "forum": {
            "id": forum.id,
            "slug": forum.slug,
            "name": forum.name,
            "description": truncate_description(forum.description.as_deref()),
            "icon": forum.icon,
            "themeColor": forum.theme_color,
            "createdAt": j::jts(forum.created_at),
        },
        "stats": {
            "members": stats.0,
            "posts": stats.1,
            "comments": stats.2,
        },
        "posts": posts.iter().map(post_json).collect::<Vec<_>>(),
        "hotPosts": hot_posts.iter().map(post_json).collect::<Vec<_>>(),
        "members": members.iter().map(|m| json!({
            "id": m.user_id,
            "username": m.username,
            "avatar": m.avatar,
            "role": m.role,
            "joinedAt": j::jts(m.joined_at),
        })).collect::<Vec<_>>(),
    })))
}
