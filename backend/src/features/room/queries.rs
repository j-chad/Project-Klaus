pub async fn is_owner(db: &sqlx::PgPool, member_id: &uuid::Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query!("SELECT is_owner FROM room_member WHERE id = $1", member_id)
        .fetch_one(db)
        .await
        .map(|row| row.is_owner)
}
