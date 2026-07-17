#![allow(dead_code)]

use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::time::Duration;

/// Redis cache manager for performance optimization
pub struct CacheManager {
    pool: Pool,
    default_ttl: Duration,
}

impl CacheManager {
    /// Creates a new cache manager with Redis connection
    pub fn new(redis_url: &str, default_ttl: Duration) -> Result<Self, redis::RedisError> {
        let cfg = Config::from_url(redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1)).map_err(|e| {
            redis::RedisError::from((redis::ErrorKind::Io, "Failed to create pool", e.to_string()))
        })?;

        Ok(Self { pool, default_ttl })
    }

    /// Creates a new cache manager from an existing pool
    pub fn from_pool(pool: Pool, default_ttl: Duration) -> Self {
        Self { pool, default_ttl }
    }

    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let _span = tracing::debug_span!("redis.get", key = %key).entered();
        let mut conn = self.pool.get().await.ok()?;
        let data: Option<Vec<u8>> = conn.get(key).await.ok()?;

        match data {
            Some(bytes) => serde_json::from_slice(&bytes).ok(),
            None => None,
        }
    }

    /// Set a value in cache with default TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), CacheError> {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Set a value in cache with custom TTL
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        let _span =
            tracing::debug_span!("redis.set", key = %key, ttl_secs = ttl.as_secs()).entered();
        let data =
            serde_json::to_vec(value).map_err(|e| CacheError::Serialization(e.to_string()))?;

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| CacheError::Pool(e.to_string()))?;

        let _: () = conn
            .set_ex(key, data, ttl.as_secs())
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;

        Ok(())
    }

    /// Delete a key from cache
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let _span = tracing::debug_span!("redis.del", key = %key).entered();
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| CacheError::Pool(e.to_string()))?;

        let _: () = conn
            .del(key)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;

        Ok(())
    }

    /// Clear cache by pattern (use with caution)
    pub async fn clear_pattern(&self, pattern: &str) -> Result<(), CacheError> {
        let _span = tracing::info_span!("redis.keys_pattern", pattern = %pattern).entered();
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| CacheError::Pool(e.to_string()))?;

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;

        if !keys.is_empty() {
            let _: () = conn
                .del(&keys)
                .await
                .map_err(|e| CacheError::Redis(e.to_string()))?;
        }

        Ok(())
    }

    /// Increment a counter
    pub async fn increment(&self, key: &str, delta: i64) -> Result<i64, CacheError> {
        let _span = tracing::debug_span!("redis.incr", key = %key, delta = delta).entered();
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| CacheError::Pool(e.to_string()))?;

        let value: i64 = conn
            .incr(key, delta)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;

        Ok(value)
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let _span = tracing::debug_span!("redis.exists", key = %key).entered();
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| CacheError::Pool(e.to_string()))?;

        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;

        Ok(exists)
    }
}

/// Cache error types
#[derive(Debug)]
pub enum CacheError {
    Serialization(String),
    Pool(String),
    Redis(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            CacheError::Pool(msg) => write!(f, "Pool error: {}", msg),
            CacheError::Redis(msg) => write!(f, "Redis error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

/// Cache key generators for common entities
pub fn cache_key_user(user_id: &str) -> String {
    format!("user:{}", user_id)
}

pub fn cache_key_profile(profile_id: &str) -> String {
    format!("profile:{}", profile_id)
}

pub fn cache_key_creator_posts(creator_id: &str, page: i64) -> String {
    format!("posts:{}:{}", creator_id, page)
}

pub fn cache_key_feed(user_id: &str, page: i64) -> String {
    format!("feed:{}:{}", user_id, page)
}

pub fn cache_key_creator_list(filter: &str, page: i64) -> String {
    format!("creators:{}:{}", filter, page)
}

/// Type alias for shared cache manager
pub type SharedCache = Arc<CacheManager>;
