//! Security headers middleware for API responses.
//!
//! Adds standard security headers to all responses:
//! - `X-Content-Type-Options: nosniff` — Prevents MIME type sniffing
//! - `X-Frame-Options: DENY` — Prevents clickjacking
//! - `Referrer-Policy: strict-origin-when-cross-origin` — Controls referrer info
//! - `Permissions-Policy` — Restricts browser features
//! - `X-XSS-Protection: 0` — Disables legacy XSS filter (use CSP instead)
//! - `Content-Security-Policy` — Restricts resource loading (API-tuned)
//!
//! Note: The frontend uses `nuxt-security` for CSP. This middleware handles
//! the API layer only.

use std::rc::Rc;

use actix_web::{
    Error,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header::{self, HeaderValue},
};
use futures::future::{LocalBoxFuture, Ready, ready};

pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersService {
            service: Rc::new(service),
        }))
    }
}

pub struct SecurityHeadersService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let mut response = service.call(req).await?;

            let headers = response.headers_mut();

            // Prevent MIME type sniffing
            headers.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );

            // Prevent clickjacking
            headers.insert(
                header::X_FRAME_OPTIONS,
                HeaderValue::from_static("DENY"),
            );

            // Control referrer information
            headers.insert(
                header::REFERRER_POLICY,
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Disable legacy XSS filter (CSP is the modern approach)
            headers.insert(
                header::HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("0"),
            );

            // Restrict browser features
            headers.insert(
                header::HeaderName::from_static("permissions-policy"),
                HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=()"),
            );

            // Content Security Policy for API
            // More restrictive than frontend — API only serves JSON
            headers.insert(
                header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_static(
                    "default-src 'none'; frame-ancestors 'none'; form-action 'none'"
                ),
            );

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test]
    async fn security_headers_are_added() {
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeaders)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        let headers = resp.headers();
        assert_eq!(
            headers.get("X-Content-Type-Options").unwrap(),
            "nosniff"
        );
        assert_eq!(headers.get("X-Frame-Options").unwrap(), "DENY");
        assert_eq!(
            headers.get("Referrer-Policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(headers.get("X-XSS-Protection").unwrap(), "0");
        assert!(headers.get("Permissions-Policy").is_some());
        assert!(headers.get("Content-Security-Policy").is_some());
    }
}
