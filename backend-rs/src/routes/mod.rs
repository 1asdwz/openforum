pub mod admin;
pub mod auth;
pub mod authentications;
pub mod comments;
pub mod forums;
pub mod media;
pub mod plugins;
pub mod posts;
pub mod users;

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::state::AppState;

/// 组装全部 API 路由（不含 `/api` 前缀，由 main 统一 nest）。
pub fn router() -> Router<AppState> {
    Router::new()
        // -------- 认证 --------
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/google", post(auth::google))
        .route("/auth/me", get(auth::me))
        // -------- 用户 --------
        .route("/users/me", put(users::update_me))
        .route("/users/:id", get(users::profile))
        .route("/users/:id/posts", get(users::user_posts))
        .route("/users/:id/comments", get(users::user_comments))
        // -------- 子论坛 --------
        .route("/forums", get(forums::list).post(forums::create))
        .route("/forums/popular", get(forums::popular))
        .route("/forums/recommended", get(forums::recommended))
        .route("/forums/:id", get(forums::detail).put(forums::update))
        .route(
            "/forums/:id/custom-page",
            get(forums::get_custom_page).put(forums::save_custom_page),
        )
        .route(
            "/forums/:id/custom-page/submit",
            post(forums::submit_custom_page),
        )
        .route("/forums/:id/join", post(forums::join))
        .route("/forums/:id/leave", post(forums::leave))
        .route("/forums/:id/members", get(forums::members))
        // -------- 帖子 --------
        .route("/posts", get(posts::list).post(posts::create))
        .route(
            "/posts/:id",
            get(posts::detail).put(posts::update).delete(posts::delete),
        )
        .route("/posts/:id/vote", post(posts::post_vote))
        .route("/posts/:id/pin", post(posts::pin))
        // -------- 评论 --------
        .route("/comments/post/:postId", get(comments::tree))
        .route("/comments", post(comments::create))
        .route("/comments/:id", delete(comments::delete))
        .route("/comments/:id/vote", post(comments::comment_vote))
        // -------- 媒体 --------
        .route("/media", post(media::upload))
        .route("/media/:id", get(media::get))
        // -------- 认证中心 --------
        .route("/authentications/types", get(authentications::types))
        .route("/authentications/challenge", get(authentications::challenge))
        .route("/authentications/media", post(authentications::upload_media))
        .route(
            "/authentications/media/:mediaId",
            get(authentications::get_media),
        )
        .route("/authentications", post(authentications::submit))
        .route("/authentications/me", get(authentications::my))
        .route(
            "/authentications/me/approved",
            get(authentications::approved),
        )
        .route("/authentications/:id/retry", post(authentications::retry))
        // -------- 管理后台 --------
        .route(
            "/admin/authentications",
            get(admin::list_authentications),
        )
        .route(
            "/admin/authentications/:id/approve",
            post(admin::approve_authentication),
        )
        .route(
            "/admin/authentications/:id/reject",
            post(admin::reject_authentication),
        )
        .route("/admin/custom-pages", get(admin::list_custom_pages))
        .route(
            "/admin/custom-pages/:id/approve",
            post(admin::approve_custom_page),
        )
        .route(
            "/admin/custom-pages/:id/reject",
            post(admin::reject_custom_page),
        )
        .route("/admin/users", get(admin::list_users))
        // -------- 插件 --------
        .route("/plugins", get(plugins::list))
        .route("/plugins/:key/enabled", put(plugins::set_enabled))
}
