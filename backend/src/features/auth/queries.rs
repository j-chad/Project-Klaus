use super::models::Room;
use sqlx::PgPool;

pub async fn get_room_by_join_code(pool: &PgPool, join_code: &str) -> Result<Room, sqlx::Error> {
    sqlx::query_as!(
        Room,
        r#"
        SELECT id, name, join_code, created_at, updated_at, max_members, started_at
        FROM room
        WHERE join_code = $1
        "#,
        join_code
    )
    .fetch_one(pool)
    .await
}
