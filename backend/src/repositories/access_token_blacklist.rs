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

    /// Add a token to the blacklist with TTL matching the token's remaining lifetime.
    ///
    /// The TTL is critical — Redis automatically expires keys after the specified duration.
    /// Always set TTL to prevent unbounded memory growth.
    pub async fn add(
        &self,
        token_hash: &str,
        ttl_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    pub async fn is_blacklisted(
        &self,
        token_hash: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis.get().await?;
        let key = format!("{}{}", self.prefix, token_hash);
        let exists: bool = conn.exists(&key).await?;
        Ok(exists)
    }

    #[allow(dead_code)]
    pub async fn remove(
        &self,
        token_hash: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis.get().await?;
        let key = format!("{}{}", self.prefix, token_hash);
        let _: usize = conn.del(&key).await?;
        Ok(())
    }

    /// Count the number of blacklisted tokens currently in Redis.
    ///
    /// Uses SCAN to iterate keys with the blacklist prefix. This is O(N) but
    /// non-blocking — Redis SCAN does not block the server.
    pub async fn count(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis.get().await?;
        let pattern = format!("{}*", self.prefix);
        let mut count = 0usize;
        let mut cursor = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;

            count += keys.len();
            cursor = new_cursor;

            if cursor == 0 {
                break;
            }
        }

        Ok(count)
    }

    /// Remove expired blacklist entries.
    ///
    /// Redis automatically expires keys via TTL, but this method provides explicit
    /// cleanup as a safety net for edge cases:
    /// - TTL was not set correctly during `add()`
    /// - Redis restarted without persistence and lost TTL metadata
    /// - Manual inspection/cleanup needed
    ///
    /// Uses SCAN to find keys and checks TTL. Only removes keys that are expired
    /// or have no TTL set (which indicates a bug).
    pub async fn cleanup_expired(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis.get().await?;
        let pattern = format!("{}*", self.prefix);
        let mut removed = 0usize;
        let mut cursor = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;

            for key in &keys {
                // Check TTL — -1 means no expiry set (bug), -2 means key doesn't exist
                let ttl: i64 = redis::cmd("TTL").arg(key).query_async(&mut conn).await?;

                // Remove if no TTL set (should never happen) or already expired
                if ttl == -1 || ttl == 0 {
                    let _: usize = conn.del(key.as_str()).await?;
                    removed += 1;
                }
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(removed)
    }
}

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

    #[test]
    fn test_hash_token_different_inputs_differ() {
        let hash1 = hash_token_for_blacklist("token-a");
        let hash2 = hash_token_for_blacklist("token-b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_is_hex_encoded() {
        let hash = hash_token_for_blacklist("test");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
