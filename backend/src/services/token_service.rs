#![allow(dead_code)]

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

use crate::errors::ApiError;

/// Claims embedded in the JWT access token.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject: user ID
    pub sub: String,
    /// Profile ID
    pub profile_id: String,
    /// Roles list
    pub roles: Vec<String>,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiry (Unix timestamp)
    pub exp: i64,
    /// Token type: "access" | "refresh"
    pub token_type: String,
}

impl Claims {
    pub fn user_id(&self) -> Result<Uuid, ApiError> {
        Uuid::parse_str(&self.sub).map_err(|_| ApiError::Unauthorized("Invalid token".to_string()))
    }

    pub fn profile_id(&self) -> Result<Uuid, ApiError> {
        Uuid::parse_str(&self.profile_id)
            .map_err(|_| ApiError::Unauthorized("Invalid token".to_string()))
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn is_creator(&self) -> bool {
        self.has_role("creator")
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }
}

/// Generate a JWT access token using RS256 (asymmetric) if public key available, otherwise HS256.
pub fn generate_access_token(
    user_id: Uuid,
    profile_id: Uuid,
    roles: Vec<String>,
    secret: &str,
    public_key: Option<&str>,
    expiry_secs: i64,
) -> Result<String, ApiError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        profile_id: profile_id.to_string(),
        roles,
        iat: now.timestamp(),
        exp: (now + Duration::seconds(expiry_secs)).timestamp(),
        token_type: "access".to_string(),
    };

    match public_key {
        Some(key) => encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(key.as_bytes())
                .map_err(|e| ApiError::Internal(format!("Invalid public key: {}", e)))?,
        )
        .map_err(|e| ApiError::Internal(format!("JWT encode error: {}", e))),
        None => encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| ApiError::Internal(format!("JWT encode error: {}", e))),
    }
}

/// Verify and decode a JWT access token using RS256 if private key available, otherwise HS256.
pub fn verify_access_token(
    token: &str,
    secret: &str,
    public_key: Option<&str>,
) -> Result<Claims, ApiError> {
    match public_key {
        Some(key) => {
            let mut validation = Validation::new(Algorithm::RS256);
            validation.validate_exp = true;

            decode::<Claims>(
                token,
                &DecodingKey::from_rsa_pem(key.as_bytes())
                    .map_err(|e| ApiError::Internal(format!("Invalid public key: {}", e)))?,
                &validation,
            )
            .map(|data| data.claims)
            .map_err(|e| {
                tracing::warn!("JWT RS256 validation failed: {}", e);
                ApiError::Unauthorized("Invalid token".to_string())
            })
        }
        None => {
            let mut validation = Validation::new(Algorithm::HS256);
            validation.validate_exp = true;

            decode::<Claims>(
                token,
                &DecodingKey::from_secret(secret.as_bytes()),
                &validation,
            )
            .map(|data| data.claims)
            .map_err(|e| {
                tracing::warn!("JWT HS256 validation failed: {}", e);
                ApiError::Unauthorized("Invalid token".to_string())
            })
        }
    }
}

/// Generate a cryptographically-secure random token (hex string).
pub fn generate_random_token(byte_len: usize) -> String {
    let bytes: Vec<u8> = (0..byte_len).map(|_| rand::random::<u8>()).collect();
    hex::encode(bytes)
}

/// Hash a token for storage (never store tokens in plain text).
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a Bunny.net signed CDN URL.
/// Token = HMAC-SHA256(token_key + url_path + expiry).
pub fn sign_bunny_url(
    cdn_base: &str,
    file_path: &str,
    token_key: &str,
    expiry_seconds: i64,
) -> String {
    use hmac::{Hmac, Mac};
    let expires = (Utc::now().timestamp() + expiry_seconds).to_string();
    let hash_base = format!("{}{}{}", token_key, file_path, expires);

    type HmacSha256 = Hmac<Sha256>;
    let mut mac =
        HmacSha256::new_from_slice(token_key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(hash_base.as_bytes());
    let result = mac.finalize();
    let token = {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(result.into_bytes())
    };

    format!(
        "{}/{}?token={}&expires={}",
        cdn_base, file_path, token, expires
    )
}
