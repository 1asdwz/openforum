use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::error::ApiError;
use crate::json as j;
use crate::models::{AuthorDto, User};

/// 帖子摘要用的内容预览：trim 后截断到 200 字符。
pub fn preview(content: Option<&str>) -> Option<String> {
    let content = content?;
    let trimmed = content.trim();
    if trimmed.chars().count() <= 200 {
        Some(trimmed.to_string())
    } else {
        let head: String = trimmed.chars().take(200).collect();
        Some(format!("{head}…"))
    }
}

/// 作者精简信息，不存在时返回 null（对齐 Java `PostService.toSummary`）。
pub async fn author(pool: &MySqlPool, user_id: i64) -> Result<Option<AuthorDto>, ApiError> {
    let row: Option<(i64, String, Option<String>)> =
        sqlx::query_as("SELECT id, username, avatar FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id, username, avatar)| AuthorDto {
        id,
        username,
        avatar,
    }))
}

/// 用户公开资料 DTO（对齐 Java `UserService.toDto`）。
pub fn user_dto(user: &User, authentications: Vec<String>, include_email: bool) -> Value {
    let email = if include_email {
        user.email.clone()
    } else {
        None::<String>
    };
    json!({
        "id": user.id,
        "username": user.username,
        "email": email,
        "avatar": user.avatar,
        "bio": user.bio,
        "role": user.role,
        "createdAt": j::jts(user.created_at),
        "authentications": authentications,
    })
}
