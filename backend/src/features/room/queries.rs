pub async fn is_owner(db: &sqlx::PgPool, member_id: &uuid::Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query!("SELECT is_owner FROM room_member WHERE id = $1", member_id)
        .fetch_one(db)
        .await
        .map(|row| row.is_owner)
}

pub async fn start_game(db: &sqlx::PgPool, member_id: &uuid::Uuid) -> Result<(), sqlx::Error> {
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
