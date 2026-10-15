//! Integration tests for access-token revocation in the JWT middleware.
//!
//! Verifies:
//! - Per-token blacklist rejects revoked tokens (401 TOKEN_REVOKED)
//! - Per-user credential epoch rejects tokens issued before a password/2FA
//!   change (401 TOKEN_REVOKED)
//! - An epoch that predates the token does NOT revoke it
//! - Redis outages fail CLOSED (503) — a token whose revocation status
//!   cannot be checked must never be accepted
//!
//! Prerequisites:
//! - Redis running at REDIS_URL_TEST (default: redis://127.0.0.1:6379)

use std::sync::Arc;

use actix_web::dev::Service;
use actix_web::{App, HttpResponse, test, web};
use deadpool::managed::Pool;
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

use backend::AppState;
use backend::middleware::auth::{JwtAuth, JwtAuthConfig, create_token};
use backend::repositories::access_token_blacklist::{
    AccessTokenBlacklist, hash_token_for_blacklist,
};

mod common;

use common::dispatch;

fn redis_pool() -> deadpool_redis::Pool {
    let redis_url =
        std::env::var("REDIS_URL_TEST").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let mut cfg = RedisConfig::from_url(&redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig::new(5));
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}

/// SAFETY: tests in this file always write the SAME env values, so races
/// between concurrent `set_var` calls are benign (idempotent writes).
fn app_config() -> backend::config::AppConfig {
    unsafe {
        std::env::set_var(
            "JWT_SECRET",
            "test-jwt-secret-key-for-revocation-tests-32b!",
        );
        std::env::set_var(
            "CSRF_SECRET_KEY",
            "test-csrf-secret-key-for-revocation-tests-32b!",
        );
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("MASTER_KEY", "test-master-key-for-revocation-tests-32b!");
        std::env::set_var(
            "BLIND_INDEX_KEY",
            "test-blind-index-key-for-revocation-tests-32b!",
        );
        std::env::set_var(
            "REFRESH_TOKEN_HASH_SALT",
            "test-refresh-salt-for-revocation-tests-32b!",
        );
    }
    backend::config::AppConfig::from_env().expect("failed to build test config")
}

/// App state for the JWT middleware. No database access happens in these
/// tests, so the DB pool never connects (deadpool is lazy).
fn app_state(config: &backend::config::AppConfig) -> web::Data<AppState> {
    let redis = redis_pool();

    let manager =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new("postgres://localhost:5432/dummy");
    let db = Pool::builder(manager)
        .max_size(1)
        .build()
        .expect("Failed to create dummy DB pool");

    web::Data::new(AppState {
        db,
        redis,
        config: Arc::new(config.clone()),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsRedisState::new(redis_pool(), backend::ws::WsLimits::default()),
        jwt_secrets: Arc::new(config.jwt_secrets.clone()),
        otel_provider: None,
    })
}

fn bearer_request(token: &str) -> actix_web::test::TestRequest {
    test::TestRequest::get()
        .uri("/protected")
        .insert_header(("Authorization", format!("Bearer {}", token)))
}

async fn make_token(config: &backend::config::AppConfig) -> (uuid::Uuid, String) {
    let user_id = uuid::Uuid::new_v4();
    let profile_id = uuid::Uuid::new_v4();
    let token = create_token(user_id, profile_id, 0, &config.jwt_secret, 3600)
        .expect("failed to create token");
    (user_id, token)
}

async fn set_epoch_marker(user_id: uuid::Uuid, epoch: u64) {
    let mut conn = redis_pool().get().await.expect("redis connection");
    redis::cmd("SET")
        .arg(format!("credential_epoch:{}", user_id))
        .arg(epoch)
        .arg("EX")
        .arg(900u64)
        .query_async::<()>(&mut conn)
        .await
        .expect("failed to set epoch marker");
}

