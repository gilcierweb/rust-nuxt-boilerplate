use crate::db::database::DBPool;
use diesel::prelude::*;
use diesel::Connection;

#[derive(QueryableByName)]
#[allow(dead_code)]
struct ExistsResult {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    flag: bool,
}

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

    /// Executes a Diesel transaction by acquiring a connection from the pool.
    /// The transaction will be committed if the closure returns `Ok`, rolled back if `Err`.
    pub async fn run_transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut PgConnection) -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: From<diesel::result::Error> + Send + 'static,
    {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().expect("Failed to get DB connection from pool");
            conn.transaction::<T, E, _>(|conn| f(conn))
        })
        .await
        .unwrap_or_else(|e| Err(E::from(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        ))))
    }

    /// Generic exists? — equivalent to Rails' `Model.exists?(column: value)`.
    ///
    /// Uses `SELECT EXISTS(...)` with parameterized values ($1) for safety.
    /// `table` and `column` are developer-controlled identifiers (never user input).
    ///
    /// # Examples
    /// ```ignore
    /// // Rails equivalent:
    /// // User.exists?(email: "foo@bar.com")
    /// repo.base.exists("users", "email", "foo@bar.com").await?
    ///
    /// // Establishment.exists?(slug: "restaurante-bom")
    /// repo.base.exists("establishments", "slug", "restaurante-bom").await?
    /// ```
    #[allow(dead_code)]
    pub async fn exists(&self, table: &str, column: &str, value: &str) -> QueryResult<bool> {
        let tbl = table.to_string();
        let col = column.to_string();
        let val = value.to_string();
        self.run(move |conn| {
            let query = format!("SELECT EXISTS(SELECT 1 FROM {tbl} WHERE {col} = $1) AS flag");
            let result = diesel::sql_query(query)
                .bind::<diesel::sql_types::Text, _>(&val)
                .get_result::<ExistsResult>(conn)?;
            Ok(result.flag)
        })
        .await
    }
}