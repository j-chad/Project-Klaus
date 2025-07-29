use super::models;
use crate::features::room::models::{GamePhase, OnionRoundStatus};
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
        SELECT room.id, room.max_members, (
            CASE
                WHEN max_members IS NOT NULL THEN (
                    SELECT COUNT(*)
                    FROM room_member
                    WHERE room_member.room_id = room.id
                )
            END
        ) AS "member_count"
        FROM room
        WHERE join_code = $1
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
        WITH new_member AS (
            INSERT INTO room_member (room_id, fingerprint, public_key, name)
            VALUES ($1, $2, $3, $4)
            RETURNING id
        ),
        iteration AS (
            SELECT id
            FROM game_iteration
            WHERE room_id = $1
            AND iteration = 0
        )
        INSERT INTO member_iteration_state (member_id, seed_commitment, iteration_id)
        SELECT new_member.id, $5, iteration.id
        FROM new_member, iteration
        RETURNING member_id as id;
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
        ),
        new_iteration AS (
            INSERT INTO game_iteration (room_id)
            SELECT new_room.id
            FROM new_room
            RETURNING id
        ),
        new_member AS (
            INSERT INTO room_member (room_id, name, fingerprint, public_key, is_owner)
            SELECT new_room.id, $4, $5, $6, TRUE
            FROM new_room
            RETURNING id
        )
        INSERT INTO member_iteration_state (member_id, seed_commitment, iteration_id)
        SELECT new_member.id, $7, new_iteration.id
        FROM new_member, new_iteration
        RETURNING member_id as id;
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
        UPDATE game_iteration
        SET started_at = NOW(), phase = 'santa_id'
        WHERE
            room_id = (SELECT room_id FROM room_member WHERE id = $1)
            AND iteration = 0
            AND started_at IS NULL
            AND phase = 'lobby'
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
        SELECT phase AS "game_phase: GamePhase"
        FROM room_member
        JOIN game_iteration on room_member.room_id = game_iteration.room_id
        WHERE room_member.id = $1
        ORDER BY game_iteration.iteration DESC
        LIMIT 1
        "#,
        member_id
    )
    .fetch_one(db)
    .await
    .map(|row| row.game_phase)
}

