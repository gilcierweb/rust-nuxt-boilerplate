use super::key_manager::KeyManager;
use crate::config::test_utils::test_config;

pub fn test_key_manager() -> KeyManager {
    let config = test_config();

    KeyManager::from_base64_keys(&config.master_key, &config.blind_index_key).unwrap()
}
