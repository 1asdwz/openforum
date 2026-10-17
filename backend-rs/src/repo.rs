use sqlx::MySqlPool;

use crate::error::ApiError;
use crate::models::{AuthChallenge, AuthMedia, User, UserAuthentication};

pub async fn find_user_by_id(pool: &MySqlPool, id: i64) -> Result<Option<User>, ApiError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn require_user(pool: &MySqlPool, id: i64) -> Result<User, ApiError> {
    find_user_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::not_found("用户不存在"))
}

pub async fn find_user_by_email(pool: &MySqlPool, email: &str) -> Result<Option<User>, ApiError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn find_user_by_google_id(pool: &MySqlPool, google_id: &str) -> Result<Option<User>, ApiError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE google_id = ?")
        .bind(google_id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn find_user_by_username_or_email(
    pool: &MySqlPool,
    value: &str,
) -> Result<Option<User>, ApiError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ? OR email = ? LIMIT 1")
        .bind(value)
        .bind(value)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn exists_username(pool: &MySqlPool, username: &str) -> Result<bool, ApiError> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE username = ? LIMIT 1")
        .bind(username)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

pub async fn exists_email(pool: &MySqlPool, email: &str) -> Result<bool, ApiError> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE email = ? LIMIT 1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_user(
    pool: &MySqlPool,
    username: &str,
    email: Option<&str>,
    password: &str,
    avatar: Option<&str>,
    google_id: Option<&str>,
    role: &str,
    bio: Option<&str>,
) -> Result<User, ApiError> {
    let result = sqlx::query(
        "INSERT INTO users (username, email, password, google_id, avatar, bio, role, is_active, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, 1, NOW(), NOW())",
    )
    .bind(username)
    .bind(email)
    .bind(password)
    .bind(google_id)
    .bind(avatar)
    .bind(bio)
    .bind(role)
    .execute(pool)
    .await?;
    let id = result.last_insert_id() as i64;
    find_user_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::internal("用户创建失败"))
}

pub async fn update_user_profile(
    pool: &MySqlPool,
    id: i64,
    avatar: Option<&str>,
    bio: Option<&str>,
    google_id: Option<&str>,
) -> Result<(), ApiError> {
    if let Some(a) = avatar {
        sqlx::query("UPDATE users SET avatar = ?, updated_at = NOW() WHERE id = ?")
            .bind(a)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(b) = bio {
        sqlx::query("UPDATE users SET bio = ?, updated_at = NOW() WHERE id = ?")
            .bind(b)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(g) = google_id {
        sqlx::query("UPDATE users SET google_id = ?, updated_at = NOW() WHERE id = ?")
            .bind(g)
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn approved_type_names(pool: &MySqlPool, user_id: i64) -> Result<Vec<String>, ApiError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT type FROM user_authentications WHERE user_id = ? AND status = 'APPROVED'",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let mut names: Vec<String> = rows.into_iter().map(|(t,)| t).collect();
    names.sort();
    names.dedup();
    Ok(names)
}

// -------- 认证实体 --------

pub async fn find_auth_by_id(pool: &MySqlPool, id: i64) -> Result<Option<UserAuthentication>, ApiError> {
    let auth = sqlx::query_as::<_, UserAuthentication>("SELECT * FROM user_authentications WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(auth)
}

pub async fn find_auth_by_user_and_type<'e, E>(
    executor: E,
    user_id: i64,
    auth_type: &str,
) -> Result<Option<UserAuthentication>, ApiError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    let auth = sqlx::query_as::<_, UserAuthentication>(
        "SELECT * FROM user_authentications WHERE user_id = ? AND type = ? LIMIT 1",
    )
    .bind(user_id)
    .bind(auth_type)
    .fetch_optional(executor)
    .await?;
    Ok(auth)
}

pub async fn list_auth_by_user(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<Vec<UserAuthentication>, ApiError> {
    let rows = sqlx::query_as::<_, UserAuthentication>(
        "SELECT * FROM user_authentications WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 插入或更新认证记录，并把生成的 id 写入实体。
/// 泛型 executor：既可传 `&MySqlPool`，也可传 `&mut *tx`（在事务内保证与审核日志的原子性）。
pub async fn save_auth<'e, E>(executor: E, auth: &mut UserAuthentication) -> Result<(), ApiError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    if auth.id > 0 {
        sqlx::query(
            "UPDATE user_authentications SET user_id=?, type=?, auth_data=?, level=?, media=?, liveness=?, \
             attestation=?, status=?, review_method=?, confidence=?, reason=?, rejection_reason=?, \
             reviewed_by=?, reviewed_at=?, updated_at=NOW() WHERE id=?",
        )
        .bind(auth.user_id)
        .bind(&auth.auth_type)
        .bind(&auth.auth_data)
        .bind(&auth.level)
        .bind(&auth.media)
        .bind(&auth.liveness)
        .bind(&auth.attestation)
        .bind(&auth.status)
        .bind(&auth.review_method)
        .bind(auth.confidence)
        .bind(&auth.reason)
        .bind(&auth.rejection_reason)
        .bind(auth.reviewed_by)
        .bind(auth.reviewed_at)
        .bind(auth.id)
        .execute(executor)
        .await?;
        return Ok(());
    }
    let result = sqlx::query(
        "INSERT INTO user_authentications (user_id, type, auth_data, level, media, liveness, attestation, \
         status, review_method, confidence, reason, rejection_reason, reviewed_by, reviewed_at, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())",
    )
    .bind(auth.user_id)
    .bind(&auth.auth_type)
    .bind(&auth.auth_data)
    .bind(&auth.level)
    .bind(&auth.media)
    .bind(&auth.liveness)
    .bind(&auth.attestation)
    .bind(&auth.status)
    .bind(&auth.review_method)
    .bind(auth.confidence)
    .bind(&auth.reason)
    .bind(&auth.rejection_reason)
    .bind(auth.reviewed_by)
    .bind(auth.reviewed_at)
    .execute(executor)
    .await?;
    auth.id = result.last_insert_id() as i64;
    // 回填时间戳：INSERT 使用数据库的 NOW()，但内存结构体的 created_at/updated_at
    // 默认是 epoch(1970)。若不回填，接口响应会把 createdAt 序列化成 1970。
    // 这里用本地墙钟回填，与随后从数据库读回的值落在同一时钟上（均为 UTC）。
    let now = chrono::Local::now().naive_local();
    auth.created_at = now;
    auth.updated_at = now;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_review_log<'e, E>(
    executor: E,
    authentication_id: i64,
    level: Option<&str>,
    action: &str,
    method: Option<&str>,
    confidence: Option<f64>,
    reason: Option<&str>,
    actor_id: Option<i64>,
) -> Result<(), ApiError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    sqlx::query(
        "INSERT INTO auth_review_logs (authentication_id, level, action, method, confidence, reason, actor_id, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, NOW())",
    )
    .bind(authentication_id)
    .bind(level)
    .bind(action)
    .bind(method)
    .bind(confidence)
    .bind(reason)
    .bind(actor_id)
    .execute(executor)
    .await?;
    Ok(())
}

// -------- 认证媒体 --------

pub async fn find_auth_media(pool: &MySqlPool, id: i64) -> Result<Option<AuthMedia>, ApiError> {
    let media = sqlx::query_as::<_, AuthMedia>("SELECT * FROM auth_media WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(media)
}

pub async fn list_auth_media_in<'e, E>(
    executor: E,
    user_id: i64,
    ids: &[i64],
) -> Result<Vec<AuthMedia>, ApiError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT * FROM auth_media WHERE user_id = ? AND id IN ({placeholders})"
    );
    let mut query = sqlx::query_as::<_, AuthMedia>(&sql).bind(user_id);
    for id in ids {
        query = query.bind(id);
    }
    let rows = query.fetch_all(executor).await?;
    Ok(rows)
}

