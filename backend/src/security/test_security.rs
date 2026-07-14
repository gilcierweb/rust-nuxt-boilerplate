use crate::{
    errors::AppError,
    security::{encryption::encrypt_value, blind_index::create_blind_index, key_manager::KeyManager},
};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_and_decrypt_value() {
        let key_manager = KeyManager::new(
            "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=", // base64 encoded 32-byte key
            "ZmVkY2JhOTg3NjU0MzIxMGZlZGNiYTk4NzY1NDMyMTA=", // base64 encoded 32-byte blind index key
            1,
        );

        let plaintext = "sensitive_data_123";
        let encrypted = encrypt_value(plaintext, &key_manager).expect("Failed to encrypt");
        assert_ne!(encrypted.ciphertext, plaintext);
        assert_eq!(encrypted.key_version, 1);
        assert!(!encrypted.iv.is_empty());

        let decrypted = encrypt_value(&encrypted.ciphertext, &key_manager)
            .expect("Failed to decrypt")
            .ciphertext;
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_create_blind_index() {
        let key_manager = KeyManager::new(
            "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=",
            "ZmVkY2JhOTg3NjU0MzIxMGZlZGNiYTk4NzY1NDMyMTA=",
            1,
        );

        let value = "test@example.com";
        let blind_index = create_blind_index(value, &key_manager);
        assert_eq!(blind_index.key_version, 1);
        assert_eq!(blind_index.value.len(), 32); // HMAC-SHA256 produces 32 bytes
    }

    #[test]
    fn test_blind_index_deterministic() {
        let key_manager = KeyManager::new(
            "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=",
            "ZmVkY2JhOTg3NjU0MzIxMGZlZGNiYTk4NzY1NDMyMTA=",
            1,
        );

        let value1 = "test@example.com";
        let value2 = "test@example.com";
        let value3 = "different@test.com";

        let blind1 = create_blind_index(value1, &key_manager);
        let blind2 = create_blind_index(value2, &key_manager);
        let blind3 = create_blind_index(value3, &key_manager);

        assert_eq!(blind1.value, blind2.value);
        assert_ne!(blind1.value, blind3.value);
    }
}