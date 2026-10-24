//! Integration tests for the rate limiter, against a live Redis.
//!
//! Verifies:
//! - the counter the Lua script returns enforces max_requests, so anonymous
//!   traffic is throttled instead of being let through on a healthy Redis
//! - the quota headers report the limit, the remaining budget and the seconds
//!   left in the window, on both the allowed and the throttled response
//! - the bucket key carries a TTL, so a window cannot leak its counter forever
//! - brute-force endpoints fail closed while Redis is unavailable, and general
//!   endpoints degrade to fail open once the circuit breaker opens
//!
//! The live scenarios connect to REDIS_URL and default to localhost:6379. Each
//! scenario reserves its own bucket prefix, so tests that run in parallel
//! threads never share a counter, and no prefix collides with production
//! traffic (rl:auth, rl:auth_strict, rl:api, rl:msg, rl:upload, rl:auth_session).

use actix_web::body::BoxBody;
use actix_web::dev::ServiceResponse;
use actix_web::{App, HttpResponse, test, web};
use deadpool_redis::{Config as RedisConfig, Runtime};

use backend::middleware::rate_limit_middleware::{
    RATE_API, RATE_AUTH_STRICT, RateLimit, RateLimiter,
};

/// Bucket prefixes reserved for this file.
const ENFORCE_KEY_PREFIX: &str = "rl:it_enforce";
const HEADERS_KEY_PREFIX: &str = "rl:it_headers";
const TTL_KEY_PREFIX: &str = "rl:it_ttl";

/// Port with nothing listening, for the Redis unavailable scenarios.
const DEAD_REDIS_URL: &str = "redis://127.0.0.1:6399";

const LIMIT: u64 = 3;
const WINDOW_SECS: u64 = 60;

async fn handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// Pool for the scenarios that need a live Redis.
fn live_redis_pool() -> deadpool_redis::Pool {
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    RedisConfig::from_url(&url)
        .create_pool(Some(Runtime::Tokio1))
        .expect("redis pool for the rate limit tests")
}

/// Pool that can never connect, for the Redis unavailable scenarios. The
/// circuit breaker belongs to each limiter instance, so every scenario builds
/// its own app to get its own breaker.
fn dead_redis_pool() -> deadpool_redis::Pool {
    RedisConfig::from_url(DEAD_REDIS_URL)
        .create_pool(Some(Runtime::Tokio1))
        .expect("pool for the unreachable Redis")
}

/// Limiter for a route of the Default category, which keeps the prefix given
/// here instead of overwriting it with a category constant.
fn anonymous_limiter(prefix: &'static str) -> RateLimiter {
    RateLimiter::new(
        live_redis_pool(),
        RateLimit::new(LIMIT, WINDOW_SECS, prefix),
    )
}

/// Bucket key the middleware builds for an anonymous request with no peer
/// address, which is what the test requests look like.
fn bucket_key(prefix: &str) -> String {
    format!("{prefix}:unknown")
}

/// Empty a bucket so a scenario starts from a known counter.
async fn reset_bucket(pool: &deadpool_redis::Pool, prefix: &str) {
    let mut conn = pool.get().await.expect("redis connection");
    redis::cmd("DEL")
        .arg(bucket_key(prefix))
        .query_async::<i64>(&mut conn)
        .await
        .expect("bucket cleanup");
}

/// Remaining TTL of a bucket, in seconds.
async fn bucket_ttl(pool: &deadpool_redis::Pool, prefix: &str) -> i64 {
    let mut conn = pool.get().await.expect("redis connection");
    redis::cmd("TTL")
        .arg(bucket_key(prefix))
        .query_async::<i64>(&mut conn)
        .await
        .expect("bucket ttl")
}

fn header_value(resp: &ServiceResponse<BoxBody>, name: &str) -> Option<String> {
    resp.headers().get(name)?.to_str().ok().map(str::to_owned)
}

#[actix_web::test]
async fn anonymous_requests_are_throttled_past_the_limit() {
    let pool = live_redis_pool();
    reset_bucket(&pool, ENFORCE_KEY_PREFIX).await;

    let app = test::init_service(
        App::new()
            .wrap(anonymous_limiter(ENFORCE_KEY_PREFIX))
            .route("/api/v1/users", web::get().to(handler)),
    )
    .await;

    let mut statuses = Vec::new();
    for _ in 0..LIMIT + 2 {
        let resp = test::call_service(
            &app,
            test::TestRequest::get().uri("/api/v1/users").to_request(),
        )
        .await;
        statuses.push(resp.status().as_u16());
    }

    assert_eq!(
        statuses,
        vec![200, 200, 200, 429, 429],
        "exactly max_requests must pass and the rest must be throttled"
    );

    reset_bucket(&pool, ENFORCE_KEY_PREFIX).await;
}

