//! Integration tests for the full auth flow.
//!
//! These tests spin up a real Actix-web app with Postgres + Redis and test
//! the complete register → login → access → refresh → logout cycle.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST (default: postgres://postgres:postgres@localhost:5432/test_db)
//! - Redis running at REDIS_URL_TEST (default: redis://127.0.0.1:6379)
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`

use std::sync::Arc;

use actix_web::dev::Service;
use actix_web::{App, test, web};
use deadpool::managed::Pool;
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde_json::Value;
use uuid::Uuid as UuidType;

mod common;

use common::dispatch;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

struct TestDb {
    pool: Pool<AsyncDieselConnectionManager<AsyncPgConnection>>,
}

impl TestDb {
    async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(manager)
            .max_size(4)
            .build()
            .expect("Failed to create pool");
        Self { pool }
    }

    fn pool(&self) -> &Pool<AsyncDieselConnectionManager<AsyncPgConnection>> {
        &self.pool
    }
}

fn redis_pool() -> deadpool_redis::Pool {
    let redis_url =
        std::env::var("REDIS_URL_TEST").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let mut cfg = RedisConfig::from_url(&redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig::new(5));
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}

fn test_config() -> backend::config::AppConfig {
    use backend::config::app_config::Environment;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let database_url = std::env::var("DATABASE_URL_TEST")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());
    let redis_url =
        std::env::var("REDIS_URL_TEST").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Generate a unique JWT secret per test run to avoid token collisions
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        let mut rng = StdRng::from_entropy();
        const CHARSET: &[u8] =
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
        (0..64)
            .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .collect::<String>()
    });
    let csrf_key = jwt_secret.clone();
    let master_key = std::env::var("MASTER_KEY").unwrap_or_else(|_| {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode([0xABu8; 32])
    });
    let blind_index_key = std::env::var("BLIND_INDEX_KEY").unwrap_or_else(|_| {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode([0xCDu8; 32])
    });

    backend::config::AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        https_port: 8443,
        tls_enabled: false,
        tls_cert_path: String::new(),
        tls_key_path: String::new(),
        frontend_url: "http://localhost:3000".to_string(),
        environment: Environment::Test,
        database_url,
        db_pool_size: 4,
        db_pool_min_idle: Some(1),
        db_pool_max_lifetime_secs: Some(1800),
        db_pool_idle_timeout_secs: Some(600),
        db_pool_connection_timeout_secs: 10,
        db_statement_timeout_secs: 30,
        redis_url,
        redis_pool_size: 5,
        jwt_secret: jwt_secret.clone(),
        jwt_public_key: None,
        jwt_access_expiry_secs: 3600,
        jwt_refresh_expiry_secs: 3600,
        master_key,
        blind_index_key,
        current_encryption_key_version: 1,
        internal_api_keys: vec![],
        resend_api_key: String::new(),
        email_from: "test@example.com".to_string(),
        email_from_name: "Test".to_string(),
        email_transport: backend::config::app_config::EmailTransportKind::Resend,
        smtp_url: String::new(),
        smtp_timeout_secs: 30,
        bunny_storage_zone: String::new(),
        bunny_storage_key: String::new(),
        bunny_cdn_url: String::new(),
        bunny_token_key: String::new(),
        bunny_stream_library_id: String::new(),
        bunny_stream_key: String::new(),
        bunny_stream_webhook_secret: String::new(),
        b2_key_id: String::new(),
        b2_application_key: String::new(),
        b2_bucket_id: String::new(),
        b2_bucket_name: String::new(),
        b2_endpoint: String::new(),
        stripe_secret_key: String::new(),
        stripe_webhook_secret: String::new(),
        stripe_publishable_key: String::new(),
        platform_commission_percent: 20.0,
        min_subscription_price_cents: 500,
        max_subscription_price_cents: 50_000,
        min_withdrawal_amount_cents: 2_000,
        totp_issuer: "Test".to_string(),
        max_video_size_bytes: 1024 * 1024 * 10,
        max_photo_size_bytes: 1024 * 1024 * 5,
        max_audio_size_bytes: 1024 * 1024 * 5,
        json_payload_limit: 1024 * 1024,
        form_payload_limit: 2 * 1024 * 1024,
        csrf_secret_key: csrf_key,
        refresh_token_hash_salt: "test-salt-for-refresh-tokens-16b!".to_string(),
        account_lockout_duration_secs: 15 * 60,
        pix_webhook_secret: String::new(),
        rate_limit_enabled: false,
        argon2_m_cost: 65536,
        argon2_t_cost: 3,
        argon2_p_cost: 1,
        trusted_proxies: Vec::new(),
        jwt_secrets: vec![backend::config::app_config::JwtSecretKey {
            kid: "test-key-1".to_string(),
            secret: jwt_secret,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            expires_at: None,
        }],
        cookie_secure_override: false,
    }
}

fn extract_cookie(
    response: &actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
    name: &str,
) -> Option<String> {
    response
        .headers()
        .get("set-cookie")?
        .to_str()
        .ok()?
        .lines()
        .find(|line| line.starts_with(&format!("{}=", name)))
        .and_then(|line| {
            let value = line.split(';').next()?;
            Some(value.strip_prefix(&format!("{}=", name))?.to_string())
        })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_full_auth_cycle() {
    let db = TestDb::new().await;
    // Use existing schema from diesel migrations (applied in CI before this test)
    // db.drop_all_tables().await;
    // db.create_auth_schema().await;

    let config = Arc::new(test_config());
    let r_pool = redis_pool();

    let state = web::Data::new(backend::AppState {
        db: db.pool().clone(),
        redis: r_pool.clone(),
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsRedisState::new(r_pool.clone(), backend::ws::WsLimits::default()),
        jwt_secrets: Arc::new(config.jwt_secrets.clone()),
        otel_provider: None,
    });

    let container = web::Data::new(backend::repositories::AppContainer::new(
        db.pool().clone(),
        r_pool.clone(),
        (*config).clone(),
    ));

    let ws_state = web::Data::new(backend::ws::WsRedisState::new(
        r_pool.clone(),
        backend::ws::WsLimits::default(),
    ));

    let app = test::init_service(
        App::new()
            .app_data(state)
            .app_data(container)
            .app_data(ws_state)
            .app_data(web::JsonConfig::default().limit(1024 * 1024).error_handler(
                |_error, _request| {
                    backend::errors::AppError::BadRequest("bad request".to_string()).into()
                },
            ))
            .service(
                web::scope("/api/v1")
                    .service(
                        // Mirror the production /auth middleware chain (JwtAuth
                        // outermost, CsrfProtection innermost): the real
                        // Bearer + CSRF flows are exercised end-to-end.
                        web::scope("/auth")
                            .wrap(backend::middleware::csrf_protection::CsrfProtection::new(vec![]))
                            .wrap(backend::middleware::auth::JwtAuth::new(
                                backend::middleware::auth_middleware::JwtAuthConfig::bearer_exempt()
                                    .with_token_blacklist(Arc::new(
                                        backend::repositories::access_token_blacklist::AccessTokenBlacklist::new(r_pool.clone()),
                                    )),
                            ))
                            .service(backend::controllers::auth_controller::register)
                            .service(backend::controllers::auth_controller::login)
                            .service(backend::controllers::auth_controller::session)
                            .service(backend::controllers::auth_controller::session_trailing)
                            .service(backend::controllers::auth_controller::refresh)
                            .service(backend::controllers::auth_controller::logout)
                            .service(backend::controllers::auth_controller::me)
                            .service(backend::controllers::auth_controller::recover_password)
                            .service(backend::controllers::auth_controller::reset_password)
                            .service(backend::controllers::auth_controller::confirm)
                            .service(backend::controllers::auth_controller::change_password)
                            .service(backend::controllers::auth_controller::setup_2fa)
                            .service(backend::controllers::auth_controller::enable_2fa)
                            .service(backend::controllers::auth_controller::disable_2fa),
                    )
                    .service(backend::controllers::health_controller::health_check),
            ),
    )
    .await;

    // --- Step 1: Register ---
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(serde_json::json!({
                "email": "test@example.com",
                "password": "StrongPass123!@#",
                "password_confirmation": "StrongPass123!@#"
            }))
            .to_request(),
    )
    .await;
    let status = resp.status();
    let body: Value = test::read_body_json(resp).await;
    println!("Register: {} - {:?}", status, body);
    assert!(status.is_success() || status.as_u16() == 409);

    // --- Step 1b: Confirm email (required for login) ---
    if status.is_success() {
        let user_id = body["user_id"].as_str().unwrap();
        let mut conn = db.pool().get().await.expect("Failed to get connection");
        diesel::sql_query("UPDATE users SET confirmed_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(UuidType::parse_str(user_id).unwrap())
            .execute(&mut *conn)
            .await
            .expect("Failed to confirm user email");
    }

    // --- Step 2: Login ---
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(serde_json::json!({
                "email": "test@example.com",
                "password": "StrongPass123!@#"
            }))
            .to_request(),
    )
    .await;
    let status = resp.status();
    let refresh_cookie = extract_cookie(&resp, "refresh_token");
    let body: Value = test::read_body_json(resp).await;
    println!("Login: {} - {:?}", status, body);
    assert!(status.is_success(), "Login failed: {:?}", body);

    let access_token = body["access_token"].as_str().unwrap();
    println!("Refresh cookie: {:?}", refresh_cookie);

    // --- Step 3: Access protected endpoint (real Bearer auth) ---
    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", format!("Bearer {}", access_token)))
            .to_request(),
    )
    .await;
    let status = resp.status();
    let body: Value = test::read_body_json(resp).await;
    println!("Me: {} - {:?}", status, body);
    assert!(status.is_success(), "Me failed: {:?}", body);
    assert_eq!(body["email"], "test@example.com");

    // Missing Bearer → 401 (the middleware inserts Claims; the extractor
    // must reject when they are absent).
    let resp = dispatch!(app, || { test::TestRequest::get().uri("/api/v1/auth/me") });
    assert_eq!(resp.status(), 401, "me without Bearer must be rejected");

    // Garbage Bearer → 401.
    let resp = dispatch!(app, || {
        test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", "Bearer not-a-real-token"))
    });
    assert_eq!(
        resp.status(),
        401,
        "me with garbage Bearer must be rejected"
    );

    // --- Step 4: Refresh token (cookie-authenticated + CSRF) ---
    // The refresh cookie alone is not enough: refresh is a cookie-based
    // state change, so the CSRF middleware requires the csrf-token header.
    // Obtain the CSRF cookie first — the middleware sets it on GET
    // responses when not already present.
    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/auth/session")
            .insert_header((
                "Cookie",
                format!("refresh_token={}", refresh_cookie.clone().unwrap()),
            ))
            .to_request(),
    )
    .await;
    assert!(
        resp.status().is_success(),
        "session GET failed before refresh"
    );
    let csrf_cookie =
        extract_cookie(&resp, "csrf_token").expect("session GET must set the csrf cookie");

    let resp = dispatch!(app, || {
        test::TestRequest::post()
            .uri("/api/v1/auth/refresh")
            .insert_header((
                "Cookie",
                format!(
                    "refresh_token={}; csrf_token={}",
                    refresh_cookie.clone().unwrap(),
                    csrf_cookie
                ),
            ))
            .insert_header(("csrf-token", csrf_cookie.clone()))
    });
    let status = resp.status();
    let body: Value = test::read_body_json(resp).await;
    println!("Refresh: {} - {:?}", status, body);
    assert!(status.is_success(), "Refresh failed: {:?}", body);

    let new_token = body["access_token"].as_str().unwrap();
    assert_ne!(access_token, new_token, "token should rotate");

    // --- Step 5: Use new token ---
    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", format!("Bearer {}", new_token)))
            .to_request(),
    )
    .await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["email"], "test@example.com");

    // --- Step 6: Logout ---
    let mut logout_req = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .insert_header(("Authorization", format!("Bearer {}", new_token)))
        .to_request();
    if let Some(ref cookie) = refresh_cookie {
        logout_req.headers_mut().insert(
            actix_web::http::header::COOKIE,
            format!("refresh_token={}", cookie).parse().unwrap(),
        );
    }
    let resp = test::call_service(&app, logout_req).await;
    println!("Logout: {}", resp.status());
    assert!(resp.status().is_success() || resp.status().as_u16() == 204);

    // --- Step 7: The logged-out access token is now revoked ---
    // Logout blacklists the presented token; the JwtAuth middleware wired
    // above enforces the blacklist, so the token must stop working.
    let resp = dispatch!(app, || {
        test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", format!("Bearer {}", new_token)))
    });
    assert_eq!(
        resp.status(),
        401,
        "blacklisted access token must be rejected after logout"
    );

    // Do not drop tables here - other tests in the same file need the schema.
    // CI runs `diesel migration run` before tests, and the scheduled migration
    // rollback workflow validates rollback separately.
}

#[actix_web::test]
async fn test_login_invalid_credentials() {
    let db = TestDb::new().await;
    // Schema is already created by diesel migration run in CI.
    // Each test uses its own TestDb pool but shares the same database.

    let config = Arc::new(test_config());
    let r_pool = redis_pool();

    let state = web::Data::new(backend::AppState {
        db: db.pool().clone(),
        redis: r_pool.clone(),
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsRedisState::new(r_pool.clone(), backend::ws::WsLimits::default()),
        jwt_secrets: Arc::new(config.jwt_secrets.clone()),
        otel_provider: None,
    });

    let container = web::Data::new(backend::repositories::AppContainer::new(
        db.pool().clone(),
        r_pool.clone(),
        (*config).clone(),
    ));

    let app = test::init_service(
        App::new()
            .app_data(state)
            .app_data(container)
            .app_data(web::JsonConfig::default().limit(1024 * 1024).error_handler(
                |_error, _request| {
                    backend::errors::AppError::BadRequest("bad request".to_string()).into()
                },
            ))
            .service(
                web::scope("/api/v1/auth").service(backend::controllers::auth_controller::login),
            ),
    )
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(serde_json::json!({
                "email": "nonexistent@example.com",
                "password": "wrongpassword"
            }))
            .to_request(),
    )
    .await;
    let status = resp.status();
    let body: Value = test::read_body_json(resp).await;
    println!("Invalid login: {} - {:?}", status, body);
    assert!(
        status.is_client_error(),
        "Expected client error, got {}: {:?}",
        status,
        body
    );

    // db.drop_all_tables().await;
}

#[actix_web::test]
async fn locked_account_cannot_use_session_refresh_or_login() {
    let db = TestDb::new().await;
    let config = Arc::new(test_config());
    let r_pool = redis_pool();

    let state = web::Data::new(backend::AppState {
        db: db.pool().clone(),
        redis: r_pool.clone(),
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsRedisState::new(r_pool.clone(), backend::ws::WsLimits::default()),
        jwt_secrets: Arc::new(config.jwt_secrets.clone()),
        otel_provider: None,
    });

    let container = web::Data::new(backend::repositories::AppContainer::new(
        db.pool().clone(),
        r_pool.clone(),
        (*config).clone(),
    ));

    let email = format!("locked-{}@example.com", uuid::Uuid::new_v4());
    let password = "StrongPass123!@#";

    let app = test::init_service(
        App::new()
            .app_data(state)
            .app_data(container)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .wrap(backend::middleware::auth::JwtAuth::new(
                            backend::middleware::auth_middleware::JwtAuthConfig::bearer_exempt()
                                .with_token_blacklist(Arc::new(
                                    backend::repositories::access_token_blacklist::AccessTokenBlacklist::new(r_pool.clone()),
                                )),
                        ))
                        .service(backend::controllers::auth_controller::register)
                        .service(backend::controllers::auth_controller::login)
                        .service(backend::controllers::auth_controller::session)
                        .service(backend::controllers::auth_controller::refresh),
                ),
            ),
    )
    .await;

    // --- Register + confirm ---
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(serde_json::json!({
                "email": email,
                "password": password,
                "password_confirmation": password
            }))
            .to_request(),
    )
    .await;
    let register_status = resp.status();
    let body: Value = test::read_body_json(resp).await;
    assert!(register_status.is_success(), "Register failed: {:?}", body);
    let user_id = body["user_id"].as_str().unwrap().to_string();

    {
        let mut conn = db.pool().get().await.expect("Failed to get connection");
        diesel::sql_query("UPDATE users SET confirmed_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(UuidType::parse_str(&user_id).unwrap())
            .execute(&mut *conn)
            .await
            .expect("Failed to confirm user email");
    }

    // --- Login (account still healthy) ---
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(serde_json::json!({ "email": email, "password": password }))
            .to_request(),
    )
    .await;
    let login_status = resp.status();
    let refresh_cookie = extract_cookie(&resp, "refresh_token");
    let login_body: Value = test::read_body_json(resp).await;
    assert!(login_status.is_success(), "Login failed: {:?}", login_body);
    assert!(refresh_cookie.is_some(), "login must set refresh cookie");

    // --- Lock the account (brute-force protection engaged) ---
    {
        let mut conn = db.pool().get().await.expect("Failed to get connection");
        diesel::sql_query("UPDATE users SET locked_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(UuidType::parse_str(&user_id).unwrap())
            .execute(&mut *conn)
            .await
            .expect("Failed to lock user");
    }

    // --- Session with a valid refresh cookie must be rejected ---
    let resp = dispatch!(app, || {
        test::TestRequest::get()
            .uri("/api/v1/auth/session")
            .insert_header((
                "Cookie",
                format!("refresh_token={}", refresh_cookie.clone().unwrap()),
            ))
    });
    assert_eq!(resp.status(), 401, "session must reject a locked account");

    // --- Refresh with a valid refresh cookie must be rejected ---
    let resp = dispatch!(app, || {
        test::TestRequest::post()
            .uri("/api/v1/auth/refresh")
            .insert_header((
                "Cookie",
                format!("refresh_token={}", refresh_cookie.clone().unwrap()),
            ))
    });
    assert_eq!(resp.status(), 401, "refresh must reject a locked account");

    // --- Login with the CORRECT password must return the GENERIC error ---
    // Account-state enumeration resistance: the locked login response must
    // be indistinguishable from a login for a nonexistent user.
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(serde_json::json!({ "email": email, "password": password }))
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), 401, "locked login must be Unauthorized");
    let locked_body: Value = test::read_body_json(resp).await;

    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(serde_json::json!({
                "email": format!("no-such-{}@example.com", uuid::Uuid::new_v4()),
                "password": password
            }))
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), 401);
    let unknown_body: Value = test::read_body_json(resp).await;

    assert_eq!(
        locked_body["error"]["message"], unknown_body["error"]["message"],
        "locked-account login must return the same generic message as unknown-user login"
    );
}

#[actix_web::test]
async fn password_change_revokes_existing_access_tokens() {
    let db = TestDb::new().await;
    let config = Arc::new(test_config());
    let r_pool = redis_pool();

    let state = web::Data::new(backend::AppState {
        db: db.pool().clone(),
        redis: r_pool.clone(),
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsRedisState::new(r_pool.clone(), backend::ws::WsLimits::default()),
        jwt_secrets: Arc::new(config.jwt_secrets.clone()),
        otel_provider: None,
    });

    let container = web::Data::new(backend::repositories::AppContainer::new(
        db.pool().clone(),
        r_pool.clone(),
        (*config).clone(),
    ));

    let email = format!("epoch-{}@example.com", uuid::Uuid::new_v4());
    let password = "StrongPass123!@#";
    let new_password = "NewStrongPass456!@#";

    let app = test::init_service(
        App::new()
            .app_data(state)
            .app_data(container)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .wrap(backend::middleware::auth::JwtAuth::new(
                            backend::middleware::auth_middleware::JwtAuthConfig::bearer_exempt()
                                .with_token_blacklist(Arc::new(
                                    backend::repositories::access_token_blacklist::AccessTokenBlacklist::new(r_pool.clone()),
                                )),
                        ))
                        .service(backend::controllers::auth_controller::register)
                        .service(backend::controllers::auth_controller::login)
                        .service(backend::controllers::auth_controller::me)
                        .service(backend::controllers::auth_controller::change_password),
                ),
            ),
    )
    .await;

    // --- Register + confirm ---
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(serde_json::json!({
                "email": email,
                "password": password,
                "password_confirmation": password
            }))
            .to_request(),
    )
    .await;
    let register_status = resp.status();
    let body: Value = test::read_body_json(resp).await;
    assert!(register_status.is_success(), "Register failed: {:?}", body);
    let user_id = body["user_id"].as_str().unwrap().to_string();

    {
        let mut conn = db.pool().get().await.expect("Failed to get connection");
        diesel::sql_query("UPDATE users SET confirmed_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(UuidType::parse_str(&user_id).unwrap())
            .execute(&mut *conn)
            .await
            .expect("Failed to confirm user email");
    }

    // --- Login ---
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(serde_json::json!({ "email": email, "password": password }))
            .to_request(),
    )
    .await;
    assert!(resp.status().is_success(), "Login failed");
    let body: Value = test::read_body_json(resp).await;
    let access_token = body["access_token"].as_str().unwrap().to_string();

    // The token works before the credential change.
    let resp = dispatch!(app, || {
        test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", format!("Bearer {}", access_token)))
    });
    assert_eq!(resp.status(), 200, "token must work before change-password");

    // --- Change password ---
    let resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/change-password")
            .insert_header(("Authorization", format!("Bearer {}", access_token)))
            .set_json(serde_json::json!({
                "current_password": password,
                "new_password": new_password,
                "password_confirmation": new_password
            }))
            .to_request(),
    )
    .await;
    let change_status = resp.status();
    let body: Value = test::read_body_json(resp).await;
    assert!(
        change_status.is_success(),
        "change-password failed: {:?}",
        body
    );

    // --- The pre-change access token must now be rejected ---
    let resp = dispatch!(app, || {
        test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", format!("Bearer {}", access_token)))
    });
    assert_eq!(
        resp.status(),
        401,
        "pre-change access token must be rejected after password change"
    );
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(
        body["error"]["code"], "TOKEN_REVOKED",
        "epoch revocation must report TOKEN_REVOKED (frontend skips refresh retry on it)"
    );
}

#[actix_web::test]
async fn test_health_endpoint() {
    let db = TestDb::new().await;
    // Schema is already created by diesel migration run in CI.
    let r_pool = redis_pool();

    let config = Arc::new(test_config());
    let state = web::Data::new(backend::AppState {
        db: db.pool().clone(),
        redis: r_pool.clone(),
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsRedisState::new(r_pool.clone(), backend::ws::WsLimits::default()),
        jwt_secrets: Arc::new(config.jwt_secrets.clone()),
        otel_provider: None,
    });

    let app = test::init_service(App::new().app_data(state).service(
        web::scope("/api/v1").service(backend::controllers::health_controller::health_check),
    ))
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::get().uri("/api/v1/health").to_request(),
    )
    .await;
    assert!(resp.status().is_success());
}
