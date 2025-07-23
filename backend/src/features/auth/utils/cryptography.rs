use crate::features::auth::errors::AuthError;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use rand::RngCore;
use rand::rngs::OsRng;
use rsa::pkcs8::DecodePublicKey;
use rsa::{Oaep, RsaPublicKey};
use sha2::{Digest, Sha256};
use std::fmt::Write;
use tracing::error;

pub fn generate_secure_token() -> Result<String, AuthError> {
    let mut token = vec![0u8; 32];
    OsRng.try_fill_bytes(&mut token).map_err(|err| {
        error!("Failed to generate secure token: {}", err);
        AuthError::TokenGenerationFailed
    })?;

    Ok(BASE64_STANDARD.encode(token))
}

pub fn decode_public_key(public_key: &str) -> Result<(Vec<u8>, String), AuthError> {
    let public_key_bytes = BASE64_STANDARD
        .decode(public_key)
        .or(Err(AuthError::InvalidPublicKey))?;

    // validate the key - we just need the bytes for now.
    RsaPublicKey::from_public_key_der(&public_key_bytes).or(Err(AuthError::InvalidPublicKey))?;

    let fingerprint = calculate_key_fingerprint(&public_key_bytes);

    Ok((public_key_bytes, fingerprint))
}

pub fn calculate_key_fingerprint(public_key_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key_bytes);
    let fingerprint = hasher.finalize();

    fingerprint.iter().fold(String::new(), |mut acc, &b| {
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

pub fn encrypt_challenge_token(token: &str, public_key_bytes: &[u8]) -> Result<String, AuthError> {
    let public_key =
        RsaPublicKey::from_public_key_der(public_key_bytes).or(Err(AuthError::InvalidPublicKey))?;

    let mut rng = OsRng;

    let padding = Oaep::new::<Sha256>();
    let encrypted_data = public_key
        .encrypt(&mut rng, padding, token.as_bytes())
        .or(Err(AuthError::TokenEncryptionFailed))?;

    Ok(BASE64_STANDARD.encode(encrypted_data))
}
