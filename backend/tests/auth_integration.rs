//! Integration tests for the full auth flow.
//!
//! These tests spin up a real Actix-web app with Postgres + Redis and test
//! the complete register → login → access → refresh → logout cycle.
//!
//! Prerequisites:
//! - Postgres running at DATABASE_URL_TEST (default: postgres://postgres:postgres@localhost:5432/test_db)
//! - Redis running at REDIS_URL_TEST (default: redis://127.0.0.1:6379)
//! - Migrations applied: `diesel migration run --database-url $DATABASE_URL_TEST`

use actix_web::{App, test, web};
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use diesel::RunQueryDsl;
use serde_json::Value;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

struct TestDb {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl TestDb {
    fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL_TEST")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Arc::new(
            Pool::builder()
                .max_size(4)
                .build(manager)
                .expect("Failed to create pool"),
        );
        Self { pool }
    }

    fn pool(&self) -> Arc<Pool<ConnectionManager<PgConnection>>> {
        self.pool.clone()
    }

    fn drop_all_tables(&self) {
        let mut conn = self.pool.get().expect("Failed to get connection");
        diesel::sql_query("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
            .execute(&mut conn)
            .expect("Failed to drop all tables");
    }

    fn create_auth_schema(&self) {
        let mut conn = self.pool.get().expect("Failed to get connection");
        diesel::sql_query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                email_encrypted BYTEA NOT NULL,
                email_blind_index BYTEA NOT NULL,
                encrypted_password TEXT NOT NULL,
                confirmed_at TIMESTAMPTZ,
                locked_at TIMESTAMPTZ,
                failed_attempts INTEGER DEFAULT 0,
                last_failed_at TIMESTAMPTZ,
                otp_secret TEXT,
                otp_enabled_at TIMESTAMPTZ,
                encryption_key_version INTEGER DEFAULT 1,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE TABLE IF NOT EXISTS profiles (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
                first_name TEXT, last_name TEXT, slug TEXT UNIQUE,
                cpf_encrypted BYTEA, cpf_blind_index BYTEA,
                phone_encrypted BYTEA, phone_blind_index BYTEA,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE TABLE IF NOT EXISTS roles (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(50) NOT NULL, resource_type VARCHAR(50), resource_id UUID
            );
            CREATE TABLE IF NOT EXISTS users_roles (
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
                PRIMARY KEY (user_id, role_id)
            );
            CREATE TABLE IF NOT EXISTS refresh_tokens (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                token_hash TEXT NOT NULL, device_info TEXT, ip_address TEXT,
                expires_at TIMESTAMPTZ NOT NULL, revoked_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE TABLE IF NOT EXISTS audit_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                actor_id UUID, actor_role_snapshot TEXT, action VARCHAR(100) NOT NULL,
                resource_type VARCHAR(100), resource_id UUID, metadata JSONB,
                ip_address TEXT, user_agent TEXT, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            INSERT INTO roles (id, name) VALUES
                ('a0000000-0000-0000-0000-000000000001', 'admin'),
                ('a0000000-0000-0000-0000-000000000002', 'operator'),
                ('a0000000-0000-0000-0000-000000000003', 'viewer')
            ON CONFLICT DO NOTHING;
            "#,
        )
        .execute(&mut conn)
        .expect("Failed to create test schema");
    }
}

fn redis_pool() -> deadpool_redis::Pool {
    let redis_url = std::env::var("REDIS_URL_TEST")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let mut cfg = RedisConfig::from_url(&redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig::new(5));
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}

fn test_config() -> backend::config::AppConfig {
    use backend::config::app_config::Environment;

    let database_url = std::env::var("DATABASE_URL_TEST")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string());
    let redis_url = std::env::var("REDIS_URL_TEST")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "test-jwt-secret-key-for-integration-tests-32b!".to_string());
    let csrf_key = jwt_secret.clone();
    let master_key = std::env::var("MASTER_KEY").unwrap_or_else(|_| {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&[0xABu8; 32])
    });
    let blind_index_key = std::env::var("BLIND_INDEX_KEY").unwrap_or_else(|_| {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&[0xCDu8; 32])
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
        db_statement_timeout_secs: Some(30),
        redis_url,
        redis_pool_size: 5,
        jwt_secret,
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
    }
}

fn extract_cookie(response: &actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>, name: &str) -> Option<String> {
    response.headers().get("set-cookie")?.to_str().ok()?.lines()
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
    let db = TestDb::new();
    db.drop_all_tables();
    db.create_auth_schema();

    let config = Arc::new(test_config());
    let r_pool = redis_pool();

    let state = web::Data::new(backend::AppState {
        db: db.pool().as_ref().clone(),
        redis: r_pool.clone(),
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsState::new(),
    });

    let container = web::Data::new(backend::repositories::AppContainer::new(
        db.pool().as_ref().clone(),
        r_pool.clone(),
        (*config).clone(),
    ));

    let ws_state = web::Data::new(backend::ws::WsState::new());

    let app = test::init_service(
        App::new()
            .app_data(state)
            .app_data(container)
            .app_data(ws_state)
            .app_data(
                web::JsonConfig::default()
                    .limit(1024 * 1024)
                    .error_handler(|_error, _request| {
                        backend::errors::AppError::BadRequest("bad request".to_string()).into()
                    }),
            )
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
                    .route("/health", web::get().to(backend::controllers::health_controller::health_check)),
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

    // --- Step 3: Access protected endpoint ---
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

    // --- Step 4: Refresh token ---
    let mut refresh_req = test::TestRequest::post()
        .uri("/api/v1/auth/refresh")
        .to_request();
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

    // --- Step 7: Verify token is invalid ---
    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", format!("Bearer {}", new_token)))
            .to_request(),
    )
    .await;
    println!("After logout: {}", resp.status());
    assert!(resp.status().is_client_error(), "token should be invalid after logout");

    db.drop_all_tables();
}

#[actix_web::test]
async fn test_login_invalid_credentials() {
    let db = TestDb::new();
    db.drop_all_tables();
    db.create_auth_schema();

    let config = Arc::new(test_config());
    let r_pool = redis_pool();

    let state = web::Data::new(backend::AppState {
        db: db.pool().as_ref().clone(),
        redis: r_pool.clone(),
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsState::new(),
    });

    let container = web::Data::new(backend::repositories::AppContainer::new(
        db.pool().as_ref().clone(),
        r_pool.clone(),
        (*config).clone(),
    ));

    let app = test::init_service(
        App::new()
            .app_data(state)
            .app_data(container)
            .app_data(
                web::JsonConfig::default()
                    .limit(1024 * 1024)
                    .error_handler(|_error, _request| {
                        backend::errors::AppError::BadRequest("bad request".to_string()).into()
                    }),
            )
            .service(
                web::scope("/api/v1/auth")
                    .service(backend::controllers::auth_controller::login),
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
    println!("Invalid login: {}", resp.status());
    assert!(resp.status().is_client_error());

    db.drop_all_tables();
}

#[actix_web::test]
async fn test_health_endpoint() {
    let db = TestDb::new();
    let r_pool = redis_pool();

    let config = Arc::new(test_config());
    let state = web::Data::new(backend::AppState {
        db: db.pool().as_ref().clone(),
        redis: r_pool.clone(),
        config: config.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsState::new(),
    });

    let app = test::init_service(
        App::new()
            .app_data(state)
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(backend::controllers::health_controller::health_check)),
            ),
    )
    .await;

    let resp = test::call_service(&app, test::TestRequest::get().uri("/api/v1/health").to_request()).await;
    assert!(resp.status().is_success());
}
