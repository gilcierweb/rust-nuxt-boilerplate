use deadpool_redis::Pool;
use redis::AsyncCommands;
use uuid::Uuid;

pub struct AccessTokenBlacklist {
    redis: Pool,
    prefix: String,
}

/// Redis key prefix for per-user credential-epoch markers.
///
/// Deliberately separate from the blacklist `prefix` so the SCAN-based
/// `count()` / `cleanup_expired()` helpers stay scoped to blacklist keys.
const EPOCH_PREFIX: &str = "credential_epoch:";

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

/// Check whether a JWT issued at `iat` (unix seconds) predates the user's
/// credential epoch (the unix timestamp of their last password/2FA change).
///
/// Tokens issued strictly before the epoch are considered revoked: whoever
/// holds them acquired them before the credential change that invalidated
/// every prior session. A token issued at the exact epoch second is still
/// valid (it postdates the change).
pub fn is_token_revoked_by_epoch(iat: usize, epoch: Option<u64>) -> bool {
    epoch.is_some_and(|e| (iat as u64) < e)
}

impl AccessTokenBlacklist {
    /// Record a user's credential epoch: the timestamp of their last
    /// password/2FA change. All access tokens with `iat` below this value
    /// are rejected by the JWT middleware until they naturally expire.
    ///
    /// The TTL matches the access-token lifetime: once every pre-change
    /// token has expired, the marker is no longer needed, so Redis drops
    /// it automatically.
    pub async fn set_credential_epoch(
        &self,
        user_id: &Uuid,
        ttl_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis.get().await?;
        let key = format!("{}{}", EPOCH_PREFIX, user_id);
        let now = chrono::Utc::now().timestamp() as u64;
        redis::cmd("SET")
            .arg(&key)
            .arg(now)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Read a user's credential epoch, if one is recorded (and not yet
    /// expired). `Ok(None)` means no credential change is recent enough
    /// to invalidate any live access token.
    pub async fn credential_epoch(
        &self,
        user_id: &Uuid,
    ) -> Result<Option<u64>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis.get().await?;
        let key = format!("{}{}", EPOCH_PREFIX, user_id);
        let value: Option<String> = conn.get(&key).await?;
        Ok(value.and_then(|v| v.parse::<u64>().ok()))
    }
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

    #[test]
    fn test_is_token_revoked_by_epoch_true_cases() {
        // Token issued strictly before the epoch → revoked.
        assert!(is_token_revoked_by_epoch(100, Some(200)));
        assert!(is_token_revoked_by_epoch(100, Some(101)));
    }

    #[test]
    fn test_is_token_revoked_by_epoch_false_cases() {
        // No epoch recorded → nothing is revoked.
        assert!(!is_token_revoked_by_epoch(100, None));
        // Token issued at or after the epoch → still valid.
        assert!(!is_token_revoked_by_epoch(200, Some(200)));
        assert!(!is_token_revoked_by_epoch(300, Some(200)));
    }
}