pub async fn insert_auth_media(
    pool: &MySqlPool,
    user_id: i64,
    kind: &str,
    object_key: &str,
    content_type: &str,
    size_bytes: i64,
    sha256: &str,
) -> Result<i64, ApiError> {
    let result = sqlx::query(
        "INSERT INTO auth_media (user_id, authentication_id, kind, object_key, content_type, size_bytes, sha256, created_at) \
         VALUES (?, NULL, ?, ?, ?, ?, ?, NOW())",
    )
    .bind(user_id)
    .bind(kind)
    .bind(object_key)
    .bind(content_type)
    .bind(size_bytes)
    .bind(sha256)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id() as i64)
}

pub async fn attach_auth_media<'e, E>(
    executor: E,
    authentication_id: i64,
    user_id: i64,
    ids: &[i64],
) -> Result<(), ApiError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    if ids.is_empty() {
        return Ok(());
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    // 必须带 user_id 过滤：否则用户可以把他人的 auth_media 绑定到自己的认证记录上（跨用户越权）。
    let sql = format!(
        "UPDATE auth_media SET authentication_id = ? WHERE user_id = ? AND id IN ({placeholders})"
    );
    let mut query = sqlx::query(&sql).bind(authentication_id).bind(user_id);
    for id in ids {
        query = query.bind(id);
    }
    query.execute(executor).await?;
    Ok(())
}

// -------- 活体挑战 --------

pub async fn find_challenge<'e, E>(
    executor: E,
    id: &str,
    user_id: i64,
) -> Result<Option<AuthChallenge>, ApiError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    let challenge = sqlx::query_as::<_, AuthChallenge>(
        "SELECT * FROM auth_challenges WHERE id = ? AND user_id = ?",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(executor)
    .await?;
    Ok(challenge)
}

pub async fn insert_challenge(
    pool: &MySqlPool,
    id: &str,
    user_id: i64,
    actions: &str,
    nonce: &str,
    expires_at: chrono::NaiveDateTime,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO auth_challenges (id, user_id, actions, nonce, expires_at, used_at, created_at) \
         VALUES (?, ?, ?, ?, ?, NULL, NOW())",
    )
    .bind(id)
    .bind(user_id)
    .bind(actions)
    .bind(nonce)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// 原子地「消费」活体挑战：仅当记录存在、属于该用户且尚未使用时才写入 used_at。
/// 返回是否真正抢到本次消费（false 表示已被并发请求抢先消费），用于消除 TOCTOU。
pub async fn mark_challenge_used<'e, E>(
    executor: E,
    id: &str,
    user_id: i64,
) -> Result<bool, ApiError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    let result = sqlx::query(
        "UPDATE auth_challenges SET used_at = NOW() WHERE id = ? AND user_id = ? AND used_at IS NULL",
    )
    .bind(id)
    .bind(user_id)
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}
