use std::env;

/// Runtime configuration, semantically aligned with the Java side's `application.yml` / `openforum.*`.
/// Reuses the existing deployment env directly (DB_URL/DB_USER/DB_PASSWORD/MINIO_*/JWT_SECRET...).
#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,

    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,

    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,

    pub minio_enabled: bool,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_bucket: String,

    pub ai_enabled: bool,
    pub ai_api_key: String,
    pub ai_api_base: String,
    pub ai_model: String,
    pub ai_timeout_seconds: u64,

    pub google_client_id: String,

    pub auth_challenge_ttl_minutes: i64,
    pub auth_max_document_count: usize,
    pub auth_max_image_bytes: i64,
    pub auth_max_video_bytes: i64,
    pub auth_auto_approve_threshold: f64,
}

fn env_or(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(v) if !v.trim().is_empty() => v,
        _ => default.to_string(),
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}

fn env_i64(key: &str, default: i64) -> i64 {
    env::var(key).ok().and_then(|v| v.trim().parse::<i64>().ok()).unwrap_or(default)
}

fn env_f64(key: &str, default: f64) -> f64 {
    env::var(key).ok().and_then(|v| v.trim().parse::<f64>().ok()).unwrap_or(default)
}

/// Parse `jdbc:mysql://host:3306/dbname?params` and return (host, port, database).
fn parse_jdbc_mysql(url: &str) -> (String, u16, String) {
    let rest = url.trim_start_matches("jdbc:").trim_start_matches("mysql://");
    let no_query = rest.split('?').next().unwrap_or(rest);
    let (hostport, db) = match no_query.split_once('/') {
        Some((h, d)) => (h, d),
        None => (no_query, "openforum"),
    };
    let (host, port) = match hostport.split_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().unwrap_or(3306)),
        None => (hostport.to_string(), 3306),
    };
    (host, port, db.to_string())
}

impl Config {
    pub fn from_env() -> Self {
        let (db_host, db_port, db_name) =
            parse_jdbc_mysql(&env_or("DB_URL", "jdbc:mysql://localhost:3306/openforum"));
        Config {
            port: env_i64("SERVER_PORT", 8080) as u16,

            db_host,
            db_port,
            db_name,
            db_user: env_or("DB_USER", "root"),
            db_password: env_or("DB_PASSWORD", "root123456"),

            jwt_secret: env_or(
                "JWT_SECRET",
                "openforum-dev-secret-key-change-me-2026-abcdefgh",
            ),
            jwt_expiration_hours: env_i64("JWT_EXPIRATION_HOURS", 168),

            minio_enabled: env_bool("MINIO_ENABLED", false),
            minio_endpoint: env_or("MINIO_ENDPOINT", "http://localhost:9000"),
            minio_access_key: env_or("MINIO_ACCESS_KEY", "minioadmin"),
            minio_secret_key: env_or("MINIO_SECRET_KEY", "minioadmin"),
            minio_bucket: env_or("MINIO_BUCKET", "openforum-auth"),

            ai_enabled: env_bool("AI_ENABLED", false),
            ai_api_key: env_or("OPENAI_API_KEY", ""),
            ai_api_base: env_or("OPENAI_API_BASE", "https://api.openai.com/v1"),
            ai_model: env_or("OPENAI_API_MODEL", "gpt-4o-mini"),
            ai_timeout_seconds: env_i64("AI_TIMEOUT_SECONDS", 20) as u64,

            google_client_id: env_or("GOOGLE_CLIENT_ID", "").trim().to_string(),

            auth_challenge_ttl_minutes: env_i64("AUTH_CHALLENGE_TTL_MINUTES", 10),
            auth_max_document_count: env_i64("AUTH_MAX_DOCUMENT_COUNT", 5) as usize,
            auth_max_image_bytes: env_i64("AUTH_MAX_IMAGE_BYTES", 10 * 1024 * 1024),
            auth_max_video_bytes: env_i64("AUTH_MAX_VIDEO_BYTES", 50 * 1024 * 1024),
            auth_auto_approve_threshold: env_f64("AUTH_AUTO_APPROVE_THRESHOLD", 0.85),
        }
    }

    pub fn ai_configured(&self) -> bool {
        self.ai_enabled && !self.ai_api_key.trim().is_empty()
    }

    /// Startup security validation: refuse to start if key strength is insufficient (matching Java `JwtTokenProvider` failing below 32 bytes).
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.jwt_secret.len() < 32 {
            anyhow::bail!(
                "JWT_SECRET must be at least 32 bytes long (currently {}); configure a strong random secret via environment variables",
                self.jwt_secret.len()
            );
        }
        if self.jwt_secret == "openforum-dev-secret-key-change-me-2026-abcdefgh" {
            tracing::warn!("Detected the built-in default JWT_SECRET; always configure a strong random secret in production");
        }
        if self.db_password == "root123456" {
            tracing::warn!("Detected the built-in default database password; ensure production uses a separate strong password");
        }
        Ok(())
    }

    pub fn google_configured(&self) -> bool {
        !self.google_client_id.is_empty()
    }

    pub fn mysql_options(&self) -> sqlx::mysql::MySqlConnectOptions {
        sqlx::mysql::MySqlConnectOptions::new()
            .host(&self.db_host)
            .port(self.db_port)
            .username(&self.db_user)
            .password(&self.db_password)
            .database(&self.db_name)
            .charset("utf8mb4")
    }
}
