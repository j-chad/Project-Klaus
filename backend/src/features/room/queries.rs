use super::models;
use crate::features::room::models::GamePhase;
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
) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO room_member (room_id, fingerprint, public_key, name)
        VALUES ($1, $2, $3, $4)
        RETURNING id;
        "#,
        room_id,
        fingerprint,
        public_key,
        name
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
    max_members: Option<u32>,
    username: &str,
    fingerprint: &str,
    public_key: &[u8],
) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        WITH new_room AS (
            INSERT INTO room (name, join_code, max_members)
            VALUES ($1, $2, $3)
            RETURNING id
        )
        INSERT INTO room_member (room_id, name, fingerprint, public_key, is_owner)
        SELECT
            new_room.id, $4, $5, $6, TRUE
        FROM new_room
        RETURNING id
        "#,
        room_name,
        join_code,
        max_members.map(|m| m as i32),
        username,
        fingerprint,
        public_key
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
