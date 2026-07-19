// Unit tests for auth utilities
#[cfg(test)]
mod tests {
    use crate::repositories::test_utils::mocks::mock_app_config;
    use crate::services::auth_service::{hash_password, verify_password};

    #[actix_rt::test]
    async fn test_password_hashing() {
        let config = mock_app_config();
        let password = "test_password123";
        let hash = hash_password(password, &config).expect("Failed to hash password");

        // Verify correct password
        assert!(verify_password(password, &hash).expect("Failed to verify password"));

        // Verify wrong password fails
        assert!(!verify_password("wrong_password", &hash).expect("Failed to verify password"));
    }

    #[actix_rt::test]
    async fn test_password_hash_unique() {
        let config = mock_app_config();
        let password = "same_password";
        let hash1 = hash_password(password, &config).expect("Failed to hash password");
        let hash2 = hash_password(password, &config).expect("Failed to hash password");

        // Same password should produce different hashes (due to salt)
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(verify_password(password, &hash1).expect("Failed to verify password"));
        assert!(verify_password(password, &hash2).expect("Failed to verify password"));
    }
}
