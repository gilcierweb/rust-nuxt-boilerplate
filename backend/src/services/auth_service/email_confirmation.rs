#![allow(dead_code)]

use diesel::prelude::*;
use time::OffsetDateTime;

use crate::errors::AppError;
use crate::models::user::User;
use crate::services::token_service::hash_token;
use rust_i18n::t;

/// Confirm a user's email by their token.
pub fn confirm_email(
    conn: &mut PgConnection,
    token: &str,
    token_salt: &str,
) -> Result<User, AppError> {
    use diesel::dsl::sql;

    use crate::db::schema::users::dsl::*;

    let user = users
        .filter(confirmation_token_digest.eq(hash_token(token, token_salt)))
        .filter(confirmed_at.is_null())
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::BadRequest(t!("auth.confirm.token_invalid").into_owned())
            },
            _ => AppError::Database(e),
        })?;

    let now = OffsetDateTime::now_utc();
    diesel::update(users.find(user.id))
        .set((
            confirmed_at.eq(Some(now)),
            email_blind_index.eq(sql::<diesel::sql_types::Bytea>(
                "COALESCE(unconfirmed_email_blind_index, email_blind_index)",
            )),
            email_encrypted.eq(sql::<diesel::sql_types::Bytea>(
                "COALESCE(unconfirmed_email_encrypted, email_encrypted)",
            )),
            confirmation_token_digest.eq::<Option<String>>(None),
            unconfirmed_email_blind_index.eq::<Option<Vec<u8>>>(None),
            unconfirmed_email_encrypted.eq::<Option<Vec<u8>>>(None),
            updated_at.eq(now),
        ))
        .returning(User::as_returning())
        .get_result::<User>(conn)
        .map_err(AppError::Database)
}

/// Record a successful login: bump counter, update timestamps and IP.
pub fn record_successful_login(
    conn: &mut PgConnection,
    user: &User,
    ip: Option<String>,
) -> Result<(), AppError> {
    use crate::db::schema::users::dsl::*;
    let now = OffsetDateTime::now_utc();
    let ip_net: Option<ipnet::IpNet> = ip.and_then(|s| s.parse().ok());
    diesel::update(users.find(user.id))
        .set((
            sign_in_count.eq(user.sign_in_count + 1),
            last_sign_in_at.eq(user.current_sign_in_at),
            last_sign_in_ip.eq(user.current_sign_in_ip),
            current_sign_in_at.eq(Some(now)),
            current_sign_in_ip.eq(ip_net),
            failed_attempts.eq(0),
            locked_at.eq::<Option<OffsetDateTime>>(None),
            updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(AppError::Database)?;
    Ok(())
}
