use sqlx::types::ipnet::IpNet;
use std::fmt::Debug;

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

#[derive(sqlx::Type)]
#[sqlx(type_name = "token_type", rename_all = "lowercase")]
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
    pub user_agent: Option<String>,
    pub ip_address: Option<IpNet>,
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("id", &self.id)
            .field("member_id", &self.member_id)
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .field("last_seen_at", &self.last_seen_at)
            .field("user_agent", &self.user_agent)
            .field("ip_address", &self.ip_address)
            .finish_non_exhaustive()
    }
}
