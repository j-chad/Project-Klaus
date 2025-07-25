#[derive(sqlx::Type)]
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
