use sqlx::MySqlPool;

use crate::error::ApiError;

/// Vote result, corresponding to Java's `VoteService.VoteOutcome`.
pub struct VoteOutcome {
    pub upvotes: i32,
    pub downvotes: i32,
    pub score: i32,
    #[allow(dead_code)]
    pub my_vote: i32,
}

/// Apply one vote: revert the old vote → write the new vote → recount. Mirrors Java's `VoteService.apply`.
/// Takes `&mut MySqlConnection` (usually from a transaction) so the count write-back shares the caller's transaction.
pub async fn apply(
    conn: &mut sqlx::MySqlConnection,
    user_id: i64,
    target_type: &str,
    target_id: i64,
    value: i32,
    current_upvotes: i32,
    current_downvotes: i32,
) -> Result<VoteOutcome, ApiError> {
    let mut upvotes = current_upvotes;
    let mut downvotes = current_downvotes;

    let existing: Option<(i64, i32)> = sqlx::query_as(
        "SELECT id, value FROM votes WHERE user_id = ? AND target_type = ? AND target_id = ?",
    )
    .bind(user_id)
    .bind(target_type)
    .bind(target_id)
    .fetch_optional(&mut *conn)
    .await?;

    match existing {
        Some((vote_id, old_value)) => {
            if old_value == 1 {
                upvotes -= 1;
            } else if old_value == -1 {
                downvotes -= 1;
            }
            if value == 0 {
                sqlx::query("DELETE FROM votes WHERE id = ?")
                    .bind(vote_id)
                    .execute(&mut *conn)
                    .await?;
            } else {
                sqlx::query("UPDATE votes SET value = ?, updated_at = NOW() WHERE id = ?")
                    .bind(value)
                    .bind(vote_id)
                    .execute(&mut *conn)
                    .await?;
            }
        }
        None => {
            if value != 0 {
                sqlx::query(
                    "INSERT INTO votes (user_id, target_type, target_id, value, created_at, updated_at) \
                     VALUES (?, ?, ?, ?, NOW(), NOW())",
                )
                .bind(user_id)
                .bind(target_type)
                .bind(target_id)
                .bind(value)
                .execute(&mut *conn)
                .await?;
            }
        }
    }

    if value == 1 {
        upvotes += 1;
    } else if value == -1 {
        downvotes += 1;
    }
    upvotes = upvotes.max(0);
    downvotes = downvotes.max(0);

    Ok(VoteOutcome {
        upvotes,
        downvotes,
        score: upvotes - downvotes,
        my_vote: value,
    })
}

/// Remove all votes for a target (called when a post/comment is deleted).
pub async fn clear_votes(
    conn: &mut sqlx::MySqlConnection,
    target_type: &str,
    target_id: i64,
) -> Result<(), ApiError> {
    sqlx::query("DELETE FROM votes WHERE target_type = ? AND target_id = ?")
        .bind(target_type)
        .bind(target_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Current user's vote value for a target; 0 when signed out or not voted.
pub async fn my_vote(
    pool: &MySqlPool,
    user_id: Option<i64>,
    target_type: &str,
    target_id: i64,
) -> Result<i32, ApiError> {
    let user_id = match user_id {
        Some(id) => id,
        None => return Ok(0),
    };
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT value FROM votes WHERE user_id = ? AND target_type = ? AND target_id = ?",
    )
    .bind(user_id)
    .bind(target_type)
    .bind(target_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(v,)| v).unwrap_or(0))
}
