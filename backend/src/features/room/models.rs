pub struct Room {
    pub id: uuid::Uuid,
    pub max_members: Option<i32>,
    pub member_count: Option<i64>,
}

#[derive(sqlx::Type, serde::Serialize, Eq, PartialEq, Debug)]
#[sqlx(type_name = "game_phase", rename_all = "snake_case")]
pub enum GamePhase {
    Lobby,
    SantaId,
    SeedReveal,
    Verification,
    Rejected,
    Completed,
}

#[derive(Debug)]
pub struct MessageRoundStatus {
    pub room_id: uuid::Uuid,
    pub user_has_sent_message: bool,
    pub current_round: i32,
    pub total_users: i64,
    pub users_remaining: i64,
}
