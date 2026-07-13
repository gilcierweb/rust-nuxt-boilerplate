// Unit tests for auth utilities
#[cfg(test)]
mod tests {
    use crate::auth::password::{password_hash, verify};

    #[actix_rt::test]
    async fn test_password_hashing() {
        let password = "test_password123";
        let hash = password_hash(password.to_string());

        // Verify correct password
        assert!(verify(password.to_string(), hash.clone()));

        // Verify wrong password fails
        assert!(!verify("wrong_password".to_string(), hash));
    }

    #[actix_rt::test]
    async fn test_password_hash_unique() {
        let password = "same_password";
        let hash1 = password_hash(password.to_string());
        let hash2 = password_hash(password.to_string());

        // Same password should produce different hashes (due to salt)
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(verify(password.to_string(), hash1));
        assert!(verify(password.to_string(), hash2));
    }
}
