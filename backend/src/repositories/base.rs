use crate::db::database::DBPool;
use diesel::prelude::*;

pub struct BaseRepo {
    pub pool: DBPool,
}

impl BaseRepo {
    pub fn new(pool: DBPool) -> Self {
        Self { pool }
    }

    /// Executes any Diesel query by acquiring a connection from the pool.
    /// This eliminates the repeated `pool.get().unwrap()` boilerplate.
    pub async fn run<F, T>(&self, f: F) -> QueryResult<T>
    where
        F: FnOnce(&mut PgConnection) -> QueryResult<T> + Send + 'static,
        T: Send + 'static,
    {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().expect("Failed to get DB connection from pool");
            f(&mut conn)
        })
        .await
        .unwrap_or_else(|e| {
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            ))
        })
    }
}
