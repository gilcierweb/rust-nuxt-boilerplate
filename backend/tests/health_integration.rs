//! Integration tests for the readiness endpoint.
//!
//! Verifies that dependency error strings - which describe internal
//! infrastructure - are only returned to callers holding a valid internal API
//! key. The route is reachable without one, so an anonymous probe must see the
//! status and nothing more.

use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::{App, test, web};
use deadpool::managed::Pool;
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use serde_json::Value;

use backend::AppState;
use backend::config::AppConfig;
use backend::controllers::health_controller::config as health_routes;

/// Deliberately unreachable dependencies: connection is refused immediately,
/// so the probes report "down" deterministically in every environment.
const UNREACHABLE_DB_URL: &str = "postgres://user:pass@127.0.0.1:1/health_probe_missing";
const UNREACHABLE_REDIS_URL: &str = "redis://127.0.0.1:1";
const INTERNAL_API_KEY: &str = "internal-key-for-health-tests";

fn config_with_api_keys() -> AppConfig {
    // SAFETY: these tests run single-threaded; the env vars are only read
    // during AppConfig::from_env() construction.
    unsafe {
        std::env::set_var("DATABASE_URL", UNREACHABLE_DB_URL);
        std::env::set_var("REDIS_URL", UNREACHABLE_REDIS_URL);
        std::env::set_var(
            "JWT_SECRET",
            "test-jwt-secret-key-for-integration-tests-32b!",
        );
        std::env::set_var(
            "MASTER_KEY",
            "dGVzdC1tYXN0ZXIta2V5LWZvci1pbnRlZ3JhdGlvbi10ZXN0cw==",
        );
        std::env::set_var(
            "BLIND_INDEX_KEY",
            "dGVzdC1ibGluZC1pbmRleC1rZXktZm9yLWludGVncmF0aW9uLXRlc3Rz",
        );
        std::env::set_var(
            "CSRF_SECRET_KEY",
            "test-csrf-secret-key-for-integration-tests-32b!",
        );
        std::env::set_var(
            "REFRESH_TOKEN_HASH_SALT",
            "test-refresh-token-hash-salt-for-integration-32b!",
        );
        std::env::set_var("INTERNAL_API_KEYS", INTERNAL_API_KEY);
    }
    AppConfig::from_env().expect("failed to build test config")
}

fn app_state(config: AppConfig) -> web::Data<AppState> {
    let db: Pool<AsyncDieselConnectionManager<AsyncPgConnection>> = Pool::builder(
        AsyncDieselConnectionManager::new(UNREACHABLE_DB_URL),
    )
    .max_size(1)
    .build()
    .expect("db pool");
    let redis = RedisConfig::from_url(UNREACHABLE_REDIS_URL)
        .create_pool(Some(Runtime::Tokio1))
        .expect("redis pool");

    let config = Arc::new(config);
    web::Data::new(AppState {
        db,
        redis: redis.clone(),
        metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
        ws: backend::ws::WsRedisState::new(
            redis.clone(),
            backend::ws::WsLimits::default(),
        ),
        jwt_secrets: Arc::new(config.jwt_secrets.clone()),
        config,
        otel_provider: None,
    })
}

#[actix_web::test]
async fn health_hides_dependency_errors_without_an_api_key() {
    let app = test::init_service(
        App::new()
            .app_data(app_state(config_with_api_keys()))
            // Mirror `routes::router`: the readiness handler is registered
            // relative, inside the /api/v1 scope.
            .service(web::scope("/api/v1").configure(health_routes)),
    )
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::get().uri("/api/v1/health").to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["db"]["status"], "down");
    assert_eq!(body["redis"]["status"], "down");
    assert!(
        body["db"]["error"].is_null(),
        "an unauthenticated caller must not see the driver error, got: {}",
        body["db"]["error"]
    );
    assert!(body["redis"]["error"].is_null());
}

#[actix_web::test]
async fn health_exposes_dependency_errors_with_a_valid_api_key() {
    let app = test::init_service(
        App::new()
            .app_data(app_state(config_with_api_keys()))
            // Mirror `routes::router`: the readiness handler is registered
            // relative, inside the /api/v1 scope.
            .service(web::scope("/api/v1").configure(health_routes)),
    )
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/health")
            .insert_header(("x-api-key", INTERNAL_API_KEY))
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["db"]["status"], "down");
    assert!(
        body["db"]["error"].is_string(),
        "an authenticated operator must get the driver error to debug with"
    );
}

#[actix_web::test]
async fn health_treats_a_wrong_api_key_as_anonymous() {
    let app = test::init_service(
        App::new()
            .app_data(app_state(config_with_api_keys()))
            // Mirror `routes::router`: the readiness handler is registered
            // relative, inside the /api/v1 scope.
            .service(web::scope("/api/v1").configure(health_routes)),
    )
    .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/health")
            .insert_header(("x-api-key", "not-the-internal-key"))
            .to_request(),
    )
    .await;

    let body: Value = test::read_body_json(resp).await;
    assert!(
        body["db"]["error"].is_null(),
        "a wrong key must not unlock the error detail"
    );
}
