use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::errors::{AppError, AppResult};

type HmacSha256 = Hmac<Sha256>;

pub fn blind_index(value: &str, key: &[u8]) -> AppResult<Vec<u8>> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| AppError::Internal("invalid blind index key".to_string()))?;
    mac.update(value.as_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}
