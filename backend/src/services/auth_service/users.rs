#![allow(dead_code)]

use diesel::prelude::*;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::models::user::{NewUser, User};
use crate::security::SecurityService;
use crate::services::token::generate_random_token;
use crate::services::token_service::hash_token;
use rust_i18n::t;

/// Fetch user by email (case-insensitive).
pub fn find_user_by_email(
    conn: &mut PgConnection,
    email_input: &str,
    locale: &str,
) -> Result<User, AppError> {
    use crate::db::schema::users::dsl::*;
    let security = SecurityService::from_env()?;
    let protected_email = security.protect_email(email_input)?;
    users
        .filter(email_blind_index.eq(protected_email.blind_index))
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::Unauthorized(t!("auth.login.failed", locale = locale).into_owned())
            },
            _ => AppError::Database(e),
        })
}

/// Fetch user by ID.
pub fn find_user_by_id(conn: &mut PgConnection, user_id: Uuid) -> Result<User, AppError> {
    use crate::db::schema::users::dsl::*;
    users
        .find(user_id)
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::NotFound(t!("validation.fields.user_id").into_owned())
            },
            _ => AppError::Database(e),
        })
}

/// Register a new user: hash password, generate confirmation token, insert row.
pub fn register_user(
    conn: &mut PgConnection,
    email_input: &str,
    password: &str,
    token_salt: &str,
    config: &AppConfig,
    locale: &str,
) -> Result<(User, String), AppError> {
    super::password::validate_password_strength(password, locale)?;
    let security = SecurityService::from_env()?;
    let protected_email = security.protect_email(email_input)?;

    // Check for existing email
    use crate::db::schema::users::dsl::*;
    let exists: bool = diesel::select(diesel::dsl::exists(
        users.filter(email_blind_index.eq(protected_email.blind_index.clone())),
    ))
    .get_result(conn)
    .map_err(AppError::Database)?;

    if exists {
        return Err(AppError::Conflict(
            t!("auth.register.email_exists", locale = locale).into_owned(),
        ));
    }

    let hashed = super::password::hash_password(password, config)?;
    let confirmation_token_plain = generate_random_token(32);

    let new_user = NewUser::new(
        protected_email.blind_index,
        protected_email.encrypted,
        hashed,
        Some(hash_token(&confirmation_token_plain, token_salt)),
        protected_email.key_version,
    );

    let user = diesel::insert_into(users)
        .values((
            id.eq(new_user.id),
            email_blind_index.eq(new_user.email_blind_index),
            email_encrypted.eq(new_user.email_encrypted),
            encrypted_password.eq(&new_user.encrypted_password),
            confirmation_token_digest.eq(new_user.confirmation_token_digest),
            unconfirmed_email_blind_index.eq(new_user.unconfirmed_email_blind_index),
            unconfirmed_email_encrypted.eq(new_user.unconfirmed_email_encrypted),
            encryption_key_version.eq(new_user.encryption_key_version),
            created_at.eq(new_user.created_at),
            updated_at.eq(new_user.updated_at),
        ))
        .returning(User::as_returning())
        .get_result::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => AppError::Conflict(t!("auth.register.email_exists", locale = locale).into_owned()),
            _ => AppError::Database(e),
        })?;

    Ok((user, confirmation_token_plain))
}
