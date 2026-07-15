#![allow(dead_code)]

use crate::errors::AppResult;
#[allow(unused_imports)]
use crate::models::user::User;
#[allow(unused_imports)]
use crate::security::SecurityService;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub profile_id: Uuid,
    pub role: i32,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_token(
    user_id: Uuid,
    profile_id: Uuid,
    role_claim: i32,
    jwt_secret: &str,
    expiry_secs: i64,
) -> AppResult<String> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id,
        profile_id,
        role: role_claim,
        exp: now + expiry_secs as usize,
        iat: now,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )?;
    Ok(token)
}

pub fn verify_token(token: &str, jwt_secret: &str) -> AppResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(token_data.claims)
}

pub fn hash_token(_token: &str, _salt: &str) -> String {
    use argon2::Argon2;
    use rand::rngs::OsRng;
    use rand::RngCore;

    // Generate a random 16-byte salt for Argon2id
    let mut salt_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut salt_bytes);

    // Hash with Argon2id (same params as password hashing)
    let mut output = [0u8; 32];
    let params = argon2::Params::new(65536, 3, 1, Some(32)).unwrap();
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    argon2
        .hash_password_into(_token.as_bytes(), &salt_bytes, &mut output)
        .expect("Argon2id hashing should not fail");

    // Return salt:hash format for storage
    format!("{}:{}", hex::encode(salt_bytes), hex::encode(output))
}

/// Verify a token against a stored Argon2id hash (format: salt:hash).
pub fn verify_token_hash(token: &str, stored: &str) -> bool {
    use argon2::Argon2;

    let parts: Vec<&str> = stored.splitn(2, ':').collect();
    if parts.len() != 2 {
        return false;
    }

    let salt_bytes = match hex::decode(parts[0]) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let expected_hash = match hex::decode(parts[1]) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let mut output = vec![0u8; expected_hash.len()];
    let params = argon2::Params::new(65536, 3, 1, Some(expected_hash.len())).unwrap();
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    argon2
        .hash_password_into(token.as_bytes(), &salt_bytes, &mut output)
        .expect("Argon2id hashing should not fail");

    // Constant-time comparison
    output == expected_hash
}

pub fn generate_random_token(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_token() {
        let user_id = Uuid::new_v4();
        let profile_id = Uuid::new_v4();
        let role_claim = 1;
        let jwt_secret = "test-secret-key-for-testing-only";
        let expiry_secs = 3600;

        let token = create_token(user_id, profile_id, role_claim, jwt_secret, expiry_secs).unwrap();
        let claims = verify_token(&token, jwt_secret).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.profile_id, profile_id);
        assert_eq!(claims.role, role_claim);
    }

    #[test]
    fn test_verify_token_fails_with_wrong_secret() {
        let user_id = Uuid::new_v4();
        let profile_id = Uuid::new_v4();
        let role_claim = 1;
        let jwt_secret = "test-secret-key-for-testing-only";
        let wrong_secret = "wrong-secret";
        let expiry_secs = 3600;

        let token = create_token(user_id, profile_id, role_claim, jwt_secret, expiry_secs).unwrap();
        let result = verify_token(&token, wrong_secret);

        assert!(result.is_err());
    }

    #[test]
    fn test_hash_token_deterministic() {
        let token = "test-refresh-token-12345";
        let salt = "test_salt";
        let hash1 = hash_token(token, salt);
        let hash2 = hash_token(token, salt);

        // Argon2id uses random salt, so hashes are different each time
        assert_ne!(hash1, hash2);
        // But both should contain the salt:hash format
        assert!(hash1.contains(':'));
        assert!(hash2.contains(':'));
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let salt = "test_salt";
        let hash1 = hash_token("token1", salt);
        let hash2 = hash_token("token2", salt);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_different_salts() {
        let token = "test-refresh-token-12345";
        let hash1 = hash_token(token, "salt1");
        let hash2 = hash_token(token, "salt2");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_token_hash() {
        let token = "test-refresh-token-12345";
        let salt = "test_salt";
        let stored = hash_token(token, salt);

        assert!(verify_token_hash(token, &stored));
        assert!(!verify_token_hash("wrong-token", &stored));
    }

    #[test]
    fn test_generate_random_token_length() {
        let token = generate_random_token(32);
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_generate_random_token_uniqueness() {
        let token1 = generate_random_token(48);
        let token2 = generate_random_token(48);

        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_random_token_charset() {
        let token = generate_random_token(100);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}