use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateChallengeTokenRequest {
    pub fingerprint: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse {
    pub challenge: String,
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
