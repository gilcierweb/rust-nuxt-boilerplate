use deadpool::managed::Pool;
use deadpool::managed::Timeouts;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;
use futures_util::FutureExt;
use std::time::Duration;

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
