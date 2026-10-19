use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::enums::AuthLevel;
use crate::error::{ApiError, ApiResult};
use crate::info;
use crate::json as j;
use crate::security::CurrentUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    pub value: Option<Value>,
}

struct InfoRow {
    value: Value,
    auth_level: String,
    auth_status: String,
}

async fn load_rows(
    state: &AppState,
    user_id: i64,
) -> Result<HashMap<String, InfoRow>, ApiError> {
    let rows: Vec<(String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT field_key, value_json, auth_level, auth_status FROM user_info WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;
    let mut map = HashMap::new();
    for (key, raw, level, status) in rows {
        map.insert(
            key.to_ascii_uppercase(),
            InfoRow {
                value: raw
                    .as_deref()
                    .and_then(|s| serde_json::from_str::<Value>(s).ok())
                    .unwrap_or(Value::Null),
                auth_level: level.unwrap_or_else(|| "NONE".to_string()),
                auth_status: status.unwrap_or_else(|| "NONE".to_string()),
            },
        );
    }
    Ok(map)
}

fn rank(raw: &str) -> i32 {
    AuthLevel::parse_or_null(Some(raw))
        .map(|l| l.rank())
        .unwrap_or(0)
}

/// GET /info/fields
pub async fn fields() -> ApiResult<Json<Value>> {
    let mut groups: Vec<Value> = Vec::new();
    for (group_key, group_label) in info::GROUPS {
        let fields: Vec<Value> = info::VISIBLE_FIELDS
            .iter()
            .filter(|f| f.enabled && f.group == group_key)
            .map(|f| {
                json!({
                    "fieldKey": f.key,
                    "label": f.label,
                    "valueType": f.value_type,
                    "options": info::options_value(f),
                    "authable": f.authable,
                    "gateOps": info::gate_ops(f),
                    "authLevels": info::auth_levels(f),
                })
            })
            .collect();
        groups.push(json!({
            "groupKey": group_key,
            "groupLabel": group_label,
            "fields": fields,
        }));
    }
    Ok(j::ok(groups))
}

/// GET /info/me
pub async fn me(
    State(state): State<AppState>,
    user: CurrentUser,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let rows = load_rows(&state, me.id).await?;

    let total = info::VISIBLE_FIELDS.len();
    let mut filled = 0usize;
    let mut groups: Vec<Value> = Vec::new();
    for (group_key, group_label) in info::GROUPS {
        let mut items: Vec<Value> = Vec::new();
        for field in info::VISIBLE_FIELDS.iter().filter(|f| f.enabled && f.group == group_key) {
            let row = rows.get(field.key);
            let value = row.map(|r| r.value.clone()).unwrap_or(Value::Null);
            let is_filled = info::is_filled(&value);
            if is_filled {
                filled += 1;
            }
            let auth_level = row.map(|r| r.auth_level.clone()).unwrap_or_else(|| "NONE".to_string());
            let auth_status = row.map(|r| r.auth_status.clone()).unwrap_or_else(|| "NONE".to_string());
            let can_upgrade = field.authable
                && auth_status != "PENDING"
                && rank(&auth_level) < AuthLevel::Extreme.rank();
            let value_text = if is_filled {
                info::value_text(field.key, &value)
            } else {
                String::new()
            };
            items.push(json!({
                "fieldKey": field.key,
                "label": field.label,
                "value": if is_filled { value } else { Value::Null },
                "valueText": value_text,
                "authLevel": auth_level,
                "authStatus": auth_status,
                "canUpgrade": can_upgrade,
                "privacy": "PRIVATE",
            }));
        }
        groups.push(json!({
            "groupKey": group_key,
            "groupLabel": group_label,
            "items": items,
        }));
    }

    let percent = if total == 0 { 0 } else { filled * 100 / total };
    Ok(j::ok(json!({
        "completeness": { "filled": filled, "total": total, "percent": percent },
        "groups": groups,
    })))
}

/// PUT /info/me/:fieldKey
pub async fn update_field(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(field_key): Path<String>,
    Json(req): Json<UpdateRequest>,
) -> ApiResult<Json<Value>> {
    let me = user.require()?;
    let field = info::find_field(&field_key)
        .filter(|f| f.enabled)
        .ok_or_else(|| ApiError::bad_request(format!("Unknown info field: {field_key}")))?;
    let raw = req
        .value
        .ok_or_else(|| ApiError::bad_request("value must not be empty"))?;
    let value = info::validate_value(field.key, &raw)?;

    let existing: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT value_json, auth_level, auth_status FROM user_info WHERE user_id = ? AND field_key = ?",
    )
    .bind(me.id)
    .bind(field.key)
    .fetch_optional(&state.pool)
    .await?;

    let existing_value = existing
        .as_ref()
        .and_then(|(v, _, _)| v.as_deref())
        .and_then(|s| serde_json::from_str::<Value>(s).ok());
    let existing_level = existing
        .as_ref()
        .and_then(|(_, l, _)| l.clone())
        .unwrap_or_else(|| "NONE".to_string());
    let existing_status = existing
        .as_ref()
        .and_then(|(_, _, s)| s.clone())
        .unwrap_or_else(|| "NONE".to_string());

    if existing_status == "PENDING" {
        return Err(ApiError::new(409, "This info field is under review"));
    }

    let changed = existing_value.as_ref() != Some(&value);
    let (level, status, message) = if !field.authable {
        ("NONE".to_string(), "NONE".to_string(), None)
    } else if changed && rank(&existing_level) >= AuthLevel::Medium.rank() {
        (
            "NONE".to_string(),
            "NONE".to_string(),
            Some("Information updated; re-authentication required"),
        )
    } else if !changed && existing_status == "APPROVED" {
        (existing_level.clone(), existing_status.clone(), None)
    } else {
        ("LOW".to_string(), "APPROVED".to_string(), None)
    };

    sqlx::query(
        "INSERT INTO user_info (user_id, field_key, value_json, auth_level, auth_status, auth_id, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, NULL, NOW(), NOW()) \
         ON DUPLICATE KEY UPDATE value_json=VALUES(value_json), auth_level=VALUES(auth_level), \
         auth_status=VALUES(auth_status), auth_id=NULL, updated_at=NOW()",
    )
    .bind(me.id)
    .bind(field.key)
    .bind(value.to_string())
    .bind(&level)
    .bind(&status)
    .execute(&state.pool)
    .await?;

    let mut data = json!({
        "fieldKey": field.key,
        "valueText": info::value_text(field.key, &value),
        "authLevel": level,
        "authStatus": status,
    });
    if let Some(message) = message {
        data["message"] = json!(message);
    }
    Ok(j::ok(data))
}
