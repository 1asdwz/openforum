use sqlx::MySqlPool;

use crate::enums::AuthLevel;
use crate::error::ApiError;
use crate::gate::{GateCondition, GateRule};
use crate::info;
use crate::json::json_to_map;
use crate::state::AppState;

const CREATE_INFO_FIELDS: &str = "\
CREATE TABLE IF NOT EXISTS info_fields (\
  field_key VARCHAR(40) NOT NULL PRIMARY KEY,\
  group_key VARCHAR(20) NOT NULL,\
  label VARCHAR(40) NOT NULL,\
  value_type VARCHAR(20) NOT NULL,\
  options_json TEXT NULL,\
  authable TINYINT NOT NULL DEFAULT 0,\
  gate_ops VARCHAR(120) NOT NULL DEFAULT '',\
  sort_order INT NOT NULL DEFAULT 0,\
  enabled TINYINT NOT NULL DEFAULT 1\
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4";

const CREATE_USER_INFO: &str = "\
CREATE TABLE IF NOT EXISTS user_info (\
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,\
  user_id BIGINT NOT NULL,\
  field_key VARCHAR(40) NOT NULL,\
  value_json TEXT NULL,\
  auth_level VARCHAR(20) NOT NULL DEFAULT 'NONE',\
  auth_status VARCHAR(20) NOT NULL DEFAULT 'NONE',\
  auth_id BIGINT NULL,\
  created_at DATETIME NOT NULL,\
  updated_at DATETIME NOT NULL,\
  UNIQUE KEY uk_user_field (user_id, field_key),\
  KEY idx_field_key (field_key)\
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4";

const CREATE_USER_HOMEPAGES: &str = "\
CREATE TABLE IF NOT EXISTS user_homepages (\
  user_id BIGINT NOT NULL PRIMARY KEY,\
  mode VARCHAR(10) NOT NULL DEFAULT 'NORMAL',\
  html MEDIUMTEXT NULL,\
  status VARCHAR(10) NOT NULL DEFAULT 'ACTIVE',\
  block_reason VARCHAR(255) NULL,\
  blocked_by BIGINT NULL,\
  blocked_at DATETIME NULL,\
  size_bytes INT NOT NULL DEFAULT 0,\
  created_at DATETIME NOT NULL,\
  updated_at DATETIME NOT NULL\
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4";

const CREATE_USER_HOMEPAGE_REPORTS: &str = "\
CREATE TABLE IF NOT EXISTS user_homepage_reports (\
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,\
  user_id BIGINT NOT NULL,\
  reporter_id BIGINT NOT NULL,\
  reason VARCHAR(500) NOT NULL,\
  created_at DATETIME NOT NULL,\
  KEY idx_target_time (user_id, created_at)\
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4";

const CREATE_FORUM_HOMEPAGES: &str = "\
CREATE TABLE IF NOT EXISTS forum_homepages (\
  forum_id BIGINT NOT NULL PRIMARY KEY,\
  mode VARCHAR(10) NOT NULL DEFAULT 'NORMAL',\
  html MEDIUMTEXT NULL,\
  status VARCHAR(10) NOT NULL DEFAULT 'ACTIVE',\
  block_reason VARCHAR(255) NULL,\
  blocked_by BIGINT NULL,\
  blocked_at DATETIME NULL,\
  size_bytes INT NOT NULL DEFAULT 0,\
  created_at DATETIME NOT NULL,\
  updated_at DATETIME NOT NULL\
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4";

const CREATE_FORUM_HOMEPAGE_REPORTS: &str = "\
CREATE TABLE IF NOT EXISTS forum_homepage_reports (\
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,\
  forum_id BIGINT NOT NULL,\
  reporter_id BIGINT NOT NULL,\
  reason VARCHAR(500) NOT NULL,\
  created_at DATETIME NOT NULL,\
  KEY idx_target_time (forum_id, created_at)\
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4";

