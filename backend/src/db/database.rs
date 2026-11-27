use std::time::Duration;

use deadpool::managed::{Pool, Timeouts};
use diesel::result::QueryResult;
use diesel_async::AsyncConnection;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use futures::future::BoxFuture;
use futures_util::FutureExt;
use tracing::Instrument;

use crate::config::AppConfig;

pub type DBPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub struct Database {
    pub pool: DBPool,
}

impl Database {
    pub fn from_config(config: &AppConfig) -> Self {
        let timeout_secs = config.db_statement_timeout_secs;

        let mut manager_config = ManagerConfig::<AsyncPgConnection>::default();
        manager_config.custom_setup = Box::new(move |url: &str| {
            let timeout = timeout_secs;
            let url = url.to_string();
            async move {
                let mut conn =
                    <AsyncPgConnection as diesel_async::AsyncConnection>::establish(&url).await?;
                let timeout_ms = (timeout * 1000) as i32;
                use diesel_async::RunQueryDsl;
                diesel::sql_query(format!("SET statement_timeout = {}", timeout_ms))
                    .execute(&mut conn)
                    .await
                    .map_err(|e| diesel::result::ConnectionError::BadConnection(e.to_string()))?;
                Ok(conn)
            }
            .boxed()
        });

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            &config.database_url,
            manager_config,
        );

        let pool = Pool::builder(manager)
            .max_size(config.db_pool_size as usize)
            .runtime(deadpool::Runtime::Tokio1)
            .timeouts(Timeouts {
                wait: Some(Duration::from_secs(config.db_pool_connection_timeout_secs)),
                create: Some(Duration::from_secs(config.db_pool_connection_timeout_secs)),
                recycle: Some(Duration::from_secs(30)),
            })
            .build()
            .expect("Failed to create database pool");

        Database { pool }
    }

    #[deprecated(since = "0.2.0", note = "Use `from_config` instead")]
    #[allow(dead_code)]
    pub fn new() -> Self {
        let config = AppConfig::from_env().expect("Failed to load configuration");
        Self::from_config(&config)
    }
}

impl Default for Database {
    fn default() -> Self {
        let config = AppConfig::from_env().expect("Failed to load configuration");
        Self::from_config(&config)
    }
}

// ---------------------------------------------------------------------------
// Connection runner helpers on the pool itself.
//
// These used to live in `repositories::base::BaseRepo`, a thin wrapper around
// the same `DBPool`. Moving them here removes the redundant wrapper and the
// misleading `base` name: the handle that repositories hold is the pool, and
// the pool knows how to run a closure against a connection (with a tracing
// span and, for `run_transaction`, a Diesel-managed transaction).
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait DbPoolExt {
    async fn run<F, T>(&self, f: F) -> QueryResult<T>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, QueryResult<T>> + Send,
        T: Send + 'static;

    async fn run_transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, Result<T, E>> + Send,
        T: Send + 'static,
        E: From<diesel::result::Error> + Send + 'static;

    /// Rails-style `exists?`: run an existence check and return a boolean.
    ///
    /// The closure should build a query that yields a `bool`, typically via
    /// `diesel::select(diesel::dsl::exists(subquery))` (which compiles to
    /// `SELECT EXISTS (...)`) or `.first::<T>().optional().map(|o| o.is_some())`.
    /// Connection checkout, the tracing span and error mapping are handled by
    /// the pool, exactly like `run`.
    async fn exists<F>(&self, f: F) -> QueryResult<bool>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, QueryResult<bool>> + Send;
}

#[async_trait::async_trait]
impl DbPoolExt for DBPool {
    async fn run<F, T>(&self, f: F) -> QueryResult<T>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, QueryResult<T>> + Send,
        T: Send + 'static,
    {
        let mut conn = self.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;
        f(&mut conn)
            .instrument(tracing::info_span!("db.query", pool = "diesel"))
            .await
    }

    async fn run_transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, Result<T, E>> + Send,
        T: Send + 'static,
        E: From<diesel::result::Error> + Send + 'static,
    {
        let mut conn = self.get().await.map_err(|e| {
            E::from(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            ))
        })?;
        // Use Diesel's native transaction API instead of manual BEGIN/COMMIT/
        // ROLLBACK via sql_query: the manual form bypasses AnsiTransactionManager,
        // so the connection's transaction depth is not tracked and deadpool may
        // hand out a connection with an open transaction to another request.
        // Diesel's `transaction` keeps the manager state correct, handles
        // savepoints/nested transactions, and rolls back on Err or panic.
        conn.transaction(async |c| f(c).await)
            .instrument(tracing::info_span!("db.transaction", pool = "diesel"))
            .await
    }

    async fn exists<F>(&self, f: F) -> QueryResult<bool>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, QueryResult<bool>> + Send,
    {
        self.run(f).await
    }
}

/// Whether a string can only ever be read as a SQL identifier.
///
/// A quote, a space, a semicolon or a parenthesis is what turns an identifier
/// into a statement: their absence leaves nothing to inject with.
pub fn is_sql_identifier(candidate: &str) -> bool {
    let mut characters = candidate.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && first != '_' {
        return false;
    }
    characters.all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    })
}

#[cfg(test)]
mod identifier_tests {
    use super::is_sql_identifier;

    #[test]
    fn a_schema_identifier_is_accepted() {
        assert!(is_sql_identifier("users"));
        assert!(is_sql_identifier("users_roles"));
        assert!(is_sql_identifier("_private"));
        assert!(is_sql_identifier("col2"));
    }

    #[test]
    fn an_identifier_cannot_carry_a_statement() {
        assert!(!is_sql_identifier("users; DROP TABLE users"));
        assert!(!is_sql_identifier("users\"--"));
        assert!(!is_sql_identifier("users(1)"));
        assert!(!is_sql_identifier("users WHERE 1=1"));
        assert!(!is_sql_identifier("public.users"));
    }

    #[test]
    fn an_identifier_cannot_start_with_a_digit_or_be_empty() {
        assert!(!is_sql_identifier("1users"));
        assert!(!is_sql_identifier(""));
    }
}
