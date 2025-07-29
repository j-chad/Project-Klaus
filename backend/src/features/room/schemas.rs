use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize)]
pub struct CreateRoomRequest {
    #[validate(length(min = 1, max = 30))]
    pub room_name: String,
    #[validate(length(min = 1, max = 30))]
    pub username: String,
    pub max_players: Option<u32>,
    pub public_key: String, // DER encoded public key
    pub seed_hash: String,
}

#[derive(Serialize)]
pub struct CreateRoomResponse {
    pub room_id: String,
    pub ephemeral_token: String,
}

#[derive(Validate, Deserialize)]
pub struct JoinRoomRequest {
    pub room_id: String,
    #[validate(length(min = 1, max = 30))]
    pub name: String,
    pub public_key: String, // DER encoded public key
    pub seed_hash: String,
}

#[derive(Deserialize)]
pub struct SantaIDMessageRequest {
    pub message_content: Vec<String>,
}

#[derive(Deserialize)]
pub struct SeedRevealRequest {
    pub seed: String,
}

#[derive(Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum VerificationRequest {
    Accept,
    Rejected { proof: String, seed_hash: String },
}

#[derive(Deserialize)]
pub struct CommitSeedRequest {
    pub seed_hash: String,
}
