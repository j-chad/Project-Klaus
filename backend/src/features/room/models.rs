pub struct Room {
    pub id: uuid::Uuid,
    pub name: String,
    pub join_code: String,
    pub max_members: Option<i32>,
    pub member_count: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::Type, serde::Serialize, Eq, PartialEq)]
#[sqlx(type_name = "game_phase", rename_all = "snake_case")]
pub enum GamePhase {
    Lobby,
    SantaId,
    SeedCommit,
    SeedReveal,
    Verification,
    Rejected,
    Completed,
}
