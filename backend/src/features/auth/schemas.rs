use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize)]
pub struct JoinRoomRequest {
    pub room_id: String,
    #[validate(length(min = 1, max = 30))]
    pub name: String,
    pub public_key: String, // DER encoded public key
}

#[derive(Deserialize)]
pub struct CreateChallengeTokenRequest {
    pub fingerprint: String,
}

#[derive(Deserialize)]
pub struct ChallengeVerificationRequest {
    pub token: String,
    pub fingerprint: String,
}

#[derive(Serialize)]
pub struct EphemeralTokenResponse {
    pub ephemeral_token: String,
}
