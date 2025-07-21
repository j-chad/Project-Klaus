use super::models::Room;
use sqlx::PgPool;
use std::net::IpAddr;

pub async fn get_room_by_join_code(pool: &PgPool, join_code: &str) -> Result<Room, sqlx::Error> {
    sqlx::query_as!(
        Room,
        r#"
        SELECT id, name, join_code, created_at, updated_at, max_members, started_at
        FROM room
        WHERE deleted_at IS NULL AND join_code = $1
        "#,
        join_code
    )
    .fetch_one(pool)
    .await
}

pub async fn new_room_member(
    pool: &PgPool,
    room_id: uuid::Uuid,
    fingerprint: &str,
    public_key: &[u8],
    is_owner: bool,
) -> Result<uuid::Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO room_member (room_id, fingerprint, public_key, is_owner)
        VALUES ($1, $2, $3, $4) RETURNING id;
        "#,
        room_id,
        fingerprint,
        public_key,
        is_owner
    )
    .fetch_one(pool)
    .await
    .map(|row| row.id)
}

pub async fn new_session(
    pool: &PgPool,
    member_id: uuid::Uuid,
    token: &str,
    user_agent: Option<&str>,
    ip_address: Option<IpAddr>,
) -> Result<uuid::Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO session (room_id, fingerprint)
        VALUES ($1, $2) RETURNING id;
        "#,
        room_id,
        fingerprint
    )
    .fetch_one(pool)
    .await
    .map(|row| row.id)
}
