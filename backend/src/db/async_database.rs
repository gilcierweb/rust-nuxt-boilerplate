//! Async database module — Migration path documentation.
//!
//! This module documents the migration path from synchronous Diesel (r2d2)
//! to async Diesel (diesel-async). The migration is NOT yet implemented
//! because diesel-async 0.5.x requires diesel ~2.2.0, which conflicts with
//! our diesel 2.3.7 dependency.
//!
//! # Current State
//!
//! The application uses synchronous Diesel with r2d2 connection pool.
//! All database queries block the actix-web worker threads, which can
//! reduce throughput under high concurrency.
//!
//! # Migration Path
//!
//! When diesel-async releases a version compatible with diesel 2.3+:
//!
//! 1. Add `diesel-async` to Cargo.toml:
//!    ```toml
//!    diesel-async = { version = "0.6", features = ["postgres", "deadpool"] }
//!    ```
//!
//! 2. Create `db/async_database.rs` with:
//!    ```rust,ignore
//!    use diesel_async::pg::AsyncPgConnection;
//!    use diesel_async::pooled_connection::deadpool::Pool;
//!
//!    pub struct AsyncDatabase {
//!        pub pool: Pool<AsyncPgConnection>,
//!    }
//!    ```
//!
//! 3. Migrate repositories one at a time:
//!    - Add async methods alongside existing sync methods
//!    - Use `#[cfg(feature = "async-db")]` to switch implementations
//!    - Remove sync methods after full migration
//!
//! 4. Update controllers to use async pool:
//!    ```rust,ignore
//!    // Before (sync)
//!    let mut conn = pool.get()?;
//!    users.find(id).first(&mut conn)
//!
//!    // After (async)
//!    let mut conn = pool.get().await?;
//!    users.find(id).first(&mut conn).await
//!    ```
//!
//! # Benefits of Async
//!
//! - Non-blocking database I/O
//! - Better utilization of actix-web worker threads
//! - Higher throughput under concurrent load
//! - Native async/await syntax
//!
//! # Current Workaround
//!
//! The synchronous pool is acceptable for current workload levels.
//! Monitor `db_pool_size` and connection wait times. If connection
//! exhaustion becomes an issue, prioritize the async migration.
//!
//! For now, the `AccessTokenBlacklist` uses Redis (already async via
//! deadpool-redis), so auth-related operations are non-blocking.

/// Placeholder for future async database implementation.
///
/// When diesel-async becomes compatible with diesel 2.3+, this module
/// will contain the `AsyncDatabase` struct and connection pool setup.
pub struct AsyncDatabasePlaceholder;
