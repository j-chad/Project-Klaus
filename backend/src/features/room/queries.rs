use super::models;
use crate::features::room::models::{GamePhase, MessageRoundStatus};
use sqlx::PgPool;
use uuid::Uuid;

/// Fetches a room by its ID.
pub async fn get_room_by_join_code(
    pool: &PgPool,
    join_code: &str,
) -> Result<Option<models::Room>, sqlx::Error> {
    sqlx::query_as!(
        models::Room,
        r#"
        SELECT room.id, room.name, room.join_code, room.created_at, room.updated_at, room.max_members, room.started_at, (
            CASE
                WHEN max_members IS NOT NULL THEN (
                    SELECT COUNT(*)
                    FROM room_member
                    WHERE room_member.room_id = room.id
                )
            END
        ) AS "member_count"
        FROM room
        WHERE deleted_at IS NULL AND join_code = $1
        "#,
        join_code
    )
        .fetch_optional(pool)
        .await
}

/// Creates a new room member.
pub async fn new_room_member(
    pool: &PgPool,
    room_id: Uuid,
    name: &str,
    fingerprint: &str,
    public_key: &[u8],
    seed_commitment: &str,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO room_member (room_id, fingerprint, public_key, name, seed_commitment)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id;
        "#,
        room_id,
        fingerprint,
        public_key,
        name,
        seed_commitment
    )
    .fetch_one(pool)
    .await
    .map(|row| row.id)
}

/// Creates a new room and an owner for that room.
///
/// Returns the ID of the newly created member (the owner).
pub async fn new_room_and_owner(
    pool: &PgPool,
    room_name: &str,
    join_code: &str,
    max_members: Option<i32>,
    username: &str,
    fingerprint: &str,
    public_key: &[u8],
    seed_commitment: &str,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        WITH new_room AS (
            INSERT INTO room (name, join_code, max_members)
            VALUES ($1, $2, $3)
            RETURNING id
        )
        INSERT INTO room_member (room_id, name, fingerprint, public_key, is_owner, seed_commitment)
        SELECT
            new_room.id, $4, $5, $6, TRUE, $7
        FROM new_room
        RETURNING id
        "#,
        room_name,
        join_code,
        max_members,
        username,
        fingerprint,
        public_key,
        seed_commitment
    )
    .fetch_one(pool)
    .await
    .map(|row| row.id)
}

pub async fn get_current_member_count(pool: &PgPool, room_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT COUNT(*) AS count
        FROM room_member
        WHERE room_id = $1
        "#,
        room_id
    )
    .fetch_one(pool)
    .await
    .map(|row| row.count.unwrap_or(0))
}

pub async fn is_owner(db: &PgPool, member_id: &Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query!("SELECT is_owner FROM room_member WHERE id = $1", member_id)
        .fetch_one(db)
        .await
        .map(|row| row.is_owner)
}

pub async fn start_game(db: &PgPool, member_id: &Uuid) -> Result<(), sqlx::Error> {
    let row_count = sqlx::query!(
        r#"
        UPDATE room
        SET started_at = NOW(), game_phase = 'santa_id'
        WHERE
            id = (SELECT room_id FROM room_member WHERE id = $1)
            AND started_at IS NULL
            AND game_phase = 'lobby'
        "#,
        member_id
    )
    .execute(db)
    .await?
    .rows_affected();

    if row_count == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn get_game_phase_by_member(
    db: &PgPool,
    member_id: &Uuid,
) -> Result<GamePhase, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT game_phase AS "game_phase: GamePhase"
        FROM room
        WHERE id = (SELECT room_id FROM room_member WHERE id = $1)
        "#,
        member_id
    )
    .fetch_one(db)
    .await
    .map(|row| row.game_phase)
}