#[actix_web::test]
async fn epoch_revokes_tokens_issued_before_credential_change() {
    let config = app_config();
    let blacklist = Arc::new(AccessTokenBlacklist::new(redis_pool()));
    let app = test::init_service(
        App::new().app_data(app_state(&config)).service(
            web::scope("/protected")
                .wrap(JwtAuth::new(
                    JwtAuthConfig::new(vec![]).with_token_blacklist(blacklist.clone()),
                ))
                .default_service(web::to(|| async { HttpResponse::Ok().body("ok") })),
        ),
    )
    .await;

    let (user_id, token) = make_token(&config).await;
    let future_epoch = (chrono::Utc::now().timestamp() + 3600) as u64;
    set_epoch_marker(user_id, future_epoch).await;

    let resp = dispatch!(app, || bearer_request(&token));
    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "TOKEN_REVOKED");
}

#[actix_web::test]
async fn epoch_predating_token_does_not_revoke_it() {
    let config = app_config();
    let blacklist = Arc::new(AccessTokenBlacklist::new(redis_pool()));
    let app = test::init_service(
        App::new().app_data(app_state(&config)).service(
            web::scope("/protected")
                .wrap(JwtAuth::new(
                    JwtAuthConfig::new(vec![]).with_token_blacklist(blacklist.clone()),
                ))
                .default_service(web::to(|| async { HttpResponse::Ok().body("ok") })),
        ),
    )
    .await;

    let (user_id, token) = make_token(&config).await;
    let past_epoch = (chrono::Utc::now().timestamp() - 3600) as u64;
    set_epoch_marker(user_id, past_epoch).await;

    let resp = dispatch!(app, || bearer_request(&token));
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn blacklisted_token_is_rejected() {
    let config = app_config();
    let blacklist = Arc::new(AccessTokenBlacklist::new(redis_pool()));
    let app = test::init_service(
        App::new().app_data(app_state(&config)).service(
            web::scope("/protected")
                .wrap(JwtAuth::new(
                    JwtAuthConfig::new(vec![]).with_token_blacklist(blacklist.clone()),
                ))
                .default_service(web::to(|| async { HttpResponse::Ok().body("ok") })),
        ),
    )
    .await;

    let (_, token) = make_token(&config).await;
    let token_hash = hash_token_for_blacklist(&token);
    blacklist
        .add(&token_hash, 60)
        .await
        .expect("failed to blacklist token");

    let resp = dispatch!(app, || bearer_request(&token));
    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "TOKEN_REVOKED");
}

#[actix_web::test]
async fn redis_outage_fails_closed() {
    let config = app_config();

    // Nothing listens on 6399: every revocation lookup fails.
    let dead_pool = RedisConfig::from_url("redis://127.0.0.1:6399")
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create dead Redis pool");
    let blacklist = Arc::new(AccessTokenBlacklist::new(dead_pool));

    let app = test::init_service(
        App::new().app_data(app_state(&config)).service(
            web::scope("/protected")
                .wrap(JwtAuth::new(
                    JwtAuthConfig::new(vec![]).with_token_blacklist(blacklist.clone()),
                ))
                .default_service(web::to(|| async { HttpResponse::Ok().body("ok") })),
        ),
    )
    .await;

    let (_, token) = make_token(&config).await;

    let resp = dispatch!(app, || bearer_request(&token));
    assert_eq!(
        resp.status(),
        actix_web::http::StatusCode::SERVICE_UNAVAILABLE
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "AUTH_STATUS_UNAVAILABLE");
}

#[actix_web::test]
async fn valid_token_without_revocation_record_passes() {
    let config = app_config();
    let blacklist = Arc::new(AccessTokenBlacklist::new(redis_pool()));
    let app = test::init_service(
        App::new().app_data(app_state(&config)).service(
            web::scope("/protected")
                .wrap(JwtAuth::new(
                    JwtAuthConfig::new(vec![]).with_token_blacklist(blacklist.clone()),
                ))
                .default_service(web::to(|| async { HttpResponse::Ok().body("ok") })),
        ),
    )
    .await;

    let (_, token) = make_token(&config).await;
    let resp = dispatch!(app, || bearer_request(&token));
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}
