use std::collections::HashSet;

use sqlx::MySqlPool;

use crate::error::ApiError;
use crate::models::SubForum;

/// Mirrors Java `ForumAccessService.isMember`.
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

/// Mirrors Java `ForumAccessService.memberForumIds`: returns the set of forum ids the user has joined.
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

/// Moderator permissions: consistent with Reddit's moderator permission dialog.
pub const PERM_EVERYTHING: &str = "EVERYTHING";
pub const PERM_USERS: &str = "USERS";
pub const PERM_CONFIG: &str = "CONFIG";
pub const PERM_FLAIR: &str = "FLAIR";
pub const PERM_MAIL: &str = "MAIL";
pub const PERM_POSTS_COMMENTS: &str = "POSTS_COMMENTS";
pub const PERM_WIKI: &str = "WIKI";

/// All valid permissions, also used as the canonical output order.
pub const ALL_PERMISSIONS: [&str; 7] = [
    PERM_EVERYTHING,
    PERM_USERS,
    PERM_CONFIG,
    PERM_FLAIR,
    PERM_MAIL,
    PERM_POSTS_COMMENTS,
    PERM_WIKI,
];

pub fn permission_name(key: &str) -> &'static str {
    match key {
        PERM_EVERYTHING => "Everything",
        PERM_USERS => "Users",
        PERM_CONFIG => "Config",
        PERM_FLAIR => "Flair",
        PERM_MAIL => "Mail",
        PERM_POSTS_COMMENTS => "Posts & Comments",
        PERM_WIKI => "Wiki",
        _ => "Unknown permission",
    }
}

/// Normalize a permission set: drop unknown values, deduplicate, fix the order; collapse to a single entry when EVERYTHING is present.
pub fn normalize_permissions<I: IntoIterator<Item = String>>(raw: I) -> Vec<String> {
    let mut set: HashSet<String> = HashSet::new();
    for item in raw {
        let key = item.trim().to_ascii_uppercase();
        if ALL_PERMISSIONS.contains(&key.as_str()) {
            set.insert(key);
        }
    }
    if set.contains(PERM_EVERYTHING) {
        return vec![PERM_EVERYTHING.to_string()];
    }
    ALL_PERMISSIONS
        .iter()
        .filter(|key| set.contains(**key))
        .map(|key| (*key).to_string())
        .collect()
}

/// Parse the comma-separated permission set stored in the database.
pub fn parse_permissions(raw: Option<&str>) -> Vec<String> {
    match raw {
        Some(value) if !value.trim().is_empty() => {
            normalize_permissions(value.split(',').map(|item| item.to_string()))
        }
        _ => Vec::new(),
    }
}

pub fn serialize_permissions(permissions: &[String]) -> String {
    normalize_permissions(permissions.iter().cloned()).join(",")
}

/// Derive effective permissions from role and stored value: the creator/OWNER always has all permissions, MODERATOR uses the stored set, everyone else gets none.
pub fn effective_permissions(role: &str, raw: Option<&str>, is_creator: bool) -> Vec<String> {
    if is_creator || role == "OWNER" {
        vec![PERM_EVERYTHING.to_string()]
    } else if role == "MODERATOR" {
        parse_permissions(raw)
    } else {
        Vec::new()
    }
}

/// Check whether the permission set covers the target permission; EVERYTHING implies all permissions.
pub fn permits(permissions: &[String], permission: &str) -> bool {
    permissions.iter().any(|p| p == PERM_EVERYTHING || p == permission)
}

/// Compute the user's effective permissions in the given forum. The creator and OWNER always have all permissions.
pub async fn permissions_of(
    pool: &MySqlPool,
    forum: &SubForum,
    user_id: Option<i64>,
) -> Result<Vec<String>, ApiError> {
    let uid = match user_id {
        Some(u) => u,
        None => return Ok(Vec::new()),
    };
    if uid == forum.creator_id {
        return Ok(vec![PERM_EVERYTHING.to_string()]);
    }
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT role, permissions FROM forum_members WHERE forum_id = ? AND user_id = ? LIMIT 1",
    )
    .bind(forum.id)
    .bind(uid)
    .fetch_optional(pool)
    .await?;
    Ok(match row {
        Some((role, raw)) => effective_permissions(&role, raw.as_deref(), false),
        None => Vec::new(),
    })
}

