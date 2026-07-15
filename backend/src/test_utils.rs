//! Test utilities for integration tests with real database.
//!
//! This module provides helpers for running integration tests against a real
//! PostgreSQL database. It requires DATABASE_URL_TEST environment variable to be set.
//!
//! Before running tests, ensure migrations are up to date:
//! ```bash
//! diesel migration run --database-url $DATABASE_URL_TEST
//! ```
//!
//! For containerized tests in CI, consider using testcontainers in the future.

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use diesel::RunQueryDsl;
use std::sync::Arc;

/// Test database context for integration tests.
pub struct TestDb {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl TestDb {
    /// Create a new test database connection from DATABASE_URL_TEST.
    pub fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Arc::new(
            Pool::builder()
                .max_size(4)
                .build(manager)
                .expect("Failed to create pool"),
        );

        Self { pool }
    }

    /// Get a connection from the pool.
    pub fn conn(&self) -> diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().expect("Failed to get connection")
    }

    /// Get the connection pool.
    pub fn pool(&self) -> Arc<Pool<ConnectionManager<PgConnection>>> {
        self.pool.clone()
    }

    /// Drop all tables (for test cleanup).
    pub fn drop_all_tables(&self) {
        let mut conn = self.conn();
        diesel::sql_query("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
            .execute(&mut conn)
            .expect("Failed to drop all tables");
    }
}

impl Default for TestDb {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to run a test with a fresh database.
pub fn with_test_db<F>(test_fn: F)
where
    F: FnOnce(&TestDb),
{
    let db = TestDb::new();
    test_fn(&db);
    db.drop_all_tables();
}

/// Generate deterministic test data for reproducibility.
pub mod test_data {
    use uuid::Uuid;

    /// Generate a deterministic UUID from a seed.
    pub fn uuid_from_seed(seed: u64) -> Uuid {
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&seed.to_le_bytes());
        bytes[8..].copy_from_slice(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48]);
        Uuid::from_bytes(bytes)
    }

    /// Generate a deterministic email for testing.
    pub fn email(index: u32) -> String {
        format!("user{}@example.com", index)
    }

    /// Generate a deterministic password for testing.
    pub fn password() -> String {
        "TestPassword123!".to_string()
    }
}