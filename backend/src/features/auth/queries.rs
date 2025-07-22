use super::models::{Room, TokenType};
use sqlx::PgPool;
use sqlx::types::ipnet::IpNet;
use std::net::IpAddr;
use uuid::Uuid;

/// Fetches a room by its ID.
pub async fn get_room_by_join_code(
    pool: &PgPool,
    join_code: &str,
) -> Result<Option<Room>, sqlx::Error> {
    sqlx::query_as!(
        Room,
        r#"
        SELECT room.id, room.name, room.join_code, room.created_at, room.updated_at, room.max_members, room.started_at, (
            CASE
                WHEN max_members IS NOT NULL THEN (
                    SELECT COUNT(*)
                    FROM room_member
                    WHERE room_member.room_id = room.id
                )
            END
        ) AS "member_count"
        FROM room
        WHERE deleted_at IS NULL AND join_code = $1
        "#,
        join_code
    )
    .fetch_optional(pool)
    .await
}

/// Creates a new room member.
pub async fn new_room_member(
    pool: &PgPool,
    room_id: Uuid,
    name: &str,
    fingerprint: &str,
    public_key: &[u8],
) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO room_member (room_id, fingerprint, public_key, name)
        VALUES ($1, $2, $3, $4)
        RETURNING id;
        "#,
        room_id,
        fingerprint,
        public_key,
        name
    )
    .fetch_one(pool)
    .await
    .map(|row| row.id)
}

pub async fn get_current_member_count(pool: &PgPool, room_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT COUNT(*) AS count
        FROM room_member
        WHERE room_id = $1
        "#,
        room_id
    )
    .fetch_one(pool)
    .await
    .map(|row| row.count.unwrap_or(0))
}

/// Creates a new token for a member.
///
/// This function also deletes any pre-existing tokens of the same type for the member.
pub async fn new_token(
    pool: &PgPool,
    member_id: Uuid,
    token_type: &TokenType,
    token: &str,
    expires_at: &chrono::DateTime<chrono::Utc>,
    user_agent: Option<String>,
    ip_address: Option<IpAddr>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH deleted AS (
            DELETE FROM tokens
            WHERE member_id = $1 AND type = $2
        )
        INSERT INTO tokens (member_id, type, token, expires_at, user_agent, ip_address)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        member_id,
        token_type as &TokenType,
        token,
        expires_at,
        user_agent,
        ip_address.map(IpNet::from)
    )
    .execute(pool)
    .await?;

    Ok(())
}
