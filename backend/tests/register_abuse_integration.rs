//! Integration tests for the anti-bot budgets that protect the registration
//! endpoint.
//!
//! These tests drive the real `RateLimiter` middleware against a live Redis,
//! with the same scope structure the production router mounts: a scope-wide
//! limiter on `/api/v1` and a nested limiter on `/auth` that charges only the
//! `Default` category (the single-charge rule). They prove, over HTTP:
//!
//! - rotating the last octet of an IPv4 address hits the **subnet aggregate**
//!   ceiling, even though every individual address stays under the per-IP
//!   budget;
//! - registrations from unrelated networks still share the **global daily**
//!   ceiling, so a distributed campaign cannot poison the users table;
//! - a spoofed leftmost `X-Forwarded-For` entry cannot mint a fresh budget —
//!   the regression that made 100+ registrations per minute possible;
//! - the strict budget is charged **once** per request, not twice by the
//!   nested and the scope-wide limiter.
//!
//! The routes answer with a stub handler: the limiter decides on the request
//! path and method alone, so the tests exercise the middleware exactly as the
//! production wiring does, without creating users or hashing passwords.
//!
//! Prerequisites: Redis running at REDIS_URL_TEST (default: redis://127.0.0.1:6379).

use actix_web::{App, HttpResponse, test, web};
use deadpool::managed::Pool;
use deadpool_redis::{Config as RedisConfig, Runtime};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

use backend::middleware::rate_limit_middleware::{RATE_API, RATE_AUTH, RateLimiter};

/// Per-IP budget of the strict category (`RATE_AUTH_STRICT`).
const PER_IP_LIMIT: usize = 10;

/// Subnet aggregate budget of the strict category (`RATE_AUTH_STRICT_SUBNET`).
const SUBNET_LIMIT: usize = 30;

/// Network of the trusted proxy the requests arrive from.
const TRUSTED_PROXY: &str = "10.0.0.1:40000";

fn redis_pool() -> deadpool_redis::Pool {
    let redis_url =
        std::env::var("REDIS_URL_TEST").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let mut cfg = RedisConfig::from_url(&redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig::new(5));
    cfg.create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}

/// A database pool that is never used: these tests do not touch the database,
/// but the middleware reads its configuration from the application container.
fn unused_db_pool() -> backend::DBPool {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
        "postgres://postgres:postgres@localhost:5432/unused_limiter_db",
    );
    Pool::builder(manager)
        .max_size(1)
        .build()
        .expect("Failed to create pool")
}

/// Configuration with the limiter enabled and one trusted proxy network.
fn limiter_config(register_daily_limit: u64) -> backend::config::AppConfig {
    use backend::config::app_config::{Environment, JwtSecretKey};

    let jwt_secret =
        "test-slimmer-jwt-secret-that-is-long-enough-for-hmac-sha256-signing!!".to_string();

    backend::config::AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        https_port: 8443,
        tls_enabled: false,
        tls_cert_path: String::new(),
        tls_key_path: String::new(),
        frontend_url: "http://localhost:3000".to_string(),
        environment: Environment::Test,
        database_url: "postgres://postgres:postgres@localhost:5432/unused_limiter_db".to_string(),
        db_pool_size: 4,
        db_pool_min_idle: Some(1),
        db_pool_max_lifetime_secs: Some(1800),
        db_pool_idle_timeout_secs: Some(600),
        db_pool_connection_timeout_secs: 10,
        db_statement_timeout_secs: 30,
        redis_url: std::env::var("REDIS_URL_TEST")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        redis_pool_size: 5,
        jwt_secret: jwt_secret.clone(),
        jwt_public_key: None,
        jwt_access_expiry_secs: 3600,
        jwt_refresh_expiry_secs: 3600,
        master_key: "q0J4u1cHcVtvRUFORlJ5WmpOMkF3WmFYc0xHRnVI=".to_string(),
        blind_index_key: "zM2r8pL4sT7wY0aB3cD6eF9gH1jK5nQ8sU2vX5zA=".to_string(),
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
        csrf_secret_key: jwt_secret.clone(),
        refresh_token_hash_salt: "test-salt-for-refresh-tokens-16b!".to_string(),
        account_lockout_duration_secs: 15 * 60,
        pix_webhook_secret: String::new(),
        rate_limit_enabled: true,
        captcha_provider: backend::security::captcha::CaptchaProvider::None,
        captcha_secret: None,
        captcha_verify_url: None,
        disposable_email_blocking: false,
        extra_disposable_domains: vec![],
        email_mx_check_enabled: false,
        email_dns_resolver_url: None,
        register_daily_limit,
        register_default_role: "customer".to_string(),
        argon2_m_cost: 65536,
        argon2_t_cost: 3,
        argon2_p_cost: 1,
        trusted_proxies: vec!["10.0.0.0/8".parse().unwrap()],
        jwt_secrets: vec![JwtSecretKey {
            kid: "test-key-1".to_string(),
            secret: jwt_secret,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            expires_at: None,
        }],
        cookie_secure_override: false,
    }
}

/// Stub handler: the limiter decides on path and method alone.
async fn stub_handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// Build the application with the production limiter structure: a scope-wide
/// limiter on `/api/v1` (which charges every category, including the strict
/// one) and a nested limiter on `/auth` that charges only `Default`.
macro_rules! init_limiter_app {
    ($config:expr) => {{
        let r_pool = redis_pool();
        let container = web::Data::new(backend::repositories::AppContainer::new(
            unused_db_pool(),
            r_pool.clone(),
            $config,
        ));
        test::init_service(
            App::new().app_data(container).service(
                web::scope("/api/v1")
                    .wrap(RateLimiter::new(r_pool.clone(), RATE_API))
                    .service(
                        web::scope("/auth")
                            .wrap(RateLimiter::default_only(r_pool.clone(), RATE_AUTH))
                            .route("/register", web::post().to(stub_handler))
                            .route("/login", web::post().to(stub_handler)),
                    ),
            ),
        )
        .await
    }};
}