pub async fn get_message_round_status(
    db: &PgPool,
    member_id: &Uuid,
) -> Result<MessageRoundStatus, sqlx::Error> {
    sqlx::query!(
        r#"
        WITH members_room AS (
            SELECT id
            FROM room
            WHERE id = (SELECT room_id FROM room_member WHERE id = $1)
        ),
        current_round AS (
            SELECT id, round_number
            FROM santa_id_round
            WHERE room_id = (SELECT id FROM members_room)
            ORDER BY round_number DESC
            LIMIT 1
        ),
        user_status AS (
            SELECT EXISTS(
                SELECT 1
                FROM santa_id_message message
                JOIN current_round ON message.round_id = current_round.id
                WHERE message.member_id = $1
            ) as has_sent_message
        ),
        remaining_count AS (
            SELECT COUNT(*) as remaining
            FROM room_member rm
            WHERE rm.room_id = (SELECT id FROM members_room)
              AND rm.id NOT IN (
                SELECT message.member_id
                FROM santa_id_message message
                JOIN current_round ON message.round_id = current_round.id
            )
        ),
        total_users AS (
            SELECT COUNT(*) as total
            FROM room_member
            WHERE room_id = (SELECT id FROM members_room)
        )
        SELECT
            current_round.round_number,
            members_room.id AS room_id,
            user_status.has_sent_message,
            remaining_count.remaining,
            total_users.total AS total_users
        FROM user_status, remaining_count, members_room, current_round, total_users
        "#,
        member_id,
    )
    .fetch_one(db)
    .await
    .map(|row| -> Result<MessageRoundStatus, sqlx::Error> {
        let user_has_sent_message = row.has_sent_message.ok_or(sqlx::Error::RowNotFound)?;
        let users_remaining = row.remaining.ok_or(sqlx::Error::RowNotFound)?;
        let total_users = row.total_users.ok_or(sqlx::Error::RowNotFound)?;

        Ok(MessageRoundStatus {
            user_has_sent_message,
            users_remaining,
            total_users,
            current_round: row.round_number,
            room_id: row.room_id,
        })
    })?
}

pub async fn create_santa_id_message(
    db: &PgPool,
    room_id: &Uuid,
    member_id: &Uuid,
    message_contents: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH current_round AS (
            SELECT id
            FROM santa_id_round
            WHERE room_id = $1
            ORDER BY round_number DESC
            LIMIT 1
        )
        INSERT INTO santa_id_message (member_id, content, round_id)
        SELECT $2, $3, current_round.id
        FROM current_round
        "#,
        room_id,
        member_id,
        message_contents
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn set_game_phase(
    db: &PgPool,
    room_id: &Uuid,
    game_phase: GamePhase,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE room
        SET game_phase = $2
        WHERE id = $1
        "#,
        room_id,
        game_phase as _
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn new_message_round(
    db: &PgPool,
    room_id: &Uuid,
    round_number: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO santa_id_round (room_id, round_number)
        VALUES ($1, $2)
        "#,
        room_id,
        round_number
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_seed_commitment_for_member(
    db: &PgPool,
    member_id: &Uuid,
) -> Result<String, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT seed_commitment
        FROM room_member
        WHERE id = $1
        "#,
        member_id
    )
    .fetch_one(db)
    .await
    .map(|row| row.seed_commitment)
}

pub async fn get_room_id_by_member(db: &PgPool, member_id: &Uuid) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT room_id
        FROM room_member
        WHERE id = $1
        "#,
        member_id
    )
    .fetch_one(db)
    .await
    .map(|row| row.room_id)
}

pub async fn reveal_seed(
    db: &PgPool,
    member_id: &Uuid,
    seed: &str,
) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE room_member
        SET seed = $2
        WHERE id = $1
        RETURNING (
            SELECT COUNT(*)
            FROM room_member
            WHERE room_id = (SELECT room_id FROM room_member WHERE id = $1)
              AND seed IS NULL
        ) AS remaining_users
        "#,
        member_id,
        seed
    )
    .fetch_one(db)
    .await
    .map(|row| row.remaining_users.map(|count| count as i32))
}

pub async fn mark_as_verified(db: &PgPool, member_id: &Uuid) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE room_member
        SET verification_status = TRUE
        WHERE id = $1
        RETURNING (
            SELECT COUNT(*)
            FROM room_member
            WHERE room_id = (SELECT room_id FROM room_member WHERE id = $1)
              AND verification_status IS FALSE
        ) AS remaining_users
        "#,
        member_id
    )
    .fetch_one(db)
    .await
    .map(|row| row.remaining_users.map(|count| count as i32))
}

pub async fn mark_as_rejected(
    db: &PgPool,
    member_id: &Uuid,
    proof: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH room_update AS (
            UPDATE room_member
            SET verification_status = FALSE, rejected_proof = $2
            WHERE id = $1
            RETURNING room_id
        )
        UPDATE room
        SET game_phase = 'rejected'
        FROM room_update
        WHERE room.id = room_update.room_id
        "#,
        member_id,
        proof
    )
    .execute(db)
    .await?;

    Ok(())
}
