use super::schemas::JoinRoomRequest;
use crate::error::AppError;
use crate::features::auth::errors::AuthError;
use crate::features::auth::queries;
use rsa::pkcs8::DecodePublicKey;
use rsa::RsaPublicKey;
use sha2::{Digest, Sha256};

/// Creates a new room member and returns the user ID.
pub async fn join_room(
    pool: &sqlx::PgPool,
    payload: &JoinRoomRequest,
) -> Result<uuid::Uuid, AppError> {
    let room = queries::get_room_by_join_code(pool, &payload.room_id)
        .await?
        .ok_or(AuthError::RoomNotFound)?;

    let public_key = RsaPublicKey::from_public_key_der()

    let user = queries::new_room_member(
        pool,
        room.id,
        &payload.fingerprint,
        &payload.public_key,
        payload.is_owner,
    )
    .await?;
}

fn calculate_key_fingerprint(public_key: &[u8]) -> String {
    return Sha256::digest(public_key)
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();
}
