use super::errors::RoomError;
use super::queries;
use crate::error::AppError;
use uuid::Uuid;

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