pub async fn get_onion_round_status(
    db: &PgPool,
    member_id: &Uuid,
) -> Result<OnionRoundStatus, sqlx::Error> {
    sqlx::query!(
        r#"
        WITH members_room AS (
            SELECT id
            FROM room
            WHERE id = (SELECT room_id FROM room_member WHERE id = $1)
        ),
        current_iteration AS (
            SELECT id
            FROM game_iteration
            WHERE room_id = (SELECT id FROM members_room)
            ORDER BY iteration DESC
            LIMIT 1
        ),
        current_round AS (
            SELECT id, round_number
            FROM onion_round
            WHERE iteration_id = (SELECT id FROM current_iteration)
            ORDER BY round_number DESC
            LIMIT 1
        ),
        user_status AS (
            SELECT EXISTS(
                SELECT 1
                FROM onion_message message
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
                FROM onion_message message
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
    .map(|row| -> Result<OnionRoundStatus, sqlx::Error> {
        let user_has_sent_message = row.has_sent_message.ok_or(sqlx::Error::RowNotFound)?;
        let users_remaining = row.remaining.ok_or(sqlx::Error::RowNotFound)?;
        let total_users = row.total_users.ok_or(sqlx::Error::RowNotFound)?;

        Ok(OnionRoundStatus {
            user_has_sent_message,
            users_remaining,
            total_users,
            current_round: row.round_number,
            room_id: row.room_id,
        })
    })?
}

pub async fn create_onion_message(
    db: &PgPool,
    room_id: &Uuid,
    member_id: &Uuid,
    message_contents: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH current_iteration AS (
            SELECT id
            FROM game_iteration
            WHERE room_id = $1
            ORDER BY iteration DESC
            LIMIT 1
        ),
        current_round AS (
            SELECT id
            FROM onion_round
            WHERE iteration_id = (SELECT id FROM current_iteration)
            ORDER BY round_number DESC
            LIMIT 1
        )
        INSERT INTO onion_message (member_id, content, round_id)
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
        UPDATE game_iteration
        SET phase = $2
        WHERE room_id = $1
          AND iteration = (
              SELECT MAX(iteration)
              FROM game_iteration
              WHERE room_id = $1
          )
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
        WITH current_iteration AS (
            SELECT id
            FROM game_iteration
            WHERE room_id = $1
            ORDER BY iteration DESC
            LIMIT 1
        )
        INSERT INTO onion_round (iteration_id, round_number)
        SELECT current_iteration.id, $2
        FROM current_iteration
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
        FROM member_iteration_state
        WHERE member_id = $1
        ORDER BY iteration_id DESC
        LIMIT 1
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
        WITH current_iteration AS (
            SELECT game_iteration.id
            FROM room_member
            JOIN game_iteration ON room_member.room_id = game_iteration.room_id
            WHERE room_member.id = $1
            ORDER BY iteration DESC
            LIMIT 1
        )
        UPDATE member_iteration_state
        SET seed = $2
        WHERE member_id = $1
        AND iteration_id = (SELECT id FROM current_iteration)
        RETURNING (
            SELECT COUNT(*)
            FROM member_iteration_state
            JOIN current_iteration ON member_iteration_state.iteration_id = current_iteration.id
            WHERE seed IS NULL
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
        WITH current_iteration AS (
            SELECT game_iteration.id
            FROM room_member
            JOIN game_iteration ON room_member.room_id = game_iteration.room_id
            WHERE room_member.id = $1
            ORDER BY iteration DESC
            LIMIT 1
        )
        UPDATE member_iteration_state
        SET verification_status = TRUE
        WHERE member_id = $1
        AND iteration_id = (SELECT id FROM current_iteration)
        RETURNING (
            SELECT COUNT(*)
            FROM member_iteration_state
            JOIN current_iteration ON member_iteration_state.iteration_id = current_iteration.id
            WHERE verification_status = FALSE
        ) AS remaining_users
        "#,
        member_id
    )
    .fetch_one(db)
    .await
    .map(|row| row.remaining_users.map(|count| count as i32))
}

pub async fn mark_as_rejected_and_restart(
    db: &PgPool,
    member_id: &Uuid,
    proof: &str,
    seed_commitment: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH current_iteration AS (
            SELECT game_iteration.id, game_iteration.iteration, game_iteration.room_id
            FROM room_member
            JOIN game_iteration ON room_member.room_id = game_iteration.room_id
            WHERE room_member.id = $1
            AND phase = 'verification'
            ORDER BY iteration DESC
            LIMIT 1
        ),
        state_update AS (
            UPDATE member_iteration_state
            SET verification_status = TRUE, rejected_proof = $2
            WHERE member_id = $1
            AND iteration_id = (SELECT id FROM current_iteration)
        ),
        update_phase as (
            UPDATE game_iteration
            SET phase = 'rejected'
            FROM current_iteration
            WHERE game_iteration.id = current_iteration.id
        ),
        new_iteration AS (
            INSERT INTO game_iteration (room_id, iteration, phase)
            SELECT current_iteration.room_id, current_iteration.iteration + 1, 'santa_id'
            FROM current_iteration
            RETURNING id
        )
        INSERT INTO member_iteration_state (member_id, seed_commitment, iteration_id)
        SELECT $1, $3, new_iteration.id
        FROM new_iteration
        "#,
        member_id,
        proof,
        seed_commitment
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_onion_messages(db: &PgPool, room_id: &Uuid) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query!(
        r#"
        WITH current_iteration AS (
            SELECT id
            FROM game_iteration
            WHERE room_id = $1
            ORDER BY iteration DESC
            LIMIT 1
        ),
        latest_round AS (
            SELECT id
            FROM onion_round
            WHERE iteration_id = (SELECT id FROM current_iteration)
            ORDER BY round_number DESC
            LIMIT 1
        )
        SELECT array_agg(message_content) AS all_messages
        FROM onion_message message
        JOIN latest_round ON message.round_id = latest_round.id
        CROSS JOIN LATERAL unnest(message.content) AS message_content
        "#,
        room_id
    )
    .fetch_one(db)
    .await?
    .all_messages
    .ok_or(sqlx::Error::RowNotFound)
}

pub async fn get_seeds_and_names(
    db: &PgPool,
    room_id: &Uuid,
) -> Result<(Vec<String>, Vec<String>), sqlx::Error> {
    let data = sqlx::query!(
        r#"
        WITH current_iteration AS (
            SELECT id
            FROM game_iteration
            WHERE room_id = $1
            ORDER BY iteration DESC
            LIMIT 1
        )
        SELECT seed as "seed!", name
        FROM member_iteration_state member_state
        JOIN current_iteration ON member_state.iteration_id = current_iteration.id
        JOIN room_member member ON member_state.member_id = member.id
        WHERE member_state.seed IS NOT NULL
            AND member.room_id = $1
        ORDER BY name
        "#,
        room_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|row| (row.seed, row.name))
    .unzip();

    Ok(data)
}

pub async fn get_member_name(db: &PgPool, member_id: &Uuid) -> Result<String, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT name
        FROM room_member
        WHERE id = $1
        "#,
        member_id
    )
    .fetch_one(db)
    .await
    .map(|row| row.name)
}

pub async fn join_next_iteration(
    db: &PgPool,
    member_id: &Uuid,
    new_seed_commitment: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH current_iteration AS (
            SELECT id, room_id
            FROM game_iteration
            WHERE room_id = (SELECT room_id FROM room_member WHERE id = $1)
            AND phase = 'santa_id'
            ORDER BY iteration DESC
            LIMIT 1
        )
        INSERT INTO member_iteration_state (member_id, seed_commitment, iteration_id)
        SELECT $1, $2, current_iteration.id
        FROM current_iteration
        "#,
        member_id,
        new_seed_commitment
    )
    .execute(db)
    .await?;

    Ok(())
}
