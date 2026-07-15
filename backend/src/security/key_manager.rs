use hmac::{Mac, SimpleHmac};
use sha2::{Digest, Sha256};
use std::env;

use crate::errors::{AppError, AppResult};

#[derive(Debug)]
pub struct KeyManager {
    master_key: Vec<u8>,
    blind_index_key: Vec<u8>,
}

impl KeyManager {
    pub fn new() -> AppResult<Self> {
        let master_key = env::var("MASTER_KEY")
            .map_err(|_| AppError::Internal("MASTER_KEY missing".to_string()))?;
        let blind_index_key = env::var("BLIND_INDEX_KEY")
            .map_err(|_| AppError::Internal("BLIND_INDEX_KEY missing".to_string()))?;

        Self::from_base64_keys(&master_key, &blind_index_key)
    }

    pub fn from_base64_keys(master_key: &str, blind_index_key: &str) -> AppResult<Self> {
        use base64::Engine;

        let master_key = base64::engine::general_purpose::STANDARD
            .decode(master_key)
            .map_err(|_| AppError::Internal("MASTER_KEY is not valid base64".to_string()))?;
        let blind_index_key = base64::engine::general_purpose::STANDARD
            .decode(blind_index_key)
            .map_err(|_| AppError::Internal("BLIND_INDEX_KEY is not valid base64".to_string()))?;

        if master_key.len() < 32 {
            return Err(AppError::Internal(
                "MASTER_KEY must decode to at least 32 bytes".to_string(),
            ));
        }

        if blind_index_key.len() < 32 {
            return Err(AppError::Internal(
                "BLIND_INDEX_KEY must decode to at least 32 bytes".to_string(),
            ));
        }

        Ok(Self {
            master_key,
            blind_index_key,
        })
    }

    pub fn derive_encryption_key(&self, version: u32) -> AppResult<[u8; 32]> {
        let info = format!("encryption:v{}", version);
        let key = Sha256::digest(&self.master_key);
        let mut mac = SimpleHmac::<Sha256>::new_from_slice(&key)
            .map_err(|_| AppError::Internal("invalid key".to_string()))?;
        mac.update(info.as_bytes());
        mac.update(&version.to_le_bytes());
        let result = mac.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result.into_bytes()[..32]);
        Ok(out)
    }

    pub fn blind_index_key(&self) -> &[u8] {
        &self.blind_index_key
    }
}

#[cfg(test)]
mod tests {
    use super::KeyManager;

    fn valid_master_key() -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&[0u8; 32])
    }

    fn valid_blind_index_key() -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&[1u8; 32])
    }

    #[test]
    fn invalid_base64_key_is_rejected() {
        let result = KeyManager::from_base64_keys(
            "not-base64",
            &valid_blind_index_key(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn valid_keys_are_accepted() {
        let result = KeyManager::from_base64_keys(&valid_master_key(), &valid_blind_index_key());
        assert!(result.is_ok());
    }

    #[test]
    fn short_master_key_is_rejected() {
        use base64::Engine;
        let short_key = base64::engine::general_purpose::STANDARD.encode(&[0u8; 16]);
        let result = KeyManager::from_base64_keys(&short_key, &valid_blind_index_key());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("at least 32 bytes"));
    }

    #[test]
    fn short_blind_index_key_is_rejected() {
        use base64::Engine;
        let short_key = base64::engine::general_purpose::STANDARD.encode(&[0u8; 16]);
        let result = KeyManager::from_base64_keys(&valid_master_key(), &short_key);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("at least 32 bytes"));
    }

    #[test]
    fn derive_encryption_key_deterministic() {
        let km = KeyManager::from_base64_keys(&valid_master_key(), &valid_blind_index_key()).unwrap();
        let key1 = km.derive_encryption_key(1).unwrap();
        let key2 = km.derive_encryption_key(1).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn derive_encryption_key_different_versions_differ() {
        let km = KeyManager::from_base64_keys(&valid_master_key(), &valid_blind_index_key()).unwrap();
        let key_v1 = km.derive_encryption_key(1).unwrap();
        let key_v2 = km.derive_encryption_key(2).unwrap();
        assert_ne!(key_v1, key_v2);
    }

    #[test]
    fn derive_encryption_key_different_master_keys_differ() {
        use base64::Engine;
        let master1 = base64::engine::general_purpose::STANDARD.encode(&[0u8; 32]);
        let master2 = base64::engine::general_purpose::STANDARD.encode(&[1u8; 32]);
        let bik = valid_blind_index_key();

        let km1 = KeyManager::from_base64_keys(&master1, &bik).unwrap();
        let km2 = KeyManager::from_base64_keys(&master2, &bik).unwrap();

        let key1 = km1.derive_encryption_key(1).unwrap();
        let key2 = km2.derive_encryption_key(1).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn blind_index_key_accessor_returns_correct_key() {
        let km = KeyManager::from_base64_keys(&valid_master_key(), &valid_blind_index_key()).unwrap();
        use base64::Engine;
        let expected = base64::engine::general_purpose::STANDARD
            .decode(valid_blind_index_key())
            .unwrap();
        assert_eq!(km.blind_index_key(), expected.as_slice());
    }
}
