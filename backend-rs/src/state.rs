use std::sync::Arc;

use sqlx::MySqlPool;

use crate::config::Config;
use crate::plugins::PluginRegistry;
use crate::storage::Storage;

/// Shared application state. `Clone` is very cheap (everything is wrapped in `Arc`).
#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub pool: MySqlPool,
    pub http: reqwest::Client,
    pub storage: Arc<Storage>,
    pub plugins: Arc<PluginRegistry>,
}