#[actix_web::test]
async fn quota_headers_report_limit_remaining_and_reset() {
    let pool = live_redis_pool();
    reset_bucket(&pool, HEADERS_KEY_PREFIX).await;

    let app = test::init_service(
        App::new()
            .wrap(anonymous_limiter(HEADERS_KEY_PREFIX))
            .route("/api/v1/users", web::get().to(handler)),
    )
    .await;

    let allowed = test::call_service(
        &app,
        test::TestRequest::get().uri("/api/v1/users").to_request(),
    )
    .await;
    assert_eq!(allowed.status(), 200);
    assert_eq!(
        header_value(&allowed, "x-ratelimit-limit").as_deref(),
        Some("3")
    );
    assert_eq!(
        header_value(&allowed, "x-ratelimit-remaining").as_deref(),
        Some("2"),
        "the allowed response must report the budget left"
    );
    let reset_on_allowed = header_value(&allowed, "x-ratelimit-reset")
        .and_then(|value| value.parse::<u64>().ok())
        .expect("the allowed response must report the seconds left in the window");
    assert!(
        reset_on_allowed > 0 && reset_on_allowed <= WINDOW_SECS,
        "reset must be the seconds left in the window, got {reset_on_allowed}"
    );

    // Drain the remaining budget to reach the throttled response.
    for _ in 0..LIMIT - 1 {
        let resp = test::call_service(
            &app,
            test::TestRequest::get().uri("/api/v1/users").to_request(),
        )
        .await;
        assert_eq!(resp.status(), 200);
    }

    let throttled = test::call_service(
        &app,
        test::TestRequest::get().uri("/api/v1/users").to_request(),
    )
    .await;
    assert_eq!(throttled.status(), 429);
    assert_eq!(
        header_value(&throttled, "x-ratelimit-limit").as_deref(),
        Some("3")
    );
    assert_eq!(
        header_value(&throttled, "x-ratelimit-remaining").as_deref(),
        Some("0"),
        "a full bucket has no budget left"
    );
    assert_eq!(
        header_value(&throttled, "retry-after").as_deref(),
        Some("60"),
        "the client must be told when to retry"
    );

    reset_bucket(&pool, HEADERS_KEY_PREFIX).await;
}

#[actix_web::test]
async fn bucket_key_carries_a_ttl_so_a_window_cannot_leak() {
    let pool = live_redis_pool();
    reset_bucket(&pool, TTL_KEY_PREFIX).await;

    let app = test::init_service(
        App::new()
            .wrap(anonymous_limiter(TTL_KEY_PREFIX))
            .route("/api/v1/users", web::get().to(handler)),
    )
    .await;

    // Two requests: the second must not restart the TTL, since only the first
    // request of a window sets it.
    for _ in 0..2 {
        let resp = test::call_service(
            &app,
            test::TestRequest::get().uri("/api/v1/users").to_request(),
        )
        .await;
        assert_eq!(resp.status(), 200);
    }

    let ttl = bucket_ttl(&pool, TTL_KEY_PREFIX).await;
    assert!(
        ttl > 0 && ttl <= WINDOW_SECS as i64,
        "the bucket must expire with the window, got a ttl of {ttl}"
    );

    reset_bucket(&pool, TTL_KEY_PREFIX).await;
}

#[actix_web::test]
async fn brute_force_endpoints_fail_closed_when_redis_is_unavailable() {
    let app = test::init_service(
        App::new()
            .wrap(RateLimiter::new(dead_redis_pool(), RATE_AUTH_STRICT))
            .route("/api/v1/auth/login", web::post().to(handler)),
    )
    .await;

    let mut statuses = Vec::new();
    for _ in 0..8 {
        let resp = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/auth/login")
                .to_request(),
        )
        .await;
        statuses.push(resp.status().as_u16());
    }

    // Enough requests to open the circuit breaker, which a limiter of any
    // other category would ride out. Login must not become unlimited.
    assert!(
        statuses.iter().all(|code| *code == 503),
        "AuthStrict must fail closed on every request while Redis is down, got {statuses:?}"
    );
}

#[actix_web::test]
async fn general_endpoints_degrade_to_fail_open_when_redis_is_unavailable() {
    let app = test::init_service(
        App::new()
            .wrap(RateLimiter::new(dead_redis_pool(), RATE_API))
            .route("/api/v1/users", web::get().to(handler)),
    )
    .await;

    let mut statuses = Vec::new();
    for _ in 0..8 {
        let resp = test::call_service(
            &app,
            test::TestRequest::get().uri("/api/v1/users").to_request(),
        )
        .await;
        statuses.push(resp.status().as_u16());
    }

    // The first requests fail closed until the circuit opens, then the
    // endpoint serves traffic again instead of turning an outage into a full
    // denial of service.
    assert!(
        statuses.contains(&200),
        "once the circuit opens, general endpoints must serve traffic again, got {statuses:?}"
    );
}
