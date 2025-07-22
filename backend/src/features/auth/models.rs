use sqlx::types::ipnet::IpNet;

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
