//! Integration tests for refresh-token rotation reuse detection.
//!
//! Verifies the OAuth 2.1 half of the rotation design:
//! - A rotated token presented again is a reuse, not a retry: the request is
//!   refused, every refresh token of the user is revoked (the whole family
//!   dies, including the token the legitimate client is holding), and the
//!   access tokens minted before the event stop being accepted.
//! - An unknown token is a plain rejection that touches nothing.
//!
//! Prerequisites:
//! - Postgres at DATABASE_URL_TEST (default:
//!   postgres://postgres:postgres@localhost:5432/test_db) with migrations
//!   applied
//! - Redis at REDIS_URL_TEST (default: redis://127.0.0.1:6379)

use std::sync::Arc;

use actix_web::dev::{Service, ServiceResponse};
use actix_web::{App, test, web};
use deadpool::managed::Pool;
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel::Connection;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use serde_json::Value;
use uuid::Uuid as UuidType;

use backend::AppState;

mod common;

use common::dispatch;

fn database_url() -> String {
    std::env::var("DATABASE_URL_TEST")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/test_db".to_string())
}

fn redis_pool() -> deadpool_redis::Pool {
    let redis_url =
        std::env::var("REDIS_URL_TEST").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let mut cfg = RedisConfig::from_url(&redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig::new(5));
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}

fn db_pool() -> Pool<AsyncDieselConnectionManager<AsyncPgConnection>> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url());
    Pool::builder(manager)
        .max_size(4)
        .build()
        .expect("Failed to create DB pool")
}

fn extract_cookie<B>(resp: &ServiceResponse<B>, name: &str) -> Option<String> {
    resp.headers()
        .get(actix_web::http::header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value.split(';').next().and_then(|pair| {
                let (key, cookie_value) = pair.split_once('=')?;
                (key == name).then(|| cookie_value.to_string())
            })
        })
}

/// The canonical RBAC catalog, through a blocking connection: the service is
/// synchronous Diesel, the same call the seeder and the other suites make.
fn seed_catalog() {
    let mut conn = diesel::PgConnection::establish(&database_url())
        .expect("Failed to connect for the catalog seed");
    backend::services::rbac_catalog_service::ensure_canonical_catalog(&mut conn)
        .expect("catalog seed run failed");
}

/// The app wired the way the production `/auth` scope is: CSRF innermost,
/// JwtAuth outermost with the shared revocation store. Expands to a
/// `(app, pool)` tuple so each test owns its own wiring.
macro_rules! reuse_app {
    () => {{
        seed_catalog();
        let pool = db_pool();
        let config = Arc::new(backend::config::test_config::test_config());
        let redis = redis_pool();

        let state = web::Data::new(AppState {
            db: pool.clone(),
            redis: redis.clone(),
            config: config.clone(),
            metrics: Arc::new(backend::services::metrics_service::MetricsRegistry::new()),
            ws: backend::ws::WsRedisState::new(redis.clone(), backend::ws::WsLimits::default()),
            jwt_secrets: Arc::new(config.jwt_secrets.clone()),
            otel_provider: None,
        });
        let container = web::Data::new(backend::repositories::AppContainer::new(
            pool.clone(),
            redis.clone(),
            (*config).clone(),
        ));
        let ws_state = web::Data::new(backend::ws::WsRedisState::new(
            redis.clone(),
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
                                .wrap(backend::middleware::csrf_protection::CsrfProtection::new(
                                    vec![],
                                ))
                                .wrap(backend::middleware::auth::JwtAuth::new(
                                    backend::middleware::auth_middleware::JwtAuthConfig::bearer_exempt()
                                        .with_token_blacklist(Arc::new(
                                            backend::repositories::access_token_blacklist::AccessTokenBlacklist::new(redis.clone()),
                                        )),
                                ))
                                .service(backend::controllers::auth_controller::register)
                                .service(backend::controllers::auth_controller::login)
                                .service(backend::controllers::auth_controller::session)
                                .service(backend::controllers::auth_controller::refresh)
                                .service(backend::controllers::auth_controller::logout)
                                .service(backend::controllers::auth_controller::me),
                        ),
                ),
        )
        .await;

        (app, pool)
    }};
}