/// Idempotent migrations run at startup: create tables, add columns, seed fields, convert legacy rules, backfill legacy authentications.
pub async fn run(state: &AppState) -> Result<(), ApiError> {
    let pool = &state.pool;

    sqlx::query(CREATE_INFO_FIELDS).execute(pool).await?;
    sqlx::query(CREATE_USER_INFO).execute(pool).await?;
    sqlx::query(CREATE_USER_HOMEPAGES).execute(pool).await?;
    sqlx::query(CREATE_USER_HOMEPAGE_REPORTS).execute(pool).await?;
    sqlx::query(CREATE_FORUM_HOMEPAGES).execute(pool).await?;
    sqlx::query(CREATE_FORUM_HOMEPAGE_REPORTS).execute(pool).await?;
    tracing::info!("Migration: info_fields / user_info / user_homepages / forum_homepages ready");

    ensure_column(pool, "browse_rule", "TEXT NULL").await?;
    ensure_column(pool, "operate_rule", "TEXT NULL").await?;
    ensure_column(pool, "gate_version", "INT NOT NULL DEFAULT 0").await?;

    let permissions_added = ensure_table_column(
        pool,
        "forum_members",
        "permissions",
        "VARCHAR(255) NOT NULL DEFAULT ''",
    )
    .await?;
    ensure_table_column(pool, "forum_members", "promoted_at", "DATETIME NULL").await?;
    ensure_table_column(pool, "forum_members", "promoted_by", "BIGINT NULL").await?;
    if permissions_added {
        backfill_moderator_permissions(pool).await?;
    }

    seed_fields(pool).await?;
    convert_legacy_rules(pool).await?;
    backfill_user_info(pool).await?;

    tracing::info!("Migration: complete");
    Ok(())
}

async fn ensure_column(pool: &MySqlPool, column: &str, definition: &str) -> Result<(), ApiError> {
    ensure_table_column(pool, "sub_forums", column, definition).await?;
    Ok(())
}

/// Idempotently add a column; returns whether it was created this run (for one-time historical backfills).
async fn ensure_table_column(
    pool: &MySqlPool,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<bool, ApiError> {
    let exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND COLUMN_NAME = ?",
    )
    .bind(table)
    .bind(column)
    .fetch_one(pool)
    .await?;
    if exists.0 > 0 {
        return Ok(false);
    }
    let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
    sqlx::query(&sql).execute(pool).await?;
    tracing::info!("Migration: added column {column} to {table}");
    Ok(true)
}

/// Historical MODERATORs previously had full management rights; grant EVERYTHING during migration to preserve behavior.
async fn backfill_moderator_permissions(pool: &MySqlPool) -> Result<(), ApiError> {
    let result = sqlx::query(
        "UPDATE forum_members SET permissions = 'EVERYTHING', \
         promoted_at = COALESCE(promoted_at, joined_at) \
         WHERE role = 'MODERATOR' AND (permissions IS NULL OR permissions = '')",
    )
    .execute(pool)
    .await?;
    if result.rows_affected() > 0 {
        tracing::info!(
            "Migration: granted EVERYTHING to {} historical moderators",
            result.rows_affected()
        );
    }
    Ok(())
}

