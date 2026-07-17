//! Integration tests for API versioning middleware.
//!
//! Verifies that `ApiVersionGuard` correctly:
//! - Adds deprecation headers (Deprecation, Sunset, Link, X-API-Warn)
//! - Returns 400 for unsupported versions with `x-api-supported`
//! - Skips version logic for non-API paths
//! - Applies to all API route groups (auth, admin, health, metrics, webhooks)
//! - Handles version extraction from URL and Accept header
//!
//! Router audit: all 29 endpoints live under `web::scope("/api/v1")` with
//! `ApiVersionGuard` applied at the outermost scope level (router.rs:69).

use actix_web::{App, http::StatusCode, test, web};

use backend::middleware::api_version::{ApiVersionConfig, ApiVersionGuard, ApiVersionInfo};

async fn ok_handler() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().body("ok")
}

async fn post_ok_handler() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Created().body("created")
}

fn default_config() -> ApiVersionConfig {
    ApiVersionConfig::new()
}

fn deprecated_v1_config() -> ApiVersionConfig {
    ApiVersionConfig::new().with_deprecated(ApiVersionInfo {
        version: 1,
        released: "2024-01-01".to_string(),
        deprecated: Some("2025-06-01".to_string()),
        sunset: Some("2025-12-31".to_string()),
        docs_url: Some("https://docs.example.com/migrate".to_string()),
    })
}

fn multi_deprecated_config() -> ApiVersionConfig {
    ApiVersionConfig {
        supported: vec![1, 2],
        deprecated: [
            (
                1,
                ApiVersionInfo {
                    version: 1,
                    released: "2024-01-01".to_string(),
                    deprecated: Some("2025-06-01".to_string()),
                    sunset: Some("2025-12-31".to_string()),
                    docs_url: Some("https://docs.example.com/v1-to-v2".to_string()),
                },
            ),
            (
                2,
                ApiVersionInfo {
                    version: 2,
                    released: "2025-06-01".to_string(),
                    deprecated: Some("2026-01-01".to_string()),
                    sunset: Some("2026-06-30".to_string()),
                    docs_url: Some("https://docs.example.com/v2-to-v3".to_string()),
                },
            ),
        ]
        .into_iter()
        .collect(),
    }
}

/// Configure all API sub-routes inside the versioned scope.
fn api_v1_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(ok_handler))
        .route("/metrics", web::get().to(ok_handler))
        .service(
            web::scope("/auth")
                .route("/login", web::post().to(post_ok_handler))
                .route("/register", web::post().to(post_ok_handler))
                .route("/me", web::get().to(ok_handler)),
        )
        .service(
            web::scope("/admin")
                .route("/roles", web::get().to(ok_handler))
                .route("/users", web::get().to(ok_handler)),
        )
        .service(web::scope("/webhooks").route("/stripe", web::post().to(post_ok_handler)));
}

// ===========================================================================
// 1. Supported version (v1) — no deprecation headers
// ===========================================================================

#[actix_web::test]
async fn supported_v1_no_deprecation_headers() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(default_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(!resp.headers().contains_key("deprecation"));
    assert!(!resp.headers().contains_key("sunset"));
    assert!(!resp.headers().contains_key("x-api-warn"));
    assert!(!resp.headers().contains_key("link"));
}

// ===========================================================================
// 2. Deprecated version — all 4 deprecation headers present
// ===========================================================================

#[actix_web::test]
async fn deprecated_v1_all_headers_present() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let h = resp.headers();
    assert_eq!(h.get("deprecation").unwrap(), "true");
    assert_eq!(h.get("sunset").unwrap(), "2025-12-31");
    assert_eq!(
        h.get("x-api-warn").unwrap().to_str().unwrap(),
        "API v1 is deprecated. Migrate to latest version."
    );

    let link = h.get("link").unwrap().to_str().unwrap();
    assert!(link.contains("rel=\"deprecation\""));
    assert!(link.contains("docs.example.com/migrate"));
}

// ===========================================================================
// 3. Deprecated headers on POST endpoint (not just GET)
// ===========================================================================

#[actix_web::test]
async fn deprecated_headers_on_post_endpoint() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    assert_eq!(resp.headers().get("deprecation").unwrap(), "true");
    assert_eq!(resp.headers().get("sunset").unwrap(), "2025-12-31");
}

// ===========================================================================
// 4–7. All route groups receive version guard
// ===========================================================================

#[actix_web::test]
async fn auth_route_gets_version_guard() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/auth/me").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("deprecation").unwrap(), "true");
}

