use super::{errors::AuthError, models::TokenType, queries, schemas, utils::cryptography};
use crate::error::AppError;
use std::net::IpAddr;

static SESSION_TOKEN_DURATION: chrono::Duration = chrono::Duration::hours(1);
static EPHEMERAL_TOKEN_DURATION: chrono::Duration = chrono::Duration::minutes(2);
static CHALLENGE_TOKEN_DURATION: chrono::Duration = chrono::Duration::minutes(2);

pub async fn create_room(
    pool: &sqlx::PgPool,
    room_name: &str,
    username: &str,
    public_key: &str,
    max_players: Option<u32>,
) -> Result<(uuid::Uuid, String), AppError> {
    let (public_key, fingerprint) = cryptography::decode_public_key(public_key)?;

    let room_code = cryptography::generate_room_code();

    let user_id = queries::new_room_and_owner(
        pool,
        room_name,
        &room_code,
        max_players,
        username,
        &fingerprint,
        &public_key,
    )
    .await?;

    Ok((user_id, room_code))
}

/// Creates a new room member and returns the user ID.
pub async fn join_room(
    pool: &sqlx::PgPool,
    room_id: &str,
    username: &str,
    public_key: &str,
) -> Result<uuid::Uuid, AppError> {
    let room = queries::get_room_by_join_code(pool, room_id)
        .await?
        .ok_or(AuthError::RoomNotFound)?;

    if let Some(max_members) = room.max_members {
        let current_members = if let Some(count) = room.member_count {
            count
        } else {
            queries::get_current_member_count(pool, room.id).await?
        };

        if current_members >= max_members as i64 {
            return Err(AuthError::RoomFull.into());
        }
    }

    let (public_key, fingerprint) = cryptography::decode_public_key(public_key)?;

    let user_id =
        queries::new_room_member(pool, room.id, username, &fingerprint, &public_key).await?;
    Ok(user_id)
}

pub async fn create_session_token(
    pool: &sqlx::PgPool,
    member_id: uuid::Uuid,
    user_agent: Option<&str>,
    ip_address: Option<IpAddr>,
) -> Result<String, AppError> {
    let token = cryptography::generate_secure_token()?;
    let expiration = chrono::Utc::now() + SESSION_TOKEN_DURATION;

    queries::new_token(
        pool,
        member_id,
        &TokenType::Session,
        &token,
        &expiration,
        user_agent,
        ip_address,
    )
    .await?;
    Ok(token)
}

pub async fn create_ephemeral_token(
    pool: &sqlx::PgPool,
    member_id: uuid::Uuid,
    user_agent: Option<&str>,
    ip_address: Option<IpAddr>,
) -> Result<String, AppError> {
    let token = cryptography::generate_secure_token()?;
    let expiration = chrono::Utc::now() + EPHEMERAL_TOKEN_DURATION;

    queries::new_token(
        pool,
        member_id,
        &TokenType::Ephemeral,
        &token,
        &expiration,
        user_agent,
        ip_address,
    )
    .await?;
    Ok(token)
}

pub async fn create_challenge_token(
    pool: &sqlx::PgPool,
    fingerprint: &str,
    user_agent: Option<&str>,
    ip_address: Option<IpAddr>,
) -> Result<String, AppError> {
    let (member_id, public_key) = queries::get_member_by_fingerprint(pool, fingerprint)
        .await?
        .ok_or(AuthError::MemberNotFound)?;

    let raw_token = cryptography::generate_secure_token()?;
    let token = cryptography::encrypt_challenge_token(&raw_token, &public_key)?;

    let expiration = chrono::Utc::now() + CHALLENGE_TOKEN_DURATION;
    queries::new_token(
        pool,
        member_id,
        &TokenType::Challenge,
        &raw_token,
        &expiration,
        user_agent,
        ip_address,
    )
    .await?;
    Ok(token)
}

pub async fn exchange_challenge_for_session(
    pool: &sqlx::PgPool,
    token: &str,
    fingerprint: &str,
    user_agent: Option<&str>,
    ip_address: Option<IpAddr>,
) -> Result<String, AppError> {
    let challenge_token =
        queries::get_and_delete_challenge_token_for_fingerprint(pool, fingerprint, token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

    if challenge_token.expires_at < chrono::Utc::now() {
        return Err(AuthError::InvalidToken.into());
    }

    let session_token =
        create_session_token(pool, challenge_token.member_id, user_agent, ip_address).await?;
    Ok(session_token)
}

pub async fn validate_websocket_token(
    pool: &sqlx::PgPool,
    token: &str,
    room_code: &str,
) -> Result<(), AppError> {
    let token = queries::get_and_delete_ephemeral_token_by_room_code(pool, room_code, token)
        .await?
        .ok_or(AuthError::InvalidToken)?;

    if token.expires_at < chrono::Utc::now() {
        return Err(AuthError::ExpiredToken.into());
    }

    Ok(())
}

pub async fn logout(pool: &sqlx::PgPool, member_id: uuid::Uuid) -> Result<(), AppError> {
    queries::delete_all_tokens(pool, member_id).await?;
    Ok(())
}
