use std::sync::Arc;

use sqlx::MySqlPool;

use crate::config::Config;
use crate::plugins::PluginRegistry;
use crate::storage::Storage;

/// 应用共享状态。`Clone` 成本极低（全部为 `Arc` 包裹）。
#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub pool: MySqlPool,
    pub http: reqwest::Client,
    pub storage: Arc<Storage>,
    pub plugins: Arc<PluginRegistry>,
}
