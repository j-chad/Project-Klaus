use serde::Serialize;
use validator::Validate;

#[derive(Validate)]
pub struct JoinRoomRequest {
    pub room_id: String,
    #[validate(length(min = 1, max = 30))]
    pub name: String,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct JoinRoomResponse {
    pub session_token: String,
    pub connection_ticket: String,
}

pub struct NewSession<'a> {
    pub token: &'a str,
    pub user_agent: Option<&'a str>,
    pub ip_address: Option<std::net::IpAddr>,
}
