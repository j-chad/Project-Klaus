use super::schemas::JoinRoomRequest;
use crate::error::AppError;
use crate::features::auth::errors::AuthError;
use crate::features::auth::queries;
use base64::{Engine, prelude::BASE64_STANDARD};
use rsa::RsaPublicKey;
use rsa::pkcs8::{DecodePublicKey, EncodePublicKey};
use sha2::{Digest, Sha256};
use tracing::error;

/// Creates a new room member and returns the user ID.
pub async fn join_room(
    pool: &sqlx::PgPool,
    payload: &JoinRoomRequest,
) -> Result<uuid::Uuid, AppError> {
    let room = queries::get_room_by_join_code(pool, &payload.room_id)
        .await?
        .ok_or(AuthError::RoomNotFound)?;

    let (public_key, fingerprint) = decode_public_key(&payload.public_key)?;

    let user = queries::new_room_member(
        pool,
        room.id,
        &payload.fingerprint,
        &payload.public_key,
        payload.is_owner,
    )
    .await?;
}

fn decode_public_key(public_key: &str) -> Result<(RsaPublicKey, String), AuthError> {
    // AuthError::InvalidPublicKey
    let public_key_bytes = BASE64_STANDARD
        .decode(public_key)
        .or(Err(AuthError::InvalidPublicKey))?;

    let key = RsaPublicKey::from_public_key_der(&public_key_bytes)
        .or(Err(AuthError::InvalidPublicKey))?;
    let fingerprint = calculate_key_fingerprint(&public_key_bytes);

    Ok((key, fingerprint))
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
