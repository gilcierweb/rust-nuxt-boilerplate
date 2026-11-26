#![allow(dead_code)]

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

use crate::config::AppConfig;
use crate::errors::AppError;
use rust_i18n::t;

/// Hash a plaintext password using Argon2id with configurable parameters.
pub fn hash_password(password: &str, config: &AppConfig) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let params = argon2::Params::new(
        config.argon2_m_cost,
        config.argon2_t_cost,
        config.argon2_p_cost,
        None,
    )
    .map_err(|e| AppError::Internal(format!("Invalid Argon2 parameters: {}", e)))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Password hash error: {}", e)))
}

/// Verify a plaintext password against an Argon2id hash.
/// Uses the parameters encoded in the hash itself.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Password parse error: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// A throwaway Argon2id hash built with the configured cost parameters.
pub fn dummy_password_hash(config: &AppConfig) -> String {
    static CACHE: DummyPasswordHashCache = DummyPasswordHashCache::new();

    let params = (
        config.argon2_m_cost,
        config.argon2_t_cost,
        config.argon2_p_cost,
    );

    CACHE
        .get_or_init(|| parking_lot::Mutex::new(std::collections::HashMap::new()))
        .lock()
        .entry(params)
        .or_insert_with(|| {
            hash_password(DUMMY_PASSWORD, config).expect("dummy hash must be derivable")
        })
        .clone()
}

/// Dummy password hashes keyed by the Argon2id cost tuple they were built with.
type DummyPasswordHashCache =
    std::sync::OnceLock<parking_lot::Mutex<std::collections::HashMap<(u32, u32, u32), String>>>;

/// The plaintext hashed by [`dummy_password_hash`]. It is never used to
/// authenticate anything; only its hash exists, and only for timing.
const DUMMY_PASSWORD: &str = "timing-equalisation-placeholder-password";

/// Check if a password hash needs to be rehashed with current parameters.
pub fn needs_rehash(hash: &str, config: &AppConfig) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return true,
    };

    let params = &parsed.params;

    let m_cost = params
        .get("m")
        .and_then(|v| v.to_string().parse::<u32>().ok())
        .unwrap_or(0);
    let t_cost = params
        .get("t")
        .and_then(|v| v.to_string().parse::<u32>().ok())
        .unwrap_or(0);
    let p_cost = params
        .get("p")
        .and_then(|v| v.to_string().parse::<u32>().ok())
        .unwrap_or(0);

    m_cost != config.argon2_m_cost
        || t_cost != config.argon2_t_cost
        || p_cost != config.argon2_p_cost
}

/// Rehash a password with current parameters. Returns the new hash.
pub fn rehash_password(password: &str, config: &AppConfig) -> Result<String, AppError> {
    hash_password(password, config)
}

/// Validate password strength: min 12 chars, at least 1 digit, 1 uppercase, 1 special char.
pub fn validate_password_strength(password: &str, locale: &str) -> Result<(), AppError> {
    if password.len() < 12 {
        return Err(AppError::Validation(
            t!("auth.password.must_be_12_chars", locale = locale).into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation(
            t!("auth.password.must_have_number", locale = locale).into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(AppError::Validation(
            t!("auth.password.must_have_uppercase", locale = locale).into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_punctuation()) {
        return Err(AppError::Validation(
            t!("auth.password.must_have_special", locale = locale).into_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::mocks::mock_app_config;
    use crate::services::token::generate_random_token;
    use crate::services::token_service::hash_token;

    #[test]
    fn test_hash_password_creates_valid_hash() {
        let password = "TestPassword123!";
        let config = mock_app_config();
        let hash = hash_password(password, &config).expect("Failed to hash password");

        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "TestPassword123!";
        let config = mock_app_config();
        let hash = hash_password(password, &config).expect("Failed to hash password");
        let result = verify_password(password, &hash).expect("Failed to verify password");
        assert!(result);
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "TestPassword123!";
        let wrong_password = "WrongPassword123!";
        let config = mock_app_config();
        let hash = hash_password(password, &config).expect("Failed to hash password");
        let result = verify_password(wrong_password, &hash).expect("Failed to verify password");
        assert!(!result);
    }

    #[test]
    fn test_verify_password_wrong_hash() {
        let password = "TestPassword123!";
        let wrong_hash = "$argon2id$v=19$m=19456,t=2,p=1$wrongsalt$wronghash";
        let result = verify_password(password, wrong_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_valid() {
        let password = "ValidPass123!";
        let result = validate_password_strength(password, "pt-BR");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        let password = "Short1!";
        let result = validate_password_strength(password, "pt-BR");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_digit() {
        let password = "NoDigitPass!";
        let result = validate_password_strength(password, "pt-BR");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_uppercase() {
        let password = "nouppercase1!";
        let result = validate_password_strength(password, "pt-BR");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_special() {
        let password = "NoSpecial123";
        let result = validate_password_strength(password, "pt-BR");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_random_token() {
        let token1 = generate_random_token(32);
        let token2 = generate_random_token(32);
        assert_eq!(token1.len(), 32);
        assert_eq!(token2.len(), 32);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_hash_token_consistency() {
        use crate::services::token_service::verify_token_hash;
        let token = "test-token-123";
        let key = "test_key";
        let hash1 = hash_token(token, key);
        let hash2 = hash_token(token, key);
        assert_eq!(hash1, hash2);
        assert!(verify_token_hash(token, &hash1, key));
        assert!(verify_token_hash(token, &hash2, key));
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let key = "test_key";
        let hash1 = hash_token("token-1", key);
        let hash2 = hash_token("token-2", key);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_different_keys() {
        let token = "test-token-123";
        let hash1 = hash_token(token, "key1");
        let hash2 = hash_token(token, "key2");
        assert_ne!(hash1, hash2);
    }
}

#[cfg(test)]
mod dummy_hash_tests {
    use super::{dummy_password_hash, verify_password};
    use crate::repositories::mocks::mock_app_config;

    fn config_with_costs(m_cost: u32, t_cost: u32, p_cost: u32) -> crate::config::AppConfig {
        let mut config = mock_app_config();
        config.argon2_m_cost = m_cost;
        config.argon2_t_cost = t_cost;
        config.argon2_p_cost = p_cost;
        config
    }

    #[test]
    fn dummy_hash_encodes_the_configured_cost() {
        let config = config_with_costs(64, 4, 2);
        let hash = dummy_password_hash(&config);

        let params_segment = hash
            .split('$')
            .nth(3)
            .unwrap_or_else(|| panic!("missing parameters in {hash}"));
        assert_eq!(params_segment, "m=64,t=4,p=2", "hash: {hash}");
        assert!(
            hash.starts_with("$argon2id$"),
            "dummy hash must use the same algorithm: {hash}"
        );
    }

    #[test]
    fn dummy_hash_never_authenticates_a_candidate() {
        let config = mock_app_config();
        let hash = dummy_password_hash(&config);

        assert!(!verify_password("any", &hash).expect("verification runs"));
        assert!(!verify_password(&hash, &hash).expect("verification runs"));
    }

    #[test]
    fn dummy_hash_is_cached_per_cost_parameters() {
        let config = config_with_costs(64, 3, 1);
        let other = config_with_costs(128, 3, 1);

        assert_eq!(dummy_password_hash(&config), dummy_password_hash(&config));
        assert_eq!(dummy_password_hash(&other), dummy_password_hash(&other));
        assert_ne!(dummy_password_hash(&config), dummy_password_hash(&other));
    }
}
