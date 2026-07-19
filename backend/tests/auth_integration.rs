//! Integration tests for the full auth flow.
//!
//! These tests spin up a real Actix-web app with Postgres + Redis and test
//! the complete register → login → access → refresh → logout cycle.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST (default: postgres://postgres:postgres@localhost:5432/test_db)
//! - Redis running at REDIS_URL_TEST (default: redis://127.0.0.1:6379)
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`

use actix_web::{App, HttpMessage, test, web};
use backend::middleware::auth::{ACCESS_TOKEN_USE, Claims};
use chrono::Utc;
use deadpool::managed::Pool;
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid as UuidType;

fn make_claims(sub: UuidType, profile_id: UuidType) -> Claims {
    let now = Utc::now();
    let exp = now + chrono::Duration::hours(1);
    Claims {
        sub,
        profile_id,
        role: 0, // viewer
        token_use: ACCESS_TOKEN_USE.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    }
}

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
    use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

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
        rate_limit_enabled: false,
        argon2_m_cost: 65536,
        argon2_t_cost: 3,
        argon2_p_cost: 1,
        jwt_secrets: vec![backend::config::app_config::JwtSecretKey {
            kid: "test-key-1".to_string(),
            secret: jwt_secret,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            expires_at: None,
        }],
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
                        web::scope("/auth")
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
                    .route(
                        "/health",
                        web::get().to(backend::controllers::health_controller::health_check),
                    ),
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
    let user_id = body["user"]["id"].as_str().unwrap();
    let profile_id = body["user"]["profile_id"].as_str().unwrap();
    println!("Refresh cookie: {:?}", refresh_cookie);

    // Create Claims for authenticated requests
    let claims = make_claims(
        UuidType::parse_str(user_id).unwrap(),
        UuidType::parse_str(profile_id).unwrap(),
    );

    // --- Step 3: Access protected endpoint ---
    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();
    req.extensions_mut().insert(claims.clone());
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: Value = test::read_body_json(resp).await;
    println!("Me: {} - {:?}", status, body);
    assert!(status.is_success(), "Me failed: {:?}", body);
    assert_eq!(body["email"], "test@example.com");

    // --- Step 4: Refresh token ---
    let mut refresh_req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .to_request();
    refresh_req.extensions_mut().insert(claims.clone());
    if let Some(ref cookie) = refresh_cookie {
        refresh_req.headers_mut().insert(
            actix_web::http::header::COOKIE,
            format!("refresh_token={}", cookie).parse().unwrap(),
        );
    }
    let resp = test::call_service(&app, refresh_req).await;
    let status = resp.status();
    let body: Value = test::read_body_json(resp).await;
    println!("Refresh: {} - {:?}", status, body);
    assert!(status.is_success(), "Refresh failed: {:?}", body);

    let new_token = body["access_token"].as_str().unwrap();
    assert_ne!(access_token, new_token, "token should rotate");

    // --- Step 5: Use new token ---
    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", new_token)))
        .to_request();
    req.extensions_mut().insert(claims.clone());
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["email"], "test@example.com");

    // --- Step 6: Logout ---
    let mut logout_req = test::TestRequest::post()
        .uri("/api/v1/auth/logout")
        .insert_header(("Authorization", format!("Bearer {}", new_token)))
        .to_request();
    logout_req.extensions_mut().insert(claims.clone());
    if let Some(ref cookie) = refresh_cookie {
        logout_req.headers_mut().insert(
            actix_web::http::header::COOKIE,
            format!("refresh_token={}", cookie).parse().unwrap(),
        );
    }
    let resp = test::call_service(&app, logout_req).await;
    println!("Logout: {}", resp.status());
    assert!(resp.status().is_success() || resp.status().as_u16() == 204);

    // Note: /auth/me is NOT protected by JwtAuth in production (only /admin is).
    // Token invalidation via blacklist only works when JwtAuth middleware is present.
    // So we don't test token invalidation here since it would require JwtAuth middleware.

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
    });

    let app = test::init_service(
        App::new()
            .app_data(state)
            .service(web::scope("/api/v1").route(
                "/health",
                web::get().to(backend::controllers::health_controller::health_check),
            )),
    )
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::get().uri("/api/v1/health").to_request(),
    )
    .await;
    assert!(resp.status().is_success());
}
