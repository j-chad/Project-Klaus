use super::errors::{ExpectedCurrent, RoomError};
use super::queries;
use crate::error::AppError;
use crate::features::auth;
use crate::features::room::models::GamePhase;
use uuid::Uuid;

pub async fn create_room(
    pool: &sqlx::PgPool,
    room_name: &str,
    username: &str,
    public_key: &str,
    max_players: Option<u32>,
) -> Result<(Uuid, String), AppError> {
    let (public_key, fingerprint) = auth::utils::cryptography::decode_public_key(public_key)?;

    let room_code = auth::utils::cryptography::generate_room_code();

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

            if current_members >= max_members as i64 {
                return Err(RoomError::RoomFull.into());
            }

            (current_members + 1) == max_members as i64
        }
        None => false,
    };

    let (public_key, fingerprint) = auth::utils::cryptography::decode_public_key(public_key)?;

    let user_id =
        queries::new_room_member(pool, room.id, username, &fingerprint, &public_key).await?;

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
    message_content: &str,
) -> Result<(), AppError> {
    expect_game_phase(db, member_id, GamePhase::SantaId).await?;

    let status = queries::get_message_round_status(db, member_id).await?;
    if status.user_has_sent_message {
        return Err(RoomError::AlreadySentMessage.into());
    }

    // create a new message in the current round
    //queries::create_santa_id_message(db, member_id, message_content).await?;

    // is the round finished? - start the next round
    if status.users_remaining <= 1 {
        //advance_message_round(db, member_id).await?;
    }

    Ok(())
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
