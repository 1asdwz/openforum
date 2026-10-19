use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

use crate::config::Config;

/// Create the MySQL connection pool. The schema is maintained by the Java side's Hibernate `ddl-auto=update`; the Rust side only reads and writes.
pub async fn connect(cfg: &Config) -> anyhow::Result<MySqlPool> {
    let options = cfg.mysql_options();
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect_with(options)
        .await?;
    Ok(pool)
}
