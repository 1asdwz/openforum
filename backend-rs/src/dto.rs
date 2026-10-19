use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::error::ApiError;
use crate::json as j;
use crate::models::{AuthorDto, User};

/// Content preview for post summaries: trim, then truncate to 200 characters.
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

/// Condensed author info; returns null when absent (mirrors Java `PostService.toSummary`).
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

/// Public user profile DTO (mirrors Java `UserService.toDto`).
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
