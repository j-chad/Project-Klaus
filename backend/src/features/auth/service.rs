use super::schemas::JoinRoomRequest;
use crate::error::AppError;
use crate::features::auth::errors::AuthError;
use crate::features::auth::models::TokenType;
use crate::features::auth::queries;
use base64::{Engine, prelude::BASE64_STANDARD};
use rand::TryRngCore;
use rand::rngs::OsRng;
use rsa::RsaPublicKey;
use rsa::pkcs8::DecodePublicKey;
use sha2::{Digest, Sha256};
use std::net::IpAddr;
use tracing::error;

static SESSION_TOKEN_DURATION: chrono::Duration = chrono::Duration::hours(1);
static EPHEMERAL_TOKEN_DURATION: chrono::Duration = chrono::Duration::minutes(2);
static CHALLENGE_TOKEN_DURATION: chrono::Duration = chrono::Duration::minutes(2);

/// Creates a new room member and returns the user ID.
pub async fn join_room(
    pool: &sqlx::PgPool,
    payload: &JoinRoomRequest,
) -> Result<uuid::Uuid, AppError> {
    let room = queries::get_room_by_join_code(pool, &payload.room_id)
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

    let (public_key, fingerprint) = decode_public_key(&payload.public_key)?;

    let user_id =
        queries::new_room_member(pool, room.id, &payload.name, &fingerprint, &public_key).await?;
    Ok(user_id)
}

pub async fn create_session_token(
    pool: &sqlx::PgPool,
    member_id: uuid::Uuid,
    user_agent: Option<String>,
    ip_address: Option<IpAddr>,
) -> Result<String, AppError> {
    let token = generate_secure_token()?;
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
    user_agent: Option<String>,
    ip_address: Option<IpAddr>,
) -> Result<String, AppError> {
    let token = generate_secure_token()?;
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
    member_id: uuid::Uuid,
    user_agent: Option<String>,
    ip_address: Option<IpAddr>,
) -> Result<String, AppError> {
    let token = generate_secure_token()?;
    let expiration = chrono::Utc::now() + CHALLENGE_TOKEN_DURATION;

    queries::new_token(
        pool,
        member_id,
        &TokenType::Challenge,
        &token,
        &expiration,
        user_agent,
        ip_address,
    )
    .await?;
    Ok(token)
}

fn generate_secure_token() -> Result<String, AuthError> {
    let mut token = vec![0u8; 32];
    OsRng.try_fill_bytes(&mut token).or_else(|err| {
        error!("Failed to generate secure token: {}", err);
        Err(AuthError::TokenGenerationFailed)
    })?;

    Ok(BASE64_STANDARD.encode(token))
}

fn decode_public_key(public_key: &str) -> Result<(Vec<u8>, String), AuthError> {
    let public_key_bytes = BASE64_STANDARD
        .decode(public_key)
        .or(Err(AuthError::InvalidPublicKey))?;

    // validate the key - we just need the bytes for now.
    RsaPublicKey::from_public_key_der(&public_key_bytes).or(Err(AuthError::InvalidPublicKey))?;

    let fingerprint = calculate_key_fingerprint(&public_key_bytes);

    Ok((public_key_bytes, fingerprint))
}

fn calculate_key_fingerprint(public_key_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key_bytes);
    let fingerprint = hasher.finalize();

    fingerprint
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join(":")
}
