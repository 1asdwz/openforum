pub mod admin;
pub mod auth;
pub mod authentications;
pub mod comments;
pub mod forum_homepages;
pub mod forums;
pub mod homepages;
pub mod info;
pub mod media;
pub mod plugins;
pub mod posts;
pub mod users;

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::state::AppState;

/// Assembles all API routes (without the `/api` prefix; main nests them uniformly).
pub fn router() -> Router<AppState> {
    Router::new()
        // -------- Auth --------
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/google", post(auth::google))
        .route("/auth/me", get(auth::me))
        // -------- Users --------
        .route("/users/me", put(users::update_me))
        .route("/users/:id", get(users::profile))
        .route("/users/:id/posts", get(users::user_posts))
        .route("/users/:id/comments", get(users::user_comments))
        // -------- Personal Homepages --------
        .route(
            "/users/me/homepage",
            get(homepages::my_get).put(homepages::save),
        )
        .route("/users/me/homepage/reset", post(homepages::reset))
        .route("/users/:id/homepage", get(homepages::public_get))
        .route("/users/:id/homepage/report", post(homepages::report))
        .route("/users/:id/homepage-data", get(homepages::homepage_data))
        // -------- User Info --------
        .route("/info/fields", get(info::fields))
        .route("/info/me", get(info::me))
        .route("/info/me/:fieldKey", put(info::update_field))
        // -------- Forums --------
        .route("/forums", get(forums::list).post(forums::create))
        .route("/forums/popular", get(forums::popular))
        .route("/forums/recommended", get(forums::recommended))
        .route("/forums/:id", get(forums::detail).put(forums::update))
        .route(
            "/forums/:id/homepage",
            get(forum_homepages::public_get).put(forum_homepages::save),
        )
        .route(
            "/forums/:id/homepage/mine",
            get(forum_homepages::my_get),
        )
        .route(
            "/forums/:id/homepage/reset",
            post(forum_homepages::reset),
        )
        .route(
            "/forums/:id/homepage/report",
            post(forum_homepages::report),
        )
        .route(
            "/forums/:id/homepage-data",
            get(forum_homepages::homepage_data),
        )
        .route("/forums/:id/join", post(forums::join))
        .route("/forums/:id/leave", post(forums::leave))
        .route("/forums/:id/members", get(forums::members))
        .route(
            "/forums/:id/moderators",
            get(forums::list_moderators).post(forums::add_moderator),
        )
        .route(
            "/forums/:id/moderators/:userId",
            put(forums::update_moderator).delete(forums::remove_moderator),
        )
        // -------- Posts --------
        .route("/posts", get(posts::list).post(posts::create))
        .route(
            "/posts/:id",
            get(posts::detail).put(posts::update).delete(posts::delete),
        )
        .route("/posts/:id/vote", post(posts::post_vote))
        .route("/posts/:id/pin", post(posts::pin))
        // -------- Comments --------
        .route("/comments/post/:postId", get(comments::tree))
        .route("/comments", post(comments::create))
        .route("/comments/:id", delete(comments::delete))
        .route("/comments/:id/vote", post(comments::comment_vote))
        // -------- Media --------
        .route("/media", post(media::upload))
        .route("/media/:id", get(media::get))
        // -------- Authentication Center --------
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
        // -------- Admin --------
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
        .route("/admin/users", get(admin::list_users))
        .route("/admin/forum-homepages", get(admin::list_forum_homepages))
        .route(
            "/admin/forum-homepages/:forumId/block",
            post(admin::block_forum_homepage),
        )
        .route(
            "/admin/forum-homepages/:forumId/unblock",
            post(admin::unblock_forum_homepage),
        )
        .route("/admin/user-homepages", get(admin::list_user_homepages))
        .route(
            "/admin/user-homepages/:userId/block",
            post(admin::block_user_homepage),
        )
        .route(
            "/admin/user-homepages/:userId/unblock",
            post(admin::unblock_user_homepage),
        )
        // -------- Plugins --------
        .route("/plugins", get(plugins::list))
        .route("/plugins/:key/enabled", put(plugins::set_enabled))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The route table mixes static and parameter segments (e.g. `/users/me/homepage` and `/users/:id/homepage`);
    /// axum panics on conflicts at build time, so this provides a safety check.
    #[test]
    fn router_builds_without_conflicts() {
        let _ = router();
    }
}
