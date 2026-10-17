use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

use crate::config::Config;

/// 建立 MySQL 连接池。表结构由 Java 端 Hibernate `ddl-auto=update` 维护，Rust 端只读写。
pub async fn connect(cfg: &Config) -> anyhow::Result<MySqlPool> {
    let options = cfg.mysql_options();
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect_with(options)
        .await?;
    Ok(pool)
}
