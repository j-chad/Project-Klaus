use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize)]
pub struct JoinRoomRequest {
    pub room_id: String,
    #[validate(length(min = 1, max = 30))]
    pub name: String,
    pub public_key: String, // DER encoded public key
}

#[derive(Serialize)]
pub struct JoinRoomResponse {
    pub connection_ticket: String,
}
