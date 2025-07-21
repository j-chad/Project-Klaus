use super::models::{Room, Session};
use crate::features::auth::schemas::NewSession;
use sqlx::PgPool;
use sqlx::types::ipnet::IpNet;
use std::net::IpAddr;

/// Fetches a room by its ID.
pub async fn get_room_by_join_code(pool: &PgPool, join_code: &str) -> Result<Room, sqlx::Error> {
    sqlx::query_as!(
        Room,
        r#"
        SELECT id, name, join_code, created_at, updated_at, max_members, started_at
        FROM room
        WHERE deleted_at IS NULL AND join_code = $1
        "#,
        join_code
    )
    .fetch_one(pool)
    .await
}

/// Creates a new room member.
pub async fn new_room_member(
    pool: &PgPool,
    room_id: uuid::Uuid,
    fingerprint: &str,
    public_key: &[u8],
    is_owner: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO room_member (room_id, fingerprint, public_key, is_owner)
        VALUES ($1, $2, $3, $4)
        "#,
        room_id,
        fingerprint,
        public_key,
        is_owner,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Creates a new ephemeral & session token for a member.
///
/// This function also deletes any pre-existing session tokens for the member.
pub async fn new_token_pair(
    pool: &PgPool,
    member_id: uuid::Uuid,
    session_token: &str,
    ephemeral_token: &str,
    user_agent: Option<String>,
    ip_address: Option<IpAddr>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH deleted AS (
            DELETE FROM tokens
            WHERE member_id = $1
        ),
        session_token AS (
            INSERT INTO tokens (member_id, type, token, expires_at, user_agent, ip_address)
            VALUES ($1, 'session', $2, NOW() + INTERVAL '1 hour', $4, $5)
        )
        INSERT INTO tokens (member_id, type, token, expires_at, user_agent, ip_address)
            VALUES ($1, 'ephemeral', $3, NOW() + INTERVAL '1 minute', $4, $5)
        "#,
        member_id,
        session_token,
        ephemeral_token,
        user_agent,
        ip_address.map(IpNet::from)
    )
    .execute(pool)
    .await?;

    Ok(())
}
