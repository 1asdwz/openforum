use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub google_id: Option<String>,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub role: String,
    #[sqlx(rename = "is_active")]
    pub active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct SubForum {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub theme_color: Option<String>,
    pub rules: Option<String>,
    pub required_auth_types: Option<String>,
    pub template_type: String,
    pub template_config: Option<String>,
    pub custom_page_html: Option<String>,
    pub custom_page_status: Option<String>,
    pub custom_page_review_note: Option<String>,
    pub creator_id: i64,
    pub member_count: i32,
    pub post_count: i32,
    #[sqlx(rename = "is_public")]
    pub public: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct Post {
    pub id: i64,
    pub forum_id: i64,
    pub author_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub upvotes: i32,
    pub downvotes: i32,
    pub score: i32,
    pub comment_count: i32,
    #[sqlx(rename = "is_pinned")]
    pub pinned: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub author_id: i64,
    pub parent_id: Option<i64>,
    pub content: Option<String>,
    pub upvotes: i32,
    pub downvotes: i32,
    pub score: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Default, FromRow)]
pub struct UserAuthentication {
    pub id: i64,
    pub user_id: i64,
    #[sqlx(rename = "type")]
    pub auth_type: String,
    pub auth_data: Option<String>,
    pub level: Option<String>,
    pub media: Option<String>,
    pub liveness: Option<String>,
    pub attestation: Option<String>,
    pub status: String,
    pub review_method: Option<String>,
    pub confidence: Option<f64>,
    pub reason: Option<String>,
    pub rejection_reason: Option<String>,
    pub reviewed_by: Option<i64>,
    pub reviewed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct AuthChallenge {
    pub actions: Option<String>,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AuthMedia {
    pub id: i64,
    pub user_id: i64,
    pub kind: String,
    pub object_key: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
}

/// 作者精简信息（对应 Java `AuthorDTO`）。
#[derive(Debug, Clone, Serialize)]
pub struct AuthorDto {
    pub id: i64,
    pub username: String,
    pub avatar: Option<String>,
}
