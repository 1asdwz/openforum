use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiResult;
use crate::json as j;
use crate::security::CurrentUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PluginToggleRequest {
    pub enabled: Option<bool>,
}

fn plugin_dto(state: &AppState, meta: &crate::plugins::PluginMeta) -> Value {
    json!({
        "key": meta.key,
        "name": meta.name,
        "type": meta.ptype,
        "enabled": state.plugins.is_enabled(meta),
        "description": meta.description,
    })
}

/// GET /plugins
pub async fn list(State(state): State<AppState>) -> ApiResult<Json<Value>> {
    let content: Vec<Value> = state
        .plugins
        .all()
        .iter()
        .map(|meta| plugin_dto(&state, meta))
        .collect();
    Ok(j::ok(content))
}

/// PUT /plugins/{key}/enabled
pub async fn set_enabled(
    State(state): State<AppState>,
    admin: CurrentUser,
    Path(key): Path<String>,
    Json(req): Json<PluginToggleRequest>,
) -> ApiResult<Json<Value>> {
    admin.require_admin()?;
    let enabled = req.enabled.unwrap_or(false);
    state.plugins.set_enabled(&key, enabled)?;
    let meta = state
        .plugins
        .by_key(&key)
        .ok_or_else(|| crate::error::ApiError::not_found(format!("插件不存在: {key}")))?;
    Ok(j::ok(plugin_dto(&state, meta)))
}