/// Remove budget buckets so a scenario starts from a known counter and leaves
/// none behind.
async fn clear_buckets(pool: &deadpool_redis::Pool, keys: &[String]) {
    let mut conn = pool.get().await.expect("redis connection");
    for key in keys {
        redis::cmd("DEL")
            .arg(key)
            .query_async::<i64>(&mut conn)
            .await
            .expect("bucket cleanup");
    }
}

/// Send a strict-category request as if it came through the trusted proxy,
/// with `client_ip` as the address the proxy chain reports. A macro (not a
/// function) keeps the request type out of the signature — the test app's
/// service type cannot be named from an integration test.
macro_rules! strict_request {
    ($app:expr, $path:expr, $client_ip:expr) => {
        test::call_service(
            &$app,
            test::TestRequest::post()
                .uri($path)
                .peer_addr(TRUSTED_PROXY.parse().unwrap())
                .insert_header(("x-forwarded-for", $client_ip))
                .to_request(),
        )
        .await
        .status()
        .as_u16()
    };
}

#[actix_web::test]
async fn rotating_addresses_inside_one_subnet_hit_the_aggregate_ceiling() {
    let pool = redis_pool();
    let app = init_limiter_app!(limiter_config(2000));

    // Clean slate: the subnet budget must start at zero for this scenario.
    let subnet_key = "rl:auth_strict_mask:198.51.100.mask".to_string();
    let mut keys = vec![subnet_key.clone()];
    for octet in 1..=(SUBNET_LIMIT + 1) as u8 {
        keys.push(format!("rl:auth_strict:198.51.100.{octet}"));
    }
    clear_buckets(&pool, &keys).await;

    // Every request uses a different address of the same /24, so the per-IP
    // budget (10/min) is never reached: only the subnet aggregate can stop it.
    let mut statuses = Vec::new();
    for octet in 1..=(SUBNET_LIMIT + 1) as u8 {
        statuses.push(strict_request!(
            app,
            "/api/v1/auth/login",
            format!("198.51.100.{octet}")
        ));
    }

    assert!(
        statuses[..SUBNET_LIMIT].iter().all(|status| *status == 200),
        "the first {SUBNET_LIMIT} requests must pass — each address is under its own budget, got {statuses:?}"
    );
    assert_eq!(
        statuses[SUBNET_LIMIT],
        429,
        "the {}-th request must be throttled by the subnet aggregate, got {statuses:?}",
        SUBNET_LIMIT + 1
    );

    clear_buckets(&pool, &keys).await;
}

#[actix_web::test]
async fn registrations_from_unrelated_networks_share_the_global_daily_ceiling() {
    let pool = redis_pool();
    let app = init_limiter_app!(limiter_config(3));

    // Four different networks: distinct per-IP budgets, distinct subnet
    // budgets. Only the global daily bucket is shared.
    let networks = ["203.0.113.1", "192.0.2.1", "198.18.0.1", "203.0.114.1"];
    let mut keys = vec!["rl:register_global".to_string()];
    for ip in networks {
        keys.push(format!("rl:auth_strict:{ip}"));
    }
    keys.extend([
        "rl:auth_strict_mask:203.0.113.mask".to_string(),
        "rl:auth_strict_mask:192.0.2.mask".to_string(),
        "rl:auth_strict_mask:198.18.0.mask".to_string(),
        "rl:auth_strict_mask:203.0.114.mask".to_string(),
    ]);
    clear_buckets(&pool, &keys).await;

    let mut statuses = Vec::new();
    for ip in networks {
        statuses.push(strict_request!(app, "/api/v1/auth/register", ip));
    }

    assert!(
        statuses[..3].iter().all(|status| *status == 200),
        "the first three registrations must pass, got {statuses:?}"
    );
    assert_eq!(
        statuses[3], 429,
        "the fourth registration must be throttled by the global daily ceiling, got {statuses:?}"
    );

    clear_buckets(&pool, &keys).await;
}

#[actix_web::test]
async fn a_spoofed_forwarded_entry_cannot_mint_a_fresh_budget() {
    let pool = redis_pool();
    let app = init_limiter_app!(limiter_config(2000));

    // The attacker controls the leftmost entry and sends a new one on every
    // request; the proxy chain reports the real client on the right. Walking
    // the chain right to left always resolves to the same real client, so all
    // requests land in the same per-IP budget.
    let keys = vec![
        "rl:auth_strict:198.19.0.44".to_string(),
        "rl:auth_strict_mask:198.19.0.mask".to_string(),
    ];
    clear_buckets(&pool, &keys).await;

    let mut statuses = Vec::new();
    for spoofed in 1..=(PER_IP_LIMIT + 2) as u8 {
        let chain = format!("198.20.0.{spoofed}, 198.19.0.44");
        statuses.push(strict_request!(app, "/api/v1/auth/login", chain));
    }

    assert!(
        statuses[..PER_IP_LIMIT].iter().all(|status| *status == 200),
        "the first {PER_IP_LIMIT} requests must pass — the strict budget is charged once per request, got {statuses:?}"
    );
    assert_eq!(
        statuses[PER_IP_LIMIT],
        429,
        "the {}-th request must be throttled: a spoofed leftmost entry must not mint a new budget, got {statuses:?}",
        PER_IP_LIMIT + 1
    );

    clear_buckets(&pool, &keys).await;
}
