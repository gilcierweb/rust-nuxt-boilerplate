//! Request ID middleware for correlation ID propagation.
//!
//! This middleware generates or extracts a request ID and stores it in
//! request extensions for downstream use. The request ID is also added
//! to response headers for client correlation.
//!
//! # Usage
//!
//! The middleware extracts `X-Request-ID` from incoming requests. If missing,
//! it generates a new UUID v4. The ID is stored in request extensions and
//! can be accessed via `RequestId::from_req(req)`.
//!
//! # Headers
//!
//! - **Incoming**: `X-Request-ID` (optional) — client-provided correlation ID
//! - **Outgoing**: `X-Request-ID` (always) — correlation ID for this request

use std::{rc::Rc, fmt};

use actix_web::{
    Error,
    HttpMessage,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header::{HeaderName, HeaderValue},
};
use futures::future::{LocalBoxFuture, Ready, ready};
use uuid::Uuid;

/// Extension key for storing request ID in request extensions.
#[derive(Clone)]
pub struct RequestId(pub String);

impl RequestId {
    /// Get the request ID from request extensions.
    pub fn from_req(req: &ServiceRequest) -> Option<String> {
        req.extensions().get::<RequestId>().map(|id| id.0.clone())
    }

    /// Get the request ID or generate a new one.
    pub fn from_req_or_new(req: &ServiceRequest) -> String {
        Self::from_req(req).unwrap_or_else(|| Uuid::new_v4().to_string())
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
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

        // Extract or generate request ID
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.trim().to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Store in request extensions
        req.extensions_mut().insert(RequestId(request_id.clone()));

        Box::pin(async move {
            let mut response = service.call(req).await?;

            // Add request ID to response headers
            if let Ok(header_value) = HeaderValue::from_str(&request_id) {
                response
                    .headers_mut()
                    .insert(HeaderName::from_static("x-request-id"), header_value);
            }

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{App, HttpResponse, http::StatusCode, test, web};

    use super::{RequestId, RequestIdMiddleware};

    #[actix_web::test]
    async fn preserves_existing_request_id() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("x-request-id", "my-correlation-id"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get("x-request-id").and_then(|v| v.to_str().ok()),
            Some("my-correlation-id")
        );
    }

    #[actix_web::test]
    async fn generates_request_id_when_missing() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        let request_id = resp
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        assert!(uuid::Uuid::parse_str(request_id).is_ok());
    }

    #[actix_web::test]
    async fn request_id_available_in_handler() {
        use actix_web::{HttpMessage, HttpRequest};

        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route(
                    "/test",
                    web::get().to(|req: HttpRequest| {
                        let id = req.extensions().get::<RequestId>().cloned();
                        async move { HttpResponse::Ok().body(id.map(|r| r.0).unwrap_or_default()) }
                    }),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("x-request-id", "ext-test-id"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        assert_eq!(body, "ext-test-id");
    }
}
