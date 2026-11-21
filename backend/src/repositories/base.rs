use diesel::result::QueryResult;
use diesel::{QueryableByName, sql_query};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures::future::BoxFuture;
use tracing::Instrument;

use crate::db::database::DBPool;

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
        f(&mut conn)
            .instrument(tracing::info_span!("db.query", pool = "diesel"))
            .await
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
        match f(&mut conn)
            .instrument(tracing::info_span!("db.transaction", pool = "diesel"))
            .await
        {
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

    /// Whether a row with `value` in `column` of `table` exists.
    ///
    /// When the table is known at compile time, prefer the query builder: Diesel
    /// spells the identifiers from the schema module, so nothing is interpolated.
    /// The idiom, already used by `auth_service`, is
    /// `diesel::select(diesel::dsl::exists(table.filter(column.eq(value))))`.
    ///
    /// This helper stays for the case where the table is only known at runtime,
    /// which the builder cannot express: the identifiers are then interpolated,
    /// and the value stays a bound parameter. The two identifiers are validated as
    /// plain lowercase identifiers first, so even a caller passing request data
    /// verbatim cannot smuggle SQL into the FROM or WHERE clause.
    #[allow(dead_code)]
    pub async fn exists(&self, table: &str, column: &str, value: &str) -> QueryResult<bool> {
        let tbl = table.to_string();
        let col = column.to_string();
        let val = value.to_string();

        if !is_sql_identifier(&tbl) || !is_sql_identifier(&col) {
            return Err(diesel::result::Error::QueryBuilderError(
                "the table and the column must be plain lowercase identifiers".into(),
            ));
        }

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
        // What turns a name into an injection: a quote, a separator, a
        // comment, a keyword run together with punctuation.
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
