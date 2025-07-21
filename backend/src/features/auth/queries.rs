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

/// Creates a new room member and session in a single transaction.
pub async fn new_room_member_and_session(
    pool: &PgPool,
    room_id: uuid::Uuid,
    fingerprint: &str,
    public_key: &[u8],
    is_owner: bool,
    session: NewSession<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH new_member AS (
            INSERT INTO room_member (room_id, fingerprint, public_key, is_owner)
            VALUES ($1, $2, $3, $4) RETURNING id
        )
        INSERT INTO session (member_id, token, user_agent, ip_address)
        SELECT new_member.id, $5, $6, $7 FROM new_member
        "#,
        room_id,
        fingerprint,
        public_key,
        is_owner,
        session.token,
        session.user_agent,
        session.ip_address.map(IpNet::from)
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Creates a new session for a member, deleting any existing sessions for that member.
pub async fn new_session(
    pool: &PgPool,
    member_id: uuid::Uuid,
    session: NewSession<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH deleted AS (
            DELETE FROM session
            WHERE member_id = $1
        )
        INSERT INTO session (member_id, token, user_agent, ip_address)
        VALUES ($1, $2, $3, $4);
        "#,
        member_id,
        session.token,
        session.user_agent,
        session.ip_address.map(IpNet::from)
    )
    .execute(pool)
    .await?;

    Ok(())
}
