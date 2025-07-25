use super::models::{Token, TokenType};
use sqlx::PgPool;
use sqlx::types::ipnet::IpNet;
use std::net::IpAddr;
use uuid::Uuid;

/// Creates a new token for a member.
///
/// This function also deletes any pre-existing tokens of the same type for the member.
pub async fn new_token(
    pool: &PgPool,
    member_id: Uuid,
    token_type: &TokenType,
    token: &str,
    expires_at: &chrono::DateTime<chrono::Utc>,
    user_agent: Option<&str>,
    ip_address: Option<IpAddr>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH deleted AS (
            DELETE FROM token
            WHERE member_id = $1 AND type = $2
        )
        INSERT INTO token (member_id, type, token, expires_at, user_agent, ip_address)
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

pub async fn get_session_token_and_update_access_time(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Token>, sqlx::Error> {
    sqlx::query_as!(
        Token,
        r#"
        UPDATE token
        SET last_seen_at = NOW()
        WHERE token = $1 AND type = 'session'
        RETURNING id, member_id, type AS "token_type: TokenType", created_at, expires_at, last_seen_at, user_agent, ip_address
        "#,
        token
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_all_tokens(pool: &PgPool, member_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM token
        WHERE member_id = $1
        "#,
        member_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_member_by_fingerprint(
    pool: &PgPool,
    fingerprint: &str,
) -> Result<Option<(Uuid, Vec<u8>)>, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT id, public_key
        FROM room_member
        WHERE fingerprint = $1
        "#,
        fingerprint
    )
    .fetch_optional(pool)
    .await
    .map(|row| row.map(|r| (r.id, r.public_key)))
}

pub async fn get_and_delete_challenge_token_for_fingerprint(
    pool: &PgPool,
    fingerprint: &str,
    token: &str,
) -> Result<Option<Token>, sqlx::Error> {
    sqlx::query_as!(
        Token,
        r#"
        DELETE FROM token
        USING room_member
        WHERE member_id = room_member.id
        AND room_member.fingerprint = $1
        AND token.type = 'challenge' 
        AND token.token = $2
        RETURNING token.id, token.member_id, token.type AS "token_type: TokenType", token.created_at, token.expires_at, token.last_seen_at, token.user_agent, token.ip_address
        "#,
        fingerprint,
        token
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_and_delete_ephemeral_token_by_room_code(
    pool: &PgPool,
    room_code: &str,
    token: &str,
) -> Result<Option<Token>, sqlx::Error> {
    sqlx::query_as!(
        Token,
        r#"
        DELETE FROM token 
        USING room_member rm, room r
        WHERE token.member_id = rm.id
        AND rm.room_id = r.id
        AND r.join_code = $1
        AND token.token = $2
        AND token.type = 'ephemeral'
        RETURNING token.id, token.member_id, token.type AS "token_type: TokenType", token.created_at, token.expires_at, token.last_seen_at, token.user_agent, token.ip_address
        "#,
        room_code,
        token
    )
    .fetch_optional(pool)
    .await
}
