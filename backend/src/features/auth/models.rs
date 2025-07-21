use sqlx::types::ipnet::IpNet;

pub struct Room {
    pub id: uuid::Uuid,
    pub name: String,
    pub join_code: String,
    pub max_members: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub enum TokenType {
    Session,
    Ephemeral,
    Challenge,
}

pub struct Token {
    pub id: uuid::Uuid,
    pub member_id: uuid::Uuid,
    pub token_type: TokenType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<IpNet>,
    pub user_agent: Option<String>,
}
