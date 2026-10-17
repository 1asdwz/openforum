use std::collections::HashSet;

use sqlx::MySqlPool;

use crate::error::ApiError;
use crate::models::SubForum;

/// 对应 Java `ForumAccessService.isMember`。
pub async fn is_member(
    pool: &MySqlPool,
    forum_id: i64,
    user_id: Option<i64>,
) -> Result<bool, ApiError> {
    let uid = match user_id {
        Some(u) => u,
        None => return Ok(false),
    };
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM forum_members WHERE forum_id = ? AND user_id = ? LIMIT 1",
    )
    .bind(forum_id)
    .bind(uid)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

/// 对应 Java `ForumAccessService.memberForumIds`：返回用户加入的板块 id 集合。
pub async fn member_forum_ids(
    pool: &MySqlPool,
    user_id: Option<i64>,
) -> Result<HashSet<i64>, ApiError> {
    let uid = match user_id {
        Some(u) => u,
        None => return Ok(HashSet::new()),
    };
    let rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT forum_id FROM forum_members WHERE user_id = ? ORDER BY joined_at DESC",
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(f,)| f).collect())
}

/// 对应 Java `ForumAccessService.canManage`：创建者或 OWNER/MODERATOR 成员。
/// 注意：全局 ADMIN 不在此判定内（与 Java 一致）。
pub async fn can_manage(
    pool: &MySqlPool,
    forum: &SubForum,
    user_id: Option<i64>,
) -> Result<bool, ApiError> {
    let uid = match user_id {
        Some(u) => u,
        None => return Ok(false),
    };
    if uid == forum.creator_id {
        return Ok(true);
    }
    let row: Option<(String,)> =
        sqlx::query_as("SELECT role FROM forum_members WHERE forum_id = ? AND user_id = ? LIMIT 1")
            .bind(forum.id)
            .bind(uid)
            .fetch_optional(pool)
            .await?;
    Ok(matches!(
        row.map(|(r,)| r).as_deref(),
        Some("OWNER") | Some("MODERATOR")
    ))
}