/// Check whether the user has a permission; EVERYTHING implies all permissions.
pub async fn has_permission(
    pool: &MySqlPool,
    forum: &SubForum,
    user_id: Option<i64>,
    permission: &str,
) -> Result<bool, ApiError> {
    let permissions = permissions_of(pool, forum, user_id).await?;
    Ok(permits(&permissions, permission))
}

/// Require a permission, used by write endpoints.
pub async fn require_permission(
    pool: &MySqlPool,
    forum: &SubForum,
    user_id: Option<i64>,
    permission: &str,
) -> Result<(), ApiError> {
    if user_id.is_none() {
        return Err(ApiError::unauthorized("Please log in first"));
    }
    if has_permission(pool, forum, user_id, permission).await? {
        return Ok(());
    }
    Err(ApiError::forbidden(format!(
        "Missing permission: {}",
        permission_name(permission)
    )))
}

/// Mirrors Java `ForumAccessService.canManage`: whether the user has any moderator permission.
/// Note: global ADMIN is not considered here (matching Java).
pub async fn can_manage(
    pool: &MySqlPool,
    forum: &SubForum,
    user_id: Option<i64>,
) -> Result<bool, ApiError> {
    Ok(!permissions_of(pool, forum, user_id).await?.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn normalize_drops_unknown_and_dedupes() {
        let result = normalize_permissions(owned(&[
            "posts_comments",
            "CONFIG",
            "UNKNOWN",
            "config",
        ]));
        assert_eq!(result, owned(&[PERM_CONFIG, PERM_POSTS_COMMENTS]));
    }

    #[test]
    fn normalize_collapses_everything() {
        let result = normalize_permissions(owned(&[PERM_USERS, PERM_EVERYTHING, PERM_WIKI]));
        assert_eq!(result, owned(&[PERM_EVERYTHING]));
    }

    #[test]
    fn parse_handles_empty_values() {
        assert!(parse_permissions(None).is_empty());
        assert!(parse_permissions(Some("  ")).is_empty());
        assert_eq!(
            parse_permissions(Some("WIKI,FLAIR")),
            owned(&[PERM_FLAIR, PERM_WIKI])
        );
    }

    #[test]
    fn serialize_uses_canonical_order() {
        let serialized = serialize_permissions(&owned(&["WIKI", "USERS"]));
        assert_eq!(serialized, "USERS,WIKI");
    }

    #[test]
    fn creator_and_owner_always_have_everything() {
        assert_eq!(
            effective_permissions("MODERATOR", Some("WIKI"), true),
            owned(&[PERM_EVERYTHING])
        );
        assert_eq!(
            effective_permissions("OWNER", None, false),
            owned(&[PERM_EVERYTHING])
        );
    }

    #[test]
    fn moderator_permissions_come_from_stored_set() {
        assert_eq!(
            effective_permissions("MODERATOR", Some("WIKI,USERS"), false),
            owned(&[PERM_USERS, PERM_WIKI])
        );
        assert!(effective_permissions("MEMBER", Some("WIKI"), false).is_empty());
        assert!(effective_permissions("MODERATOR", None, false).is_empty());
    }

    #[test]
    fn permits_expands_everything() {
        assert!(permits(&owned(&[PERM_EVERYTHING]), PERM_WIKI));
        assert!(permits(&owned(&[PERM_USERS]), PERM_USERS));
        assert!(!permits(&owned(&[PERM_USERS]), PERM_WIKI));
        assert!(!permits(&[], PERM_USERS));
    }
}
