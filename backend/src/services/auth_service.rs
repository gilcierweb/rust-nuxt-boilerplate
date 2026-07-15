#![allow(dead_code)]

#[allow(unused_imports)]
use crate::errors::AppResult;
use crate::{
    errors::AppError,
    models::user::{NewUser, User},
    security::SecurityService,
    services::{token::generate_random_token, token_service::hash_token},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

/// Current Argon2 parameters for password hashing.
/// Update these when strengthening requirements.
const ARGON2_M_COST: u32 = 65536;
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 1;

/// Hash a plaintext password using Argon2id with current parameters.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let params = argon2::Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, None)
        .map_err(|e| AppError::Internal(format!("Invalid Argon2 parameters: {}", e)))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Password hash error: {}", e)))
}

/// Verify a plaintext password against an Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Password parse error: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Check if a password hash needs to be rehashed with current parameters.
/// Returns true if the hash was created with different parameters than the current policy.
pub fn needs_rehash(hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return true,
    };

    let params = &parsed.params;

    let m_cost = params.get("m").and_then(|v| v.to_string().parse::<u32>().ok()).unwrap_or(0);
    let t_cost = params.get("t").and_then(|v| v.to_string().parse::<u32>().ok()).unwrap_or(0);
    let p_cost = params.get("p").and_then(|v| v.to_string().parse::<u32>().ok()).unwrap_or(0);

    m_cost != ARGON2_M_COST || t_cost != ARGON2_T_COST || p_cost != ARGON2_P_COST
}

/// Rehash a password with current parameters. Returns the new hash.
/// Should only be called after successful password verification.
pub fn rehash_password(password: &str) -> Result<String, AppError> {
    hash_password(password)
}

/// Validate password strength: min 12 chars, at least 1 digit, 1 uppercase, 1 special char.
pub fn validate_password_strength(password: &str) -> Result<(), AppError> {
    if password.len() < 12 {
        return Err(AppError::Validation(
            t!("auth.password.must_be_12_chars").into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation(
            t!("auth.password.must_have_number").into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(AppError::Validation(
            t!("auth.password.must_have_uppercase").into_owned(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_punctuation()) {
        return Err(AppError::Validation(
            t!("auth.password.must_have_special").into_owned(),
        ));
    }
    Ok(())
}

/// Fetch user by email (case-insensitive).
pub fn find_user_by_email(conn: &mut PgConnection, email_input: &str) -> Result<User, AppError> {
    use crate::db::schema::users::dsl::*;
    let security = SecurityService::from_env()?;
    let protected_email = security.protect_email(email_input)?;
    users
        .filter(email_blind_index.eq(protected_email.blind_index))
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::Unauthorized("Invalid email or password".to_string())
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
            diesel::result::Error::NotFound => AppError::NotFound("User".to_string()),
            _ => AppError::Database(e),
        })
}

/// Register a new user: hash password, generate confirmation token, insert row.
pub fn register_user(
    conn: &mut PgConnection,
    email_input: &str,
    password: &str,
    token_salt: &str,
) -> Result<(User, String), AppError> {
    validate_password_strength(password)?;
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
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    let hashed = hash_password(password)?;
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
        .map_err(AppError::Database)?;

    Ok((user, confirmation_token_plain))
}

/// Confirm a user's email by their token.
pub fn confirm_email(conn: &mut PgConnection, token: &str, token_salt: &str) -> Result<User, AppError> {
    use crate::db::schema::users::dsl::*;
    use diesel::dsl::sql;

    let user = users
        .filter(confirmation_token_digest.eq(hash_token(token, token_salt)))
        .filter(confirmed_at.is_null())
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => {
                AppError::BadRequest("Invalid or already used confirmation token".to_string())
            },
            _ => AppError::Database(e),
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
        .map_err(AppError::Database)
}

/// Record a successful login: bump counter, update timestamps and IP.
pub fn record_successful_login(
    conn: &mut PgConnection,
    user: &User,
    ip: Option<String>,
) -> Result<(), AppError> {
    use crate::db::schema::users::dsl::*;
    let now = Utc::now();
    let ip_net: Option<ipnet::IpNet> = ip.and_then(|s| s.parse().ok());
    diesel::update(users.find(user.id))
        .set((
            sign_in_count.eq(user.sign_in_count + 1),
            last_sign_in_at.eq(user.current_sign_in_at),
            last_sign_in_ip.eq(user.current_sign_in_ip),
            current_sign_in_at.eq(Some(now)),
            current_sign_in_ip.eq(ip_net),
            failed_attempts.eq(0),
            locked_at.eq::<Option<chrono::DateTime<Utc>>>(None),
            updated_at.eq(now),
        ))
        .execute(conn)
        .map_err(AppError::Database)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::r2d2::{ConnectionManager, Pool};
    use std::ops::Deref;
    use std::sync::Arc;

    fn setup_test_db() -> Arc<Pool<ConnectionManager<PgConnection>>> {
        let database_url = std::env::var("DATABASE_URL_TEST")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        Arc::new(
            Pool::builder()
                .max_size(1)
                .build(manager)
                .expect("Failed to create pool"),
        )
    }

    fn get_conn(
        pool: &Arc<Pool<ConnectionManager<PgConnection>>>,
    ) -> impl Deref<Target = PgConnection> {
        pool.get().expect("Failed to get connection")
    }

    #[test]
    fn test_hash_password_creates_valid_hash() {
        let password = "TestPassword123!";
        let hash = hash_password(password).expect("Failed to hash password");

        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "TestPassword123!";
        let hash = hash_password(password).expect("Failed to hash password");
        let result = verify_password(password, &hash).expect("Failed to verify password");
        assert!(result);
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "TestPassword123!";
        let wrong_password = "WrongPassword123!";
        let hash = hash_password(password).expect("Failed to hash password");
        let result = verify_password(wrong_password, &hash).expect("Failed to verify password");
        assert!(!result);
    }

    #[test]
    fn test_verify_password_wrong_hash() {
        let password = "TestPassword123!";
        let wrong_hash = "$argon2id$v=19$m=19456,t=2,p=1$wrongsalt$wronghash";
        let result = verify_password(password, wrong_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_valid() {
        let password = "ValidPass123!";
        let result = validate_password_strength(password);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        let password = "Short1!";
        let result = validate_password_strength(password);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_digit() {
        let password = "NoDigitPass!";
        let result = validate_password_strength(password);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_uppercase() {
        let password = "nouppercase1!";
        let result = validate_password_strength(password);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_special() {
        let password = "NoSpecial123";
        let result = validate_password_strength(password);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_random_token() {
        let token1 = generate_random_token(32);
        let token2 = generate_random_token(32);
        assert_eq!(token1.len(), 32);
        assert_eq!(token2.len(), 32);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_hash_token_consistency() {
        use super::super::token_service::verify_token_hash;
        let token = "test-token-123";
        let salt = "test_salt";
        let hash1 = hash_token(token, salt);
        let hash2 = hash_token(token, salt);
        // Argon2id uses random salt, so hashes are different each time
        assert_ne!(hash1, hash2);
        // But both should verify correctly with the original token
        assert!(verify_token_hash(token, &hash1));
        assert!(verify_token_hash(token, &hash2));
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let salt = "test_salt";
        let hash1 = hash_token("token-1", salt);
        let hash2 = hash_token("token-2", salt);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_different_salts() {
        let token = "test-token-123";
        let hash1 = hash_token(token, "salt1");
        let hash2 = hash_token(token, "salt2");
        assert_ne!(hash1, hash2);
    }
}
