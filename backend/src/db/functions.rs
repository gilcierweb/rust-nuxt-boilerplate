//! The SQL functions the query builder does not map, expressed once.
//!
//! `LIKE` in Postgres is case sensitive while the search of the admin lists is
//! case insensitive, so a fragment match lowers both sides. `lower` and `like`
//! are ordinary functions in Postgres (unlike `ILIKE`, which is an operator the
//! query builder cannot build), which keeps the comparison in the query builder
//! instead of an interpolated SQL fragment.

use diesel::expression::AsExpression;
use diesel::sql_types::{Bool, Nullable, Text};

diesel::define_sql_function! {
    fn lower(value: Nullable<Text>) -> Nullable<Text>;
}

diesel::define_sql_function! {
    fn like(left: Nullable<Text>, right: Nullable<Text>) -> Bool;
}

/// A case insensitive fragment match on a nullable text column.
///
/// The column is lowered together with the pattern, so a term in lower case
/// matches a stored name in any case. A NULL column never matches, which is the
/// behaviour of `LIKE` on a missing value.
pub fn matches_fragment<Column>(
    column: Column,
    pattern: &str,
) -> like<lower<Column>, Option<String>>
where
    Column: AsExpression<Nullable<Text>>,
{
    like(lower(column), Some(pattern.to_lowercase()))
}
