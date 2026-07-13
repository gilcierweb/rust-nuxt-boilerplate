#![allow(dead_code)]

use crate::{
    config::AppConfig,
    errors::{AppError, AppResult},
    models::user::User,
};

use super::{
    blind_index::blind_index,
    encryption::{decrypt, encrypt},
    key_manager::KeyManager,
    normalization::{normalize_cpf, normalize_email, normalize_phone},
};

#[derive(Debug, Clone)]
pub struct ProtectedValue {
    pub blind_index: Vec<u8>,
    pub encrypted: Vec<u8>,
    pub key_version: i32,
}

pub struct SecurityService {
    key_manager: KeyManager,
    current_key_version: u32,
}

impl SecurityService {
    pub fn from_config(config: &AppConfig) -> AppResult<Self> {
        Ok(Self {
            key_manager: KeyManager::from_base64_keys(&config.master_key, &config.blind_index_key)?,
            current_key_version: config.current_encryption_key_version,
        })
    }

    pub fn from_env() -> AppResult<Self> {
        Ok(Self {
            key_manager: KeyManager::new()?,
            current_key_version: std::env::var("CURRENT_ENCRYPTION_KEY_VERSION")
                .ok()
                .and_then(|value| value.parse::<u32>().ok())
                .unwrap_or(1),
        })
    }

    pub fn normalize_email(&self, email: &str) -> String {
        normalize_email(email)
    }

    pub fn normalize_cpf(&self, cpf: &str) -> String {
        normalize_cpf(cpf)
    }

    pub fn normalize_phone(&self, phone: &str) -> String {
        normalize_phone(phone)
    }

    pub fn protect_email(&self, email: &str) -> AppResult<ProtectedValue> {
        let normalized = self.normalize_email(email);
        self.protect_normalized_value(&normalized)
    }

    pub fn protect_cpf(&self, cpf: &str) -> AppResult<ProtectedValue> {
        let normalized = self.normalize_cpf(cpf);
        self.protect_normalized_value(&normalized)
    }

    pub fn protect_phone(&self, phone: &str) -> AppResult<ProtectedValue> {
        let normalized = self.normalize_phone(phone);
        self.protect_normalized_value(&normalized)
    }

    #[allow(dead_code)]
    pub fn encrypt_optional_field(&self, value: Option<&str>) -> AppResult<Option<Vec<u8>>> {
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => {
                let encryption_key = self
                    .key_manager
                    .derive_encryption_key(self.current_key_version)?;
                encrypt(value.as_bytes(), &encryption_key).map(Some)
            }
            None => Ok(None),
        }
    }

    pub fn decrypt_optional_field(
        &self,
        encrypted: Option<&[u8]>,
        key_version: i32,
    ) -> AppResult<Option<String>> {
        let Some(encrypted) = encrypted else {
            return Ok(None);
        };

        let encryption_key = self
            .key_manager
            .derive_encryption_key(key_version.max(1) as u32)?;
        let plaintext = decrypt(encrypted, &encryption_key)?;

        String::from_utf8(plaintext)
            .map(Some)
            .map_err(|_| AppError::Internal("decrypted field is not valid UTF-8".to_string()))
    }

    fn protect_normalized_value(&self, normalized: &str) -> AppResult<ProtectedValue> {
        let blind_index = blind_index(normalized, self.key_manager.blind_index_key())?;
        let encryption_key = self
            .key_manager
            .derive_encryption_key(self.current_key_version)?;
        let encrypted = encrypt(normalized.as_bytes(), &encryption_key)?;

        Ok(ProtectedValue {
            blind_index,
            encrypted,
            key_version: self.current_key_version as i32,
        })
    }

    pub fn decrypt_user_email(&self, user: &User) -> AppResult<String> {
        let key_version = user.encryption_key_version.max(1) as u32;
        let encryption_key = self.key_manager.derive_encryption_key(key_version)?;
        let plaintext = decrypt(&user.email_encrypted, &encryption_key)?;

        String::from_utf8(plaintext)
            .map_err(|_| AppError::Internal("decrypted email is not valid UTF-8".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::SecurityService;
    use crate::security::test_utils::test_config;

    #[test]
    fn email_blind_index_is_deterministic() {
        let security = SecurityService::from_config(&test_config()).unwrap();
        let first = security.protect_email(" User@Example.com ").unwrap();
        let second = security.protect_email("user@example.com").unwrap();

        assert_eq!(first.blind_index, second.blind_index);
        assert_ne!(first.encrypted, second.encrypted);
    }

    #[test]
    fn different_emails_generate_different_blind_indexes() {
        let security = SecurityService::from_config(&test_config()).unwrap();
        let first = security.protect_email("alice@example.com").unwrap();
        let second = security.protect_email("bob@example.com").unwrap();

        assert_ne!(first.blind_index, second.blind_index);
    }

    #[test]
    fn cpf_blind_index_is_deterministic_after_normalization() {
        let security = SecurityService::from_config(&test_config()).unwrap();
        let first = security.protect_cpf("123.456.789-00").unwrap();
        let second = security.protect_cpf("12345678900").unwrap();

        assert_eq!(first.blind_index, second.blind_index);
        assert_ne!(first.encrypted, second.encrypted);
    }

    #[test]
    fn phone_blind_index_is_deterministic_after_normalization() {
        let security = SecurityService::from_config(&test_config()).unwrap();
        let first = security.protect_phone("+55 (11) 99999-0000").unwrap();
        let second = security.protect_phone("5511999990000").unwrap();

        assert_eq!(first.blind_index, second.blind_index);
        assert_ne!(first.encrypted, second.encrypted);
    }

    #[test]
    fn optional_field_encrypt_decrypt_round_trip_works() {
        let security = SecurityService::from_config(&test_config()).unwrap();

        let encrypted = security
            .encrypt_optional_field(Some("  +55 11 99999-0000  "))
            .unwrap();
        let decrypted = security
            .decrypt_optional_field(encrypted.as_deref(), 1)
            .unwrap();

        assert_eq!(decrypted.as_deref(), Some("+55 11 99999-0000"));
    }
}
