use super::errors::RoomError;
use super::queries;
use crate::error::AppError;

pub async fn requires_owner_permission(
    db: &sqlx::PgPool,
    member_id: &uuid::Uuid,
) -> Result<(), AppError> {
    if queries::is_owner(db, member_id).await? {
        Ok(())
    } else {
        Err(RoomError::RequiresOwnerPermission.into())
    }
}
