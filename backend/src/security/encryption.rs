use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;

use crate::errors::{AppError, AppResult};

pub fn encrypt(data: &[u8], key: &[u8]) -> AppResult<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| AppError::Internal("invalid encryption key length".to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::from(nonce_bytes);
    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|_| AppError::Internal("failed to encrypt data".to_string()))?;

    Ok([nonce_bytes.to_vec(), ciphertext].concat())
}

pub fn decrypt(data: &[u8], key: &[u8]) -> AppResult<Vec<u8>> {
    if data.len() < 12 {
        return Err(AppError::Internal(
            "encrypted payload is missing nonce".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce_bytes: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| AppError::Internal("encrypted payload has invalid nonce".to_string()))?;

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| AppError::Internal("invalid encryption key length".to_string()))?;
    let nonce = Nonce::from(nonce_bytes);

    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| AppError::Internal("failed to decrypt data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::{decrypt, encrypt};
    use crate::security::test_utils::test_key_manager;

    #[test]
    fn encrypt_decrypt_round_trip_works() {
        let key = test_key_manager().derive_encryption_key(1).unwrap();
        let plaintext = b"user@example.com";

        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_uses_random_nonce() {
        let key = test_key_manager().derive_encryption_key(1).unwrap();
        let plaintext = b"user@example.com";

        let first = encrypt(plaintext, &key).unwrap();
        let second = encrypt(plaintext, &key).unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn decrypt_fails_for_corrupted_payload() {
        let key = test_key_manager().derive_encryption_key(1).unwrap();
        let plaintext = b"user@example.com";
        let mut ciphertext = encrypt(plaintext, &key).unwrap();

        let last_index = ciphertext.len() - 1;
        ciphertext[last_index] ^= 0x01;

        assert!(decrypt(&ciphertext, &key).is_err());
    }
}
