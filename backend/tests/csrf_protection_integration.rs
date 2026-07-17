//! Integration tests for CSRF protection middleware.
//!
//! Verifies:
//! - Bearer token skips CSRF check (SPA token auth)
//! - POST without Bearer and without CSRF token → 403
//! - POST with valid CSRF token → passes
//! - GET sets CSRF cookie
//! - Excluded paths (login, register, webhooks) skip CSRF
//! - Cookie auth paths enforce CSRF even with Bearer token

use actix_web::{App, HttpRequest, http::StatusCode, test, web};

use backend::config::AppConfig;
use backend::middleware::csrf_protection::CsrfProtection;

fn make_config() -> AppConfig {
    // SAFETY: these tests run single-threaded; env vars are only read during
    // AppConfig::from_env() construction and are not shared across test threads.
    unsafe {
        std::env::set_var(
            "CSRF_SECRET_KEY",
            "test-csrf-secret-key-for-integration-tests-32b!",
        );
        std::env::set_var(
            "JWT_SECRET",
            "test-jwt-secret-key-for-integration-tests-32b!",
        );
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        std::env::set_var("MASTER_KEY", "test-master-key-for-integration-tests-32b!");
        std::env::set_var(
            "BLIND_INDEX_KEY",
            "test-blind-index-key-for-integration-tests-32b!",
        );
        std::env::set_var(
            "REFRESH_TOKEN_HASH_SALT",
            "test-refresh-token-hash-salt-for-integration-32b!",
        );
    }
    AppConfig::from_env().expect("failed to build test config")
}

async fn state_changing_handler(_req: HttpRequest) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().body("ok")
}

async fn get_handler(_req: HttpRequest) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().body("ok")
}

#[actix_web::test]
async fn post_with_bearer_skips_csrf() {
    let config = make_config();
    let app = test::init_service(
        App::new().app_data(web::Data::new(config)).service(
            web::scope("/api/v1")
                .wrap(CsrfProtection::new(vec![]))
                .route("/admin/data", web::post().to(state_changing_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/data")
        .insert_header(("authorization", "Bearer test-token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn post_without_bearer_and_without_csrf_returns_403() {
    let config = make_config();
    let app = test::init_service(
        App::new().app_data(web::Data::new(config)).service(
            web::scope("/api/v1")
                .wrap(CsrfProtection::new(vec![]))
                .route("/admin/data", web::post().to(state_changing_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/data")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"]["code"], "CSRF_TOKEN_INVALID");
}

#[actix_web::test]
async fn excluded_path_skips_csrf() {
    let config = make_config();
    let app = test::init_service(
        App::new().app_data(web::Data::new(config)).service(
            web::scope("/api/v1")
                .wrap(CsrfProtection::new(vec![]))
                .route("/auth/login", web::post().to(state_changing_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn get_request_sets_csrf_cookie() {
    let config = make_config();
    let app = test::init_service(
        App::new().app_data(config.clone()).service(
            web::scope("/api/v1")
                .wrap(CsrfProtection::new(vec![]))
                .route("/admin/data", web::get().to(get_handler)),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/data")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key("set-cookie"));

    let cookie = resp.headers().get("set-cookie").unwrap().to_str().unwrap();
    assert!(cookie.contains("csrf_token"));
}

#[actix_web::test]
async fn state_changing_request_rotates_csrf_token() {
    let config = make_config();
    let app = test::init_service(
        App::new().app_data(config.clone()).service(
            web::scope("/api/v1")
                .wrap(CsrfProtection::new(vec![]))
                .route("/admin/data", web::post().to(state_changing_handler)),
        ),
    )
    .await;

    // POST with valid CSRF token → CSRF check runs → cookie is rotated
    let token = backend::middleware::csrf_protection::generate_csrf_token(&config.csrf_secret_key);

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/data")
        .insert_header(("csrf-token", token.clone()))
        .cookie(
            actix_web::cookie::Cookie::build("csrf_token", token)
                .path("/")
                .finish(),
        )
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key("set-cookie"));
}

#[actix_web::test]
async fn cookie_auth_path_enforces_csrf_with_bearer() {
    let config = make_config();
    let csrf =
        CsrfProtection::new(vec![]).with_cookie_auth_paths(vec!["/api/v1/admin".to_string()]);

    let app = test::init_service(
        App::new().app_data(web::Data::new(config)).service(
            web::scope("/api/v1")
                .wrap(csrf)
                .route("/admin/data", web::post().to(state_changing_handler)),
        ),
    )
    .await;

    // Even with Bearer token, cookie auth path enforces CSRF
    let req = test::TestRequest::post()
        .uri("/api/v1/admin/data")
        .insert_header(("authorization", "Bearer test-token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn non_cookie_auth_path_still_skips_csrf_with_bearer() {
    let config = make_config();
    // cookie_auth_paths only includes /api/v1/admin, not /api/v1/users
    let csrf =
        CsrfProtection::new(vec![]).with_cookie_auth_paths(vec!["/api/v1/admin".to_string()]);

    let app = test::init_service(
        App::new().app_data(web::Data::new(config)).service(
            web::scope("/api/v1")
                .wrap(csrf)
                .route("/users/data", web::post().to(state_changing_handler)),
        ),
    )
    .await;

    // /api/v1/users is NOT in cookie_auth_paths → Bearer skips CSRF
    let req = test::TestRequest::post()
        .uri("/api/v1/users/data")
        .insert_header(("authorization", "Bearer test-token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