/// Register a fresh subject, confirm it, log it in: returns the access token
/// and the refresh cookie. The email carries a uuid so repeated runs on a
/// shared database never collide on the blind index.
macro_rules! register_and_login {
    ($app:expr, $pool:expr) => {{
        let email = format!("reuse-{}@example.com", UuidType::new_v4());
        let password = "StrongPass123!@#";

        let resp = test::call_service(
            &$app,
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
        let register_body: Value = test::read_body_json(resp).await;
        let user_id = register_body["user_id"]
            .as_str()
            .expect("register names the user")
            .to_string();

        {
            let mut conn = $pool.get().await.expect("Failed to get connection");
            diesel::sql_query("UPDATE users SET confirmed_at = NOW() WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(UuidType::parse_str(&user_id).unwrap())
                .execute(&mut *conn)
                .await
                .expect("Failed to confirm the user");
        }

        let resp = test::call_service(
            &$app,
            test::TestRequest::post()
                .uri("/api/v1/auth/login")
                .set_json(serde_json::json!({ "email": email, "password": password }))
                .to_request(),
        )
        .await;
        let login_status = resp.status();
        let refresh_cookie = extract_cookie(&resp, "refresh_token").expect("login sets the cookie");
        let login_body: Value = test::read_body_json(resp).await;
        assert!(login_status.is_success(), "login failed");
        let access_token = login_body["access_token"]
            .as_str()
            .expect("login mints one")
            .to_string();

        (access_token, refresh_cookie)
    }};
}

/// The CSRF pair the refresh endpoint requires: a GET seeds the cookie, the
/// header echoes it.
macro_rules! csrf_pair {
    ($app:expr, $refresh_cookie:expr) => {{
        let resp = test::call_service(
            &$app,
            test::TestRequest::get()
                .uri("/api/v1/auth/session")
                .insert_header(("Cookie", format!("refresh_token={}", $refresh_cookie)))
                .to_request(),
        )
        .await;
        assert!(resp.status().is_success(), "session GET failed");
        extract_cookie(&resp, "csrf_token").expect("session GET sets the csrf cookie")
    }};
}

macro_rules! call_refresh {
    ($app:expr, $refresh_cookie:expr, $csrf:expr) => {
        dispatch!($app, || {
            test::TestRequest::post()
                .uri("/api/v1/auth/refresh")
                .insert_header((
                    "Cookie",
                    format!("refresh_token={}; csrf_token={}", $refresh_cookie, $csrf),
                ))
                .insert_header(("csrf-token", $csrf.to_string()))
        })
    };
}

macro_rules! call_me {
    ($app:expr, $access_token:expr) => {
        dispatch!($app, || {
            test::TestRequest::get()
                .uri("/api/v1/auth/me")
                .insert_header(("Authorization", format!("Bearer {}", $access_token)))
        })
    };
}

#[actix_web::test]
async fn a_replayed_refresh_token_kills_the_whole_family() {
    let (app, pool) = reuse_app!();
    let (access_token, first_cookie) = register_and_login!(app, pool);

    // The credential epoch is recorded in whole seconds and the token's iat is
    // compared against it with a strict less-than: a reuse landing in the same
    // second as the login treats both as simultaneous and keeps the token. The
    // whole flow runs in milliseconds, so cross a second boundary first.
    tokio::time::sleep(std::time::Duration::from_millis(1_100)).await;

    // The legitimate rotation: the first cookie becomes the second one.
    let csrf = csrf_pair!(app, first_cookie);
    let resp = call_refresh!(app, first_cookie, csrf);
    assert!(
        resp.status().is_success(),
        "the first rotation must succeed"
    );
    let second_cookie = extract_cookie(&resp, "refresh_token").expect("rotation sets a new cookie");
    assert_ne!(
        second_cookie, first_cookie,
        "rotation must mint a new token, not echo the old one"
    );

    // The replay: the already-rotated cookie comes back. This is the reuse the
    // audit found unreported: the request must be refused.
    let resp = call_refresh!(app, first_cookie, csrf);
    assert_eq!(
        resp.status(),
        401,
        "a replayed refresh token must be refused"
    );

    // The family dies with it: the token the legitimate client now holds is
    // revoked as well, because the lineage is compromised by definition.
    let resp = call_refresh!(app, second_cookie, csrf);
    assert_eq!(
        resp.status(),
        401,
        "the rotated token of the same family must be revoked too"
    );

    // And the access tokens minted before the event stop being accepted: the
    // reuse bumps the credential epoch, which the JWT middleware reads.
    let resp = call_me!(app, access_token);
    assert_eq!(
        resp.status(),
        401,
        "the access token must not outlive the revoked family"
    );
}

#[actix_web::test]
async fn an_unknown_refresh_token_is_refused_without_touching_the_family() {
    let (app, pool) = reuse_app!();
    let (access_token, first_cookie) = register_and_login!(app, pool);
    let csrf = csrf_pair!(app, first_cookie);

    // A cookie no row matches: the plain rejection, and the real session of
    // the user stays alive, because nothing was revoked.
    let resp = call_refresh!(app, "not-a-real-token", csrf);
    assert_eq!(resp.status(), 401, "an unknown token is refused");

    let resp = call_refresh!(app, first_cookie, csrf);
    assert!(
        resp.status().is_success(),
        "the real session must survive an unknown-token attempt"
    );

    let resp = call_me!(app, access_token);
    assert_eq!(resp.status(), 200, "the access token stays valid");
}
