#![allow(dead_code)]

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use diesel::SelectableHelper;
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    errors::ApiError,
    models::user::{NewUser, User},
    security::SecurityService,
    services::{token::generate_random_token, token_service::hash_token},
};

/// Hash a plaintext password using Argon2id.
pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ApiError::Internal(format!("Password hash error: {}", e)))
}

/// Verify a plaintext password against an Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| ApiError::Internal(format!("Password parse error: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Validate password strength: min 12 chars, at least 1 digit, 1 uppercase, 1 special char.
pub fn validate_password_strength(password: &str) -> Result<(), ApiError> {
    if password.len() < 12 {
        return Err(ApiError::Validation(
            t!("auth.password.must_be_12_chars").into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ApiError::Validation(
            t!("auth.password.must_have_number").into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ApiError::Validation(
            t!("auth.password.must_have_uppercase").into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_punctuation()) {
        return Err(ApiError::Validation(
            t!("auth.password.must_have_special").into_owned(),
        ));
    }
    Ok(())
}

/// Fetch user by email (case-insensitive).
pub fn find_user_by_email(conn: &mut PgConnection, email_input: &str) -> Result<User, ApiError> {
    use crate::db::schema::users::dsl::*;
    let security = SecurityService::from_env()?;
    let protected_email = security.protect_email(email_input)?;
    users
        .filter(email_blind_index.eq(protected_email.blind_index))
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                ApiError::Unauthorized("Invalid email or password".to_string())
            }
            _ => ApiError::Database(e),
        })
}

/// Fetch user by ID.
pub fn find_user_by_id(conn: &mut PgConnection, user_id: Uuid) -> Result<User, ApiError> {
    use crate::db::schema::users::dsl::*;
    users
        .find(user_id)
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => ApiError::NotFound("User".to_string()),
            _ => ApiError::Database(e),
        })
}

/// Register a new user: hash password, generate confirmation token, insert row.
pub fn register_user(
    conn: &mut PgConnection,
    email_input: &str,
    password: &str,
) -> Result<(User, String), ApiError> {
    validate_password_strength(password)?;
    let security = SecurityService::from_env()?;
    let protected_email = security.protect_email(email_input)?;

    // Check for existing email
    use crate::db::schema::users::dsl::*;
    let exists: bool = diesel::select(diesel::dsl::exists(
        users.filter(email_blind_index.eq(protected_email.blind_index.clone())),
    ))
    .get_result(conn)
    .map_err(ApiError::Database)?;

    if exists {
        return Err(ApiError::Conflict("Email already registered".to_string()));
    }

    let hashed = hash_password(password)?;
    let confirmation_token_plain = generate_random_token(32);

    let new_user = NewUser::new(
        protected_email.blind_index,
        protected_email.encrypted,
        hashed,
        Some(hash_token(&confirmation_token_plain)),
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
        .map_err(ApiError::Database)?;

    Ok((user, confirmation_token_plain))
}

/// Confirm a user's email by their token.
pub fn confirm_email(conn: &mut PgConnection, token: &str) -> Result<User, ApiError> {
    use crate::db::schema::users::dsl::*;
    use diesel::dsl::sql;

    let user = users
        .filter(confirmation_token_digest.eq(hash_token(token)))
        .filter(confirmed_at.is_null())
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                ApiError::BadRequest("Invalid or already used confirmation token".to_string())
            }
            _ => ApiError::Database(e),
        })?;

    let now = Utc::now();
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
        .map_err(ApiError::Database)
}

