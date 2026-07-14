use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::config::AppConfig;

pub type DBPool = Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    pub pool: DBPool,
}

impl Database {
    /// Creates a new database pool using configuration from AppConfig.
    /// 
    /// Pool settings can be tuned via environment variables:
    /// - `DB_POOL_SIZE`: Maximum number of connections (default: 10)
    /// - `DB_POOL_MIN_IDLE`: Minimum idle connections (default: 2)
    /// - `DB_POOL_MAX_LIFETIME_SECS`: Max connection lifetime (default: 30 mins)
    /// - `DB_POOL_IDLE_TIMEOUT_SECS`: Idle connection timeout (default: 10 mins)
    /// - `DB_POOL_CONNECTION_TIMEOUT_SECS`: Connection timeout (default: 10 secs)
    pub fn from_config(config: &AppConfig) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
        
        let mut pool_builder = Pool::builder()
            .max_size(config.db_pool_size)
            .connection_timeout(std::time::Duration::from_secs(
                config.db_pool_connection_timeout_secs,
            ));

        // Set min idle if configured
        if let Some(min_idle) = config.db_pool_min_idle {
            pool_builder = pool_builder.min_idle(Some(min_idle));
        } else {
            // Default: 20% of pool size or 2, whichever is smaller
            let default_min_idle = std::cmp::min(2, config.db_pool_size / 5);
            pool_builder = pool_builder.min_idle(Some(default_min_idle));
        }

        // Set max lifetime if configured
        if let Some(max_lifetime) = config.db_pool_max_lifetime_secs {
            pool_builder = pool_builder.max_lifetime(Some(std::time::Duration::from_secs(max_lifetime)));
        } else {
            // Default: 30 minutes
            pool_builder = pool_builder.max_lifetime(Some(std::time::Duration::from_secs(1800)));
        }

        // Set idle timeout if configured
        if let Some(idle_timeout) = config.db_pool_idle_timeout_secs {
            pool_builder = pool_builder.idle_timeout(Some(std::time::Duration::from_secs(idle_timeout)));
        } else {
            // Default: 10 minutes
            pool_builder = pool_builder.idle_timeout(Some(std::time::Duration::from_secs(600)));
        }

        let result = pool_builder
            .build(manager)
            .expect("Failed to create database pool");

        Database { pool: result }
    }

    /// Creates a new database pool using environment variables.
    #[deprecated(since = "0.2.0", note = "Use `from_config` instead")]
    #[allow(dead_code)]
    pub fn new() -> Self {
        let config = AppConfig::from_env().expect("Failed to load configuration");
        Self::from_config(&config)
    }
}
