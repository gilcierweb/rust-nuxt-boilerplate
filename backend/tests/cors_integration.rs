//! Integration tests for the CORS middleware.
//!
//! Verifies:
//! - A preflight from a configured origin is answered, echoing the headers the
//!   SPA and the Nitro proxy need (authorization, csrf-token, x-request-id)
//! - An origin that is not on the allow-list receives no allow-origin header,
//!   so credentialed access stays opt-in
//! - No configured origins means no cross-origin access at all

use actix_web::http::{Method, StatusCode};
use actix_web::{App, HttpResponse, test, web};
use backend::middleware::cors::build_cors;

async fn handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// CORS middleware for the given allow-list.
fn cors_middleware(origins: &[&str]) -> actix_cors::Cors {
    build_cors(
        &origins
            .iter()
            .map(|origin| (*origin).to_string())
            .collect::<Vec<_>>(),
    )
}

#[actix_web::test]
async fn preflight_echoes_the_headers_the_spa_sends() {
    // The SPA submits the CSRF double-submit header on refresh/logout. A
    // cross-origin deployment whose preflight does not echo it cannot
    // authenticate at all.
    let app = test::init_service(
        App::new()
            .wrap(cors_middleware(&["https://app.example.com"]))
            .route("/api/v1/auth/refresh", web::route().to(handler)),
    )
    .await;

    let req = test::TestRequest::with_uri("/api/v1/auth/refresh")
        .method(Method::OPTIONS)
        .insert_header(("origin", "https://app.example.com"))
        .insert_header(("access-control-request-method", "POST"))
        .insert_header((
            "access-control-request-headers",
            "authorization,content-type,csrf-token,x-request-id,x-api-key",
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let allowed = resp
        .headers()
        .get("access-control-allow-headers")
        .expect("CORS must answer the preflight")
        .to_str()
        .unwrap();
    assert!(allowed.contains("csrf-token"), "got: {allowed}");
    assert!(allowed.contains("x-request-id"), "got: {allowed}");
    assert!(allowed.contains("x-api-key"), "got: {allowed}");
    assert!(
        allowed.contains("authorization"),
        "existing headers must keep working, got: {allowed}"
    );
}

#[actix_web::test]
async fn preflight_from_an_unlisted_origin_is_not_allowed() {
    // Credentialed CORS is opt-in per origin: a wildcard would let any site
    // ride the user's cookies.
    let app = test::init_service(
        App::new()
            .wrap(cors_middleware(&["https://app.example.com"]))
            .route("/api/v1/auth/refresh", web::route().to(handler)),
    )
    .await;

    let req = test::TestRequest::with_uri("/api/v1/auth/refresh")
        .method(Method::OPTIONS)
        .insert_header(("origin", "https://evil.example.net"))
        .insert_header(("access-control-request-method", "POST"))
        .insert_header(("access-control-request-headers", "content-type"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.headers().get("access-control-allow-origin").is_none(),
        "an unlisted origin must not receive the allow-origin header"
    );
}

#[actix_web::test]
async fn no_configured_origins_grants_no_cross_origin_access() {
    let app = test::init_service(
        App::new()
            .wrap(cors_middleware(&[]))
            .route("/api/v1/auth/refresh", web::route().to(handler)),
    )
    .await;

    let req = test::TestRequest::with_uri("/api/v1/auth/refresh")
        .method(Method::OPTIONS)
        .insert_header(("origin", "https://app.example.com"))
        .insert_header(("access-control-request-method", "POST"))
        .insert_header(("access-control-request-headers", "content-type"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.headers().get("access-control-allow-origin").is_none(),
        "an empty origin list must deny cross-origin requests"
    );
}
