use crate::db::database::DBPool;
use diesel::{QueryableByName, result::QueryResult, sql_query};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures::future::BoxFuture;

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

    pub async fn run<F, T>(&self, f: F) -> QueryResult<T>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, QueryResult<T>> + Send,
        T: Send + 'static,
    {
        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;
        f(&mut conn).await
    }

    pub async fn run_transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, Result<T, E>> + Send,
        T: Send + 'static,
        E: From<diesel::result::Error> + Send + 'static,
    {
        let mut conn = self.pool.get().await.map_err(|e| {
            E::from(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            ))
        })?;
        diesel::sql_query("BEGIN")
            .execute(&mut *conn)
            .await
            .map_err(E::from)?;
        match f(&mut conn).await {
            Ok(val) => {
                diesel::sql_query("COMMIT")
                    .execute(&mut *conn)
                    .await
                    .map_err(E::from)?;
                Ok(val)
            },
            Err(e) => {
                let _ = diesel::sql_query("ROLLBACK").execute(&mut *conn).await;
                Err(e)
            },
        }
    }

    #[allow(dead_code)]
    pub async fn exists(&self, table: &str, column: &str, value: &str) -> QueryResult<bool> {
        let tbl = table.to_string();
        let col = column.to_string();
        let val = value.to_string();
        self.run(move |conn| {
            Box::pin(async move {
                let query = format!("SELECT EXISTS(SELECT 1 FROM {tbl} WHERE {col} = $1) AS flag");
                let result = sql_query(query)
                    .bind::<diesel::sql_types::Text, _>(&val)
                    .get_result::<ExistsResult>(conn)
                    .await?;
                Ok(result.flag)
            })
        })
        .await
    }
}
