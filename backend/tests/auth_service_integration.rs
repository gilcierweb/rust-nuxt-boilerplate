//! Integration tests for the auth_service DB-bound helpers
//! (users + email confirmation).
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST (default: postgres://postgres:postgres@localhost:5432/test_db)
//! - MASTER_KEY and BLIND_INDEX_KEY set (valid base64, >= 32 bytes)

use backend::services::auth_service::{
    confirm_email, find_user_by_email, find_user_by_id, record_successful_login, register_user,
};
use diesel::r2d2::{ConnectionManager, Pool};
use std::sync::Arc;

fn setup_env() {
    // SAFETY: idempotent writes, same values across tests.
    unsafe {
        std::env::set_var("MASTER_KEY", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    }
    unsafe {
        std::env::set_var(
            "BLIND_INDEX_KEY",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        );
    }
}

fn setup_pool() -> Arc<Pool<ConnectionManager<diesel::PgConnection>>> {
    setup_env();
    let url = std::env::var("DATABASE_URL_TEST")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());
    let manager = ConnectionManager::<diesel::PgConnection>::new(url);
    Arc::new(
        Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("Failed to create pool"),
    )
}

#[test]
fn find_user_by_email_returns_not_found_for_unknown_email() {
    let pool = setup_pool();
    let mut conn = pool.get().expect("conn");
    let email = format!("nonexistent-{}@example.com", uuid::Uuid::new_v4());
    let res = find_user_by_email(&mut conn, &email, "pt-BR");
    assert!(res.is_err());
}

#[test]
fn find_user_by_id_returns_not_found_for_unknown_id() {
    let pool = setup_pool();
    let mut conn = pool.get().expect("conn");
    let res = find_user_by_id(&mut conn, uuid::Uuid::new_v4());
    assert!(res.is_err());
}

#[test]
fn register_user_rejects_duplicate_email() {
    use backend::config::test_config::test_config;
    let pool = setup_pool();
    let mut conn = pool.get().expect("conn");
    let email = format!("dup-{}@example.com", uuid::Uuid::new_v4());
    let pw = "ValidPass123!";
    let config = test_config();
    let first = register_user(&mut conn, &email, pw, "salt", &config, "pt-BR");
    assert!(
        first.is_ok(),
        "first registration should succeed: {:?}",
        first.err()
    );
    let second = register_user(&mut conn, &email, pw, "salt", &config, "pt-BR");
    assert!(
        matches!(second, Err(backend::errors::AppError::Conflict(_))),
        "duplicate should be Conflict, got {:?}",
        second.err()
    );
}

#[test]
fn register_user_rejects_weak_password() {
    use backend::config::test_config::test_config;
    let pool = setup_pool();
    let mut conn = pool.get().expect("conn");
    let email = format!("weak-{}@example.com", uuid::Uuid::new_v4());
    let config = test_config();
    let res = register_user(&mut conn, &email, "short", "salt", &config, "pt-BR");
    assert!(res.is_err());
}

#[test]
fn confirm_email_rejects_invalid_token() {
    let pool = setup_pool();
    let mut conn = pool.get().expect("conn");
    let res = confirm_email(&mut conn, "not-a-real-token", "salt");
    assert!(res.is_err());
}

#[test]
fn confirm_email_succeeds_with_valid_token() {
    use backend::config::test_config::test_config;
    let pool = setup_pool();
    let mut conn = pool.get().expect("conn");
    let email = format!("confirm-{}@example.com", uuid::Uuid::new_v4());
    let config = test_config();
    let (user, token) = register_user(&mut conn, &email, "ValidPass123!", "salt", &config, "pt-BR")
        .expect("registration should succeed");
    assert!(user.confirmed_at.is_none());

    let confirmed = confirm_email(&mut conn, &token, "salt").expect("confirmation should succeed");
    assert!(confirmed.confirmed_at.is_some());
}

#[test]
fn record_successful_login_bumps_counters() {
    use backend::config::test_config::test_config;
    let pool = setup_pool();
    let mut conn = pool.get().expect("conn");
    let email = format!("login-{}@example.com", uuid::Uuid::new_v4());
    let config = test_config();
    let (user, _token) =
        register_user(&mut conn, &email, "ValidPass123!", "salt", &config, "pt-BR")
            .expect("registration should succeed");

    let before = user.sign_in_count;
    record_successful_login(&mut conn, &user, Some("127.0.0.1".to_string()))
        .expect("record login should succeed");

    let refreshed = backend::services::auth_service::find_user_by_id(&mut conn, user.id)
        .expect("user should exist");
    assert_eq!(refreshed.sign_in_count, before + 1);
    assert!(refreshed.current_sign_in_at.is_some());
}