/// Record a successful login: bump counter, update timestamps and IP.
pub fn record_successful_login(
    conn: &mut PgConnection,
    user: &User,
    ip: Option<String>,
) -> Result<(), ApiError> {
    use crate::db::schema::users::dsl::*;
    let now = Utc::now();
    let ip_net: Option<ipnet::IpNet> = ip.and_then(|s| s.parse().ok());
    diesel::update(users.find(user.id))
        .set((
            sign_in_count.eq(user.sign_in_count + 1),
            last_sign_in_at.eq(user.current_sign_in_at),
            last_sign_in_ip.eq(user.current_sign_in_ip.clone()),
            current_sign_in_at.eq(Some(now)),
            current_sign_in_ip.eq(ip_net),
            failed_attempts.eq(0),
            locked_at.eq::<Option<chrono::DateTime<Utc>>>(None),
            updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(ApiError::Database)?;
    Ok(())
}

/// Increment failed login attempts; lock account if threshold reached.
pub fn record_failed_login(
    conn: &mut PgConnection,
    user: &User,
    max_attempts: i32,
) -> Result<(), ApiError> {
    use crate::db::schema::users::dsl::*;
    let now = Utc::now();
    let new_attempts = user.failed_attempts + 1;
    let lock_time = if new_attempts >= max_attempts {
        Some(now)
    } else {
        None
    };

    diesel::update(users.find(user.id))
        .set((
            failed_attempts.eq(new_attempts),
            locked_at.eq(lock_time),
            updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(ApiError::Database)?;
    Ok(())
}

/// Issue a password reset token (stored as plain, returned to caller for email).
pub fn create_password_reset_token(
    conn: &mut PgConnection,
    email_input: &str,
) -> Result<(User, String), ApiError> {
    use crate::db::schema::users::dsl::*;
    let security = SecurityService::from_env()?;
    let protected_email = security.protect_email(email_input)?;

    let user = users
        .filter(email_blind_index.eq(protected_email.blind_index))
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                // Intentionally vague to prevent email enumeration
                ApiError::BadRequest(
                    "If this email exists you will receive reset instructions".to_string(),
                )
            }
            _ => ApiError::Database(e),
        })?;

    let token_plain = generate_random_token(32);
    let now = Utc::now();

    diesel::update(users.find(user.id))
        .set((
            reset_password_token_digest.eq(Some(hash_token(&token_plain))),
            reset_password_sent_at.eq(Some(now)),
            updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(ApiError::Database)?;

    Ok((user, token_plain))
}

/// Consume a reset token, validate expiry (2h), set new password.
pub fn reset_password(
    conn: &mut PgConnection,
    token: &str,
    new_password: &str,
) -> Result<User, ApiError> {
    use crate::db::schema::users::dsl::*;

    validate_password_strength(new_password)?;

    let user = users
        .filter(reset_password_token_digest.eq(hash_token(token)))
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                ApiError::BadRequest("Invalid or expired reset token".to_string())
            }
            _ => ApiError::Database(e),
        })?;

    // Check token expiry (2 hours)
    if let Some(sent_at) = user.reset_password_sent_at {
        if Utc::now() - sent_at > chrono::Duration::hours(2) {
            return Err(ApiError::BadRequest("Reset token has expired".to_string()));
        }
    } else {
        return Err(ApiError::BadRequest("Invalid reset token".to_string()));
    }

    let hashed = hash_password(new_password)?;
    let now = Utc::now();

    diesel::update(users.find(user.id))
        .set((
            encrypted_password.eq(hashed),
            reset_password_token_digest.eq::<Option<String>>(None),
            reset_password_sent_at.eq::<Option<chrono::DateTime<Utc>>>(None),
            failed_attempts.eq(0),
            locked_at.eq::<Option<chrono::DateTime<Utc>>>(None),
            updated_at.eq(now),
        ))
        .returning(User::as_returning())
        .get_result::<User>(conn)
        .map_err(ApiError::Database)
}

/// Get roles for a user from users_roles + roles tables.
pub fn get_user_roles(conn: &mut PgConnection, user_id: Uuid) -> Result<Vec<String>, ApiError> {
    use crate::db::schema::{roles, users_roles};

    let role_names: Vec<String> = users_roles::table
        .inner_join(roles::table.on(roles::id.eq(users_roles::role_id)))
        .filter(users_roles::user_id.eq(user_id))
        .select(roles::name)
        .load::<String>(conn)
        .map_err(ApiError::Database)?;

    Ok(role_names)
}