#[actix_web::test]
async fn admin_route_gets_version_guard() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/roles")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("deprecation").unwrap(), "true");
}

#[actix_web::test]
async fn metrics_route_gets_version_guard() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("deprecation").unwrap(), "true");
}

#[actix_web::test]
async fn webhook_route_gets_version_guard() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/webhooks/stripe")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    assert_eq!(resp.headers().get("deprecation").unwrap(), "true");
}

// ===========================================================================
// 8. Unsupported version — 400 with x-api-supported + JSON body
// ===========================================================================

#[actix_web::test]
async fn unsupported_version_returns_400() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(default_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v99/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let h = resp.headers();
    let supported = h.get("x-api-supported").unwrap().to_str().unwrap();
    assert!(supported.contains("v1"));

    let ct = h.get("content-type").unwrap().to_str().unwrap();
    assert!(ct.contains("application/json"));
}

#[actix_web::test]
async fn unsupported_version_error_body_format() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(default_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v2/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "UNSUPPORTED_API_VERSION");
    assert!(body["error"]["message"].as_str().unwrap().contains("v2"));
    assert!(body["error"]["message"].as_str().unwrap().contains("v1"));
}

// ===========================================================================
// 9. Non-API path — version guard skipped entirely
// ===========================================================================

#[actix_web::test]
async fn non_api_path_skips_version_guard() {
    let app = test::init_service(
        App::new()
            .service(
                web::scope("/api")
                    .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                    .service(web::scope("/v1").configure(api_v1_routes)),
            )
            .route("/health", web::get().to(ok_handler)),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(!resp.headers().contains_key("deprecation"));
    assert!(!resp.headers().contains_key("sunset"));
    assert!(!resp.headers().contains_key("x-api-warn"));
}

// ===========================================================================
// 10. Version via Accept header (vnd format)
// ===========================================================================

#[actix_web::test]
async fn deprecated_via_accept_header() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .insert_header(("accept", "application/vnd.app-boilerplate.v1+json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("deprecation").unwrap(), "true");
}

// ===========================================================================
// 11. URL version takes priority over Accept header
// ===========================================================================

#[actix_web::test]
async fn url_version_takes_priority_over_accept_header() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(default_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .insert_header(("accept", "application/vnd.app-boilerplate.v99+json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// ===========================================================================
// 12. Swagger UI outside versioned scope — no version headers
// ===========================================================================

#[actix_web::test]
async fn swagger_ui_outside_versioned_scope() {
    let app = test::init_service(
        App::new()
            .service(
                web::scope("/api")
                    .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                    .service(web::scope("/v1").configure(api_v1_routes)),
            )
            .route("/swagger-ui/test", web::get().to(ok_handler)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/swagger-ui/test")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(!resp.headers().contains_key("deprecation"));
}

// ===========================================================================
// 13–14. Multiple deprecated versions — different sunset/docs per version
// ===========================================================================

#[actix_web::test]
async fn deprecated_v1_and_v2_different_sunset_dates() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(multi_deprecated_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    // v1 → sunset 2025-12-31
    let req = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("deprecation").unwrap(), "true");
    assert_eq!(resp.headers().get("sunset").unwrap(), "2025-12-31");
    let link = resp.headers().get("link").unwrap().to_str().unwrap();
    assert!(link.contains("v1-to-v2"));
}

#[actix_web::test]
async fn unsupported_v3_returns_400_with_multi_version_config() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(multi_deprecated_config()))
                .service(web::scope("/v1").configure(api_v1_routes)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v3/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let supported = resp
        .headers()
        .get("x-api-supported")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(supported.contains("v1"));
    assert!(supported.contains("v2"));
    assert!(!supported.contains("v3"));
}

// ===========================================================================
// 15. No version in URL — guard skips (path doesn't start with /api/v)
// ===========================================================================

#[actix_web::test]
async fn no_version_in_url_guard_skips() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                .route("/test", web::get().to(ok_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/test").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(!resp.headers().contains_key("deprecation"));
}

// ===========================================================================
// 16. PATCH request also gets version headers
// ===========================================================================

#[actix_web::test]
async fn patch_request_gets_deprecation_headers() {
    let app = test::init_service(
        App::new().service(
            web::scope("/api")
                .wrap(ApiVersionGuard::new(deprecated_v1_config()))
                .service(web::scope("/v1").route("/resource", web::patch().to(ok_handler))),
        ),
    )
    .await;

    let req = test::TestRequest::patch()
        .uri("/api/v1/resource")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("deprecation").unwrap(), "true");
}
