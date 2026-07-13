use hmac::{Mac, SimpleHmac};
use sha2::{Digest, Sha256};
use std::env;

use crate::errors::{AppError, AppResult};

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

    #[test]
    fn invalid_base64_key_is_rejected() {
        let result = KeyManager::from_base64_keys(
            "not-base64",
            "ZmVkY2JhOTg3NjU0MzIxMGZlZGNiYTk4NzY1NDMyMTA=",
        );

        assert!(result.is_err());
    }
}
