//! The queries the builder does not map, expressed once.
//!
//! The search of the admin lists is case insensitive. Postgres spells it
//! `ILIKE`, which is an operator rather than a function, so no builder of the
//! query builder produces it. The "Extending Diesel" guide documents custom
//! operators for this case: `infix_operator!` emits the operator with bound
//! parameters, which keeps the comparison inside the query builder instead of
//! an interpolated SQL fragment.

use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::sql_types::{Nullable, Text};

diesel::infix_operator!(CaseInsensitiveLike, " ILIKE ", backend: Pg);

/// A case insensitive fragment match on a nullable text column.
///
/// `ILIKE` folds the case of both sides, so a term in lower case matches a
/// stored name in any case. A NULL column never matches, which is the
/// behaviour of the operator on a missing value.
pub type FragmentMatch<Column> = CaseInsensitiveLike<
    <Column as AsExpression<Nullable<Text>>>::Expression,
    <String as AsExpression<Text>>::Expression,
>;

pub fn matches_fragment<Column>(column: Column, pattern: &str) -> FragmentMatch<Column>
where
    Column: AsExpression<Nullable<Text>>,
{
    CaseInsensitiveLike::new(
        column.as_expression(),
        AsExpression::<Text>::as_expression(pattern.to_string()),
    )
}
