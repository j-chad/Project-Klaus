use super::schemas::JoinRoomRequest;
use crate::error::AppError;
use crate::features::auth::errors::AuthError;
use crate::features::auth::queries;
use base64::{Engine, prelude::BASE64_STANDARD};
use rand::TryRngCore;
use rand::rngs::OsRng;
use rsa::RsaPublicKey;
use rsa::pkcs8::{DecodePublicKey, EncodePublicKey};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::net::IpAddr;
use tracing::error;
use uuid::Uuid;

/// Creates a new room member and returns the user ID.
pub async fn join_room(
    pool: &sqlx::PgPool,
    payload: &JoinRoomRequest,
) -> Result<uuid::Uuid, AppError> {
    let room = queries::get_room_by_join_code(pool, &payload.room_id)
        .await?
        .ok_or(AuthError::RoomNotFound)?;

    if let Some(max_members) = room.max_members {
        let current_members = queries::get_current_member_count(pool, room.id).await?;
        if current_members >= max_members as u32 {
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
    queries::new_session_token(pool, member_id, &token, user_agent, ip_address).await?;
    Ok(token)
}

pub async fn create_ephemeral_token(
    pool: &sqlx::PgPool,
    member_id: uuid::Uuid,
    user_agent: Option<String>,
    ip_address: Option<IpAddr>,
) -> Result<String, AppError> {
    let token = generate_secure_token()?;
    queries::new_ephemeral_token(pool, member_id, &token, user_agent, ip_address).await?;
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
