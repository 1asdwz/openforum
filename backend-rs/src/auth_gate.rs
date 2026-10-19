use sqlx::MySqlPool;

use crate::enums::{is_known_auth_type, AuthLevel};
use crate::error::ApiError;
use crate::json::json_to_map;

/// Result of the authentication gate check, corresponding to Java's `AuthGateService.GateResult`.
pub struct GateResult {
    pub passed: bool,
    pub message: String,
}

fn is_age_requirement(req: &str) -> Option<i32> {
    let rest = req.strip_prefix("AGE_")?;
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    rest.parse::<i32>().ok()
}

fn parse_requirements(required_csv: Option<&str>) -> Vec<String> {
    match required_csv {
        Some(csv) if !csv.trim().is_empty() => csv
            .split(',')
            .map(|p| p.trim().to_uppercase())
            .filter(|p| !p.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// Evaluates a sub-forum's `required_auth_types` requirements.
///
/// Element forms: `TYPE`, `TYPE:LEVEL` (e.g. `REAL_NAME:HIGH`), `AGE_n` (legacy syntax).
pub async fn evaluate(
    pool: &MySqlPool,
    user_id: Option<i64>,
    required_csv: Option<&str>,
) -> Result<GateResult, ApiError> {
    let requirements = parse_requirements(required_csv);
    if requirements.is_empty() {
        return Ok(GateResult {
            passed: true,
            message: "No authentication requirements".to_string(),
        });
    }
    let user_id = match user_id {
        Some(id) => id,
        None => {
            return Ok(GateResult {
                passed: false,
                message: "Please sign in before accessing this forum".to_string(),
            })
        }
    };

    // type -> (level, auth_data)
    let rows: Vec<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT type, level, auth_data FROM user_authentications WHERE user_id = ? AND status = 'APPROVED'",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut approved: std::collections::HashMap<String, (AuthLevel, Option<String>)> =
        std::collections::HashMap::new();
    for (ty, level, data) in rows {
        let lvl = AuthLevel::parse_or_null(level.as_deref()).unwrap_or(AuthLevel::Low);
        approved.entry(ty).or_insert((lvl, data));
    }

    let mut missing = Vec::new();
    for req in &requirements {
        if let Some(min_age) = is_age_requirement(req) {
            let age_value = approved.get("AGE").and_then(|(_, data)| {
                json_to_map(data.as_deref()).get("age").and_then(|v| {
                    v.as_i64()
                        .map(|n| n as i32)
                        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<i32>().ok()))
                })
            });
            match age_value {
                Some(v) if v >= min_age => {}
                _ => missing.push(req.clone()),
            }
            continue;
        }

        let (type_part, level_part) = match req.split_once(':') {
            Some((t, l)) => (t.to_string(), Some(l.to_string())),
            None => (req.clone(), None),
        };
        let required_level = AuthLevel::parse_or_null(level_part.as_deref()).unwrap_or(AuthLevel::Low);
        let known = is_known_auth_type(&type_part);
        let record = if known {
            approved.get(&type_part)
        } else {
            None
        };
        match record {
            Some((lvl, _)) if lvl.rank() >= required_level.rank() => {}
            _ => missing.push(req.clone()),
        }
    }

    if missing.is_empty() {
        Ok(GateResult {
            passed: true,
            message: "Authentication requirements met".to_string(),
        })
    } else {
        Ok(GateResult {
            passed: false,
            message: format!("Authentication required: {}", missing.join(", ")),
        })
    }
}
