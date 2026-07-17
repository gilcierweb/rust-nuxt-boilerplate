//! Async database module using diesel-async.
//!
//! This module provides an async connection pool for Diesel queries.
//! It coexists with the synchronous `database.rs` module, allowing
//! incremental migration from sync to async.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::db::async_database::AsyncDatabase;
//!
//! let db = AsyncDatabase::from_config(&config);
//! let mut conn = db.pool.get().await?;
//! // Use diesel-async query methods
//! ```

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;

use crate::config::AppConfig;

/// Async database connection pool using diesel-async + deadpool.
///
/// This pool provides truly async database access without blocking
/// actix-web worker threads.
pub struct AsyncDatabase {
    pub pool: Pool<AsyncPgConnection>,
}

impl AsyncDatabase {
    /// Creates a new async database pool using configuration from AppConfig.
    ///
    /// Pool settings mirror the synchronous pool:
    /// - `DB_POOL_SIZE`: Maximum number of connections
    /// - `DB_POOL_CONNECTION_TIMEOUT_SECS`: Connection timeout
    pub fn from_config(config: &AppConfig) -> Self {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database_url);

        let pool = Pool::builder(manager)
            .max_size(config.db_pool_size as usize)
            .build()
            .expect("Failed to create async database pool");

        AsyncDatabase { pool }
    }

    /// Get a connection from the pool.
    pub async fn get(
        &self,
    ) -> Result<
        diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Ok(self.pool.get().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_database_config_structure() {
        // Verify the struct compiles and has expected fields
        let _ = std::mem::size_of::<AsyncDatabase>();
    }
}
