use super::errors::{ExpectedCurrent, RoomError};
use super::{queries, utils};
use crate::error::AppError;
use crate::features::auth;
use crate::features::room::models::GamePhase;
use crate::features::room::schemas::VerificationRequest;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use tracing::error;
use uuid::Uuid;

pub async fn create_room(
    pool: &sqlx::PgPool,
    room_name: &str,
    username: &str,
    public_key: &str,
    seed_commitment: &str,
    max_players: Option<u32>,
) -> Result<(Uuid, String), AppError> {
    let (public_key, fingerprint) = auth::utils::cryptography::decode_public_key(public_key)?;

    let room_code = auth::utils::cryptography::generate_room_code();

    let user_id = queries::new_room_and_owner(
        pool,
        room_name,
        &room_code,
        max_players.map(|p| p as i32),
        username,
        &fingerprint,
        &public_key,
        seed_commitment,
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
    seed_commitment: &str,
) -> Result<Uuid, AppError> {
    let room = queries::get_room_by_join_code(pool, room_id)
        .await?
        .ok_or(RoomError::RoomNotFound)?;

    let should_start_game: bool = match room.max_members {
        Some(max_members) => {
            let current_members = if let Some(count) = room.member_count {
                count
            } else {
                queries::get_current_member_count(pool, room.id).await?
            };

            if current_members >= i64::from(max_members) {
                return Err(RoomError::RoomFull.into());
            }

            (current_members + 1) == i64::from(max_members)
        }
        None => false,
    };

    let (public_key, fingerprint) = auth::utils::cryptography::decode_public_key(public_key)?;

    let user_id = queries::new_room_member(
        pool,
        room.id,
        username,
        &fingerprint,
        &public_key,
        seed_commitment,
    )
    .await?;

    if should_start_game {
        start_game(pool, &user_id).await?;
    }

    Ok(user_id)
}

pub async fn requires_owner_permission(
    db: &sqlx::PgPool,
    member_id: &Uuid,
) -> Result<(), AppError> {
    if queries::is_owner(db, member_id).await? {
        Ok(())
    } else {
        Err(RoomError::RequiresOwnerPermission.into())
    }
}

pub async fn start_game(db: &sqlx::PgPool, member_id: &Uuid) -> Result<(), AppError> {
    queries::start_game(db, member_id).await?;
    Ok(())
}

pub async fn handle_santa_id_message(
    db: &sqlx::PgPool,
    member_id: &Uuid,
    message_contents: &[String],
) -> Result<(), AppError> {
    expect_game_phase(db, member_id, GamePhase::SantaId).await?;

    let status = queries::get_message_round_status(db, member_id).await?;
    if status.user_has_sent_message {
        return Err(RoomError::AlreadySentMessage.into());
    } else if status.users_remaining == 0 {
        error!("User has not sent a message but no users remaining");
        return Err(AppError::unknown_error());
    }

    queries::create_santa_id_message(db, &status.room_id, member_id, message_contents).await?;

    if status.users_remaining == 1 {
        advance_message_round(
            db,
            &status.room_id,
            status.current_round,
            i32::try_from(status.total_users).map_err(|_e| AppError::unknown_error())?,
        )
        .await?;
    }

    Ok(())
}

pub async fn reveal_seed(db: &sqlx::PgPool, member_id: &Uuid, seed: &str) -> Result<(), AppError> {
    expect_game_phase(db, member_id, GamePhase::SeedReveal).await?;

    let seed_commitment = queries::get_seed_commitment_for_member(db, member_id).await?;
    let seed_hash = base64_hash(seed).map_err(|_| RoomError::InvalidSeed)?;
    if seed_commitment != seed_hash {
        return Err(RoomError::LiarLiarPantsOnFire(
            "Seed commitment does not match provided seed".to_string(),
        )
        .into());
    }

    let remaining_seed_reveals = queries::reveal_seed(db, member_id, seed)
        .await?
        .ok_or(AppError::unknown_error())?;

    // TODO: check if the returning value is before or after the update.
    //       I suspect it is before
    if remaining_seed_reveals == 1 {
        let room_id = queries::get_room_id_by_member(db, member_id).await?;
        queries::set_game_phase(db, &room_id, GamePhase::Verification).await?;
    }

    Ok(())
}

pub async fn handle_verification(
    db: &sqlx::PgPool,
    member_id: &Uuid,
    verification_request: &VerificationRequest,
) -> Result<(), AppError> {
    expect_game_phase(db, member_id, GamePhase::Verification).await?;

    // if rejected - check if the proof is valid
    // if so - set the game phase to Rejected
    // if accepted - update the member's verification status
    // if all members have accepted, set the game phase to Complete

    if let VerificationRequest::Rejected { proof } = verification_request {
        return handle_verification_rejection(db, member_id, proof).await;
    }

    let remaining_verifications = queries::mark_as_verified(db, member_id)
        .await?
        .ok_or(AppError::unknown_error())?;

    // TODO: check if the returning value is before or after the update.
    //       I suspect it is before
    if remaining_verifications == 1 {
        let room_id = queries::get_room_id_by_member(db, member_id).await?;
        queries::set_game_phase(db, &room_id, GamePhase::Completed).await?;
    }

    Ok(())
}

async fn handle_verification_rejection(
    db: &sqlx::PgPool,
    member_id: &Uuid,
    proof: &str,
) -> Result<(), AppError> {
    let santa_id = base64_hash(proof).map_err(|_| RoomError::InvalidRejectionProof)?;

    let room_id = queries::get_room_id_by_member(db, member_id).await?;

    // check hash is a valid santa id
    let santa_ids = queries::get_santa_id_messages(db, &room_id).await?;
    if !santa_ids.contains(&santa_id) {
        return Err(RoomError::LiarLiarPantsOnFire(
            "Provided rejection proof does not match any Santa ID".to_string(),
        )
        .into());
    }

    // construct the bijection and verify self-assignment
    let seed_components = queries::get_seed_reveals(db, &room_id).await?;
    let seed = utils::bijection::combine_seed_components(&seed_components)
        .map_err(|_| AppError::unknown_error())?;
    // if !bijection_seed.verify_self_assignment(&hash, member_id) {
    //     return Err(RoomError::LiarLiarPantsOnFire(
    //         "Rejection proof does not match the bijection seed".to_string(),
    //     )
    //     .into());
    // }

    // proof is valid
    queries::mark_as_rejected(db, member_id, proof).await?;

    Ok(())
}

async fn advance_message_round(
    db: &sqlx::PgPool,
    room_id: &Uuid,
    current_round: i32,
    members_in_room: i32,
) -> Result<(), AppError> {
    // Once N rounds have been completed - all messages should be decrypted.
    if current_round == members_in_room {
        return queries::set_game_phase(db, room_id, GamePhase::SeedReveal)
            .await
            .map_err(std::convert::Into::into);
    }

    queries::new_message_round(db, room_id, current_round + 1)
        .await
        .map_err(std::convert::Into::into)
}

async fn expect_game_phase(
    db: &sqlx::PgPool,
    member_id: &Uuid,
    expected_phase: GamePhase,
) -> Result<(), AppError> {
    let current_phase = queries::get_game_phase_by_member(db, member_id).await?;
    if current_phase != expected_phase {
        return Err(RoomError::InvalidGamePhase(ExpectedCurrent {
            expected: expected_phase,
            current: current_phase,
        })
        .into());
    }
    Ok(())
}

fn base64_hash(base64_str: &str) -> Result<String, base64::DecodeError> {
    let bytes = BASE64_STANDARD.decode(base64_str)?;

    Ok(auth::utils::cryptography::sha256_hex(&bytes))
}
