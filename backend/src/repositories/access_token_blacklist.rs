use deadpool_redis::Pool;
use redis::AsyncCommands;

pub struct AccessTokenBlacklist {
    redis: Pool,
    prefix: String,
}

impl AccessTokenBlacklist {
    pub fn new(redis: Pool) -> Self {
        Self {
            redis,
            prefix: "access_token_blacklist:".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn new_with_prefix(redis: Pool, prefix: String) -> Self {
        Self { redis, prefix }
    }

    /// Add a token to the blacklist with TTL matching the token's remaining lifetime
    pub async fn add(&self, token_hash: &str, ttl_seconds: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis.get().await?;
        let key = format!("{}{}", self.prefix, token_hash);
        redis::cmd("SET")
            .arg(&key)
            .arg("1")
            .arg("EX")
            .arg(ttl_seconds)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Check if a token is blacklisted
    pub async fn is_blacklisted(&self, token_hash: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis.get().await?;
        let key = format!("{}{}", self.prefix, token_hash);
        let exists: bool = conn.exists(&key).await?;
        Ok(exists)
    }

    #[allow(dead_code)]
    pub async fn remove(&self, token_hash: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis.get().await?;
        let key = format!("{}{}", self.prefix, token_hash);
        let _: usize = conn.del(&key).await?;
        Ok(())
    }
}

/// Helper to hash a token for storage in blacklist

/// Helper to hash a token for storage in blacklist
pub fn hash_token_for_blacklist(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token_for_blacklist() {
        let token = "test-token-123";
        let hash = hash_token_for_blacklist(token);
        assert_eq!(hash.len(), 64); // SHA256 hex = 64 chars
        assert_eq!(hash, hash_token_for_blacklist(token)); // Deterministic
    }
}