async fn seed_fields(pool: &MySqlPool) -> Result<(), ApiError> {
    for field in info::all_fields() {
        sqlx::query(
            "INSERT INTO info_fields (field_key, group_key, label, value_type, options_json, authable, gate_ops, sort_order, enabled) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON DUPLICATE KEY UPDATE group_key=VALUES(group_key), label=VALUES(label), value_type=VALUES(value_type), \
             options_json=VALUES(options_json), authable=VALUES(authable), gate_ops=VALUES(gate_ops), \
             sort_order=VALUES(sort_order), enabled=VALUES(enabled)",
        )
        .bind(field.key)
        .bind(field.group)
        .bind(field.label)
        .bind(field.value_type)
        .bind(field.options)
        .bind(field.authable)
        .bind(field.gate_ops)
        .bind(field.sort)
        .bind(field.enabled)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn convert_legacy_rules(pool: &MySqlPool) -> Result<(), ApiError> {
    let rows: Vec<(i64, Option<String>)> = sqlx::query_as(
        "SELECT id, required_auth_types FROM sub_forums WHERE gate_version = 0",
    )
    .fetch_all(pool)
    .await?;
    if rows.is_empty() {
        return Ok(());
    }
    let total = rows.len();
    for (id, csv) in rows {
        let rule = legacy_rule_from_csv(csv.as_deref());
        let json = if rule.conditions.is_empty() {
            None
        } else {
            Some(rule.to_json_string())
        };
        sqlx::query(
            "UPDATE sub_forums SET operate_rule = COALESCE(operate_rule, ?), gate_version = 1 WHERE id = ?",
        )
        .bind(json)
        .bind(id)
        .execute(pool)
        .await?;
    }
    tracing::info!("Migration: converted legacy auth rules for {total} forums");
    Ok(())
}

/// Legacy `required_auth_types` → new rule.
pub fn legacy_rule_from_csv(csv: Option<&str>) -> GateRule {
    let mut conditions: Vec<GateCondition> = Vec::new();
    let text = match csv {
        Some(s) if !s.trim().is_empty() => s,
        _ => return GateRule { version: 1, conditions },
    };
    for raw in text.split(',') {
        let req = raw.trim().to_ascii_uppercase();
        if req.is_empty() {
            continue;
        }
        if let Some(min_age) = req.strip_prefix("AGE_").and_then(|s| {
            if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
                s.parse::<i64>().ok()
            } else {
                None
            }
        }) {
            conditions.push(GateCondition {
                field: "AGE".to_string(),
                op: Some("gte".to_string()),
                min: Some(min_age as f64),
                ..Default::default()
            });
            continue;
        }

        let (type_part, level_part) = match req.split_once(':') {
            Some((t, l)) => (t.to_string(), Some(l.to_string())),
            None => (req.clone(), None),
        };
        let field_key = if type_part == "NATIONALITY" {
            "COUNTRY"
        } else {
            type_part.as_str()
        };
        if info::find_field(field_key).is_none() {
            tracing::warn!("Migration: skipping unknown auth type {type_part}");
            continue;
        }
        let level = level_part.unwrap_or_else(|| "LOW".to_string());
        let level = AuthLevel::parse(&level)
            .map(|l| l.name().to_string())
            .unwrap_or_else(|_| "LOW".to_string());
        conditions.push(GateCondition {
            field: field_key.to_string(),
            auth_level: Some(level),
            ..Default::default()
        });
    }
    GateRule { version: 1, conditions }
}

async fn backfill_user_info(pool: &MySqlPool) -> Result<(), ApiError> {
    let rows: Vec<(i64, i64, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, user_id, type, level, auth_data FROM user_authentications WHERE status = 'APPROVED'",
    )
    .fetch_all(pool)
    .await?;
    let mut written = 0usize;
    for (auth_id, user_id, auth_type, level, data) in rows {
        let data = json_to_map(data.as_deref());
        let level = AuthLevel::parse_or_null(level.as_deref())
            .unwrap_or(AuthLevel::Low)
            .name();
        for (field_key, value) in info::granted_fields(&auth_type, &data) {
            // Only fill in missing entries; do not overwrite values later entered or verified by the user.
            let result = sqlx::query(
                "INSERT IGNORE INTO user_info (user_id, field_key, value_json, auth_level, auth_status, auth_id, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, 'APPROVED', ?, NOW(), NOW())",
            )
            .bind(user_id)
            .bind(&field_key)
            .bind(value.to_string())
            .bind(level)
            .bind(auth_id)
            .execute(pool)
            .await?;
            written += result.rows_affected() as usize;
        }
    }
    if written > 0 {
        tracing::info!("Migration: backfilled {written} user info rows");
    }
    Ok(())
}
