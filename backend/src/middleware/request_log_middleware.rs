use std::{rc::Rc, time::Instant};

use actix_web::{
    Error,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header::{HeaderName, HeaderValue},
};
use futures::future::{LocalBoxFuture, Ready, ready};

use super::request_id::RequestId;

pub struct RequestLogMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestLogMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestLogMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLogMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestLogMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestLogMiddlewareService<S>
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
        let method = req.method().to_string();
        let path = req.path().to_string();
        let user_agent = req
            .headers()
            .get(actix_web::http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        // Get request ID from extensions (set by RequestIdMiddleware) or generate one
        let request_id = RequestId::from_req_or_new(&req);
        let started_at = Instant::now();

        Box::pin(async move {
            let result = service.call(req).await;
            let duration_ms = started_at.elapsed().as_secs_f64() * 1000.0;

            match result {
                Ok(mut response) => {
                    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
                        response
                            .headers_mut()
                            .insert(HeaderName::from_static("x-request-id"), header_value);
                    }

                    tracing::info!(
                        target: "http.request",
                        request_id = %request_id,
                        method = %method,
                        path = %path,
                        user_agent = %user_agent,
                        status = response.status().as_u16(),
                        duration_ms = duration_ms,
                        "request_completed"
                    );

                    Ok(response)
                }
                Err(error) => {
                    tracing::error!(
                        target: "http.request",
                        request_id = %request_id,
                        method = %method,
                        path = %path,
                        user_agent = %user_agent,
                        duration_ms = duration_ms,
                        error = %error,
                        "request_failed"
                    );
                    Err(error)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{App, HttpResponse, http::StatusCode, test, web};

    use super::RequestLogMiddleware;
    use crate::middleware::request_id::RequestIdMiddleware;

    #[actix_web::test]
    async fn preserves_existing_request_id_header() {
        let app = test::init_service(
            App::new()
                .wrap(RequestLogMiddleware)
                .wrap(RequestIdMiddleware)
                .route(
                    "/ok",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                ),
        )
        .await;

        let request = test::TestRequest::get()
            .uri("/ok")
            .insert_header(("x-request-id", "req-123"))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("x-request-id")
                .and_then(|value| value.to_str().ok()),
            Some("req-123")
        );
    }

    #[actix_web::test]
    async fn generates_request_id_when_missing() {
        let app = test::init_service(
            App::new()
                .wrap(RequestLogMiddleware)
                .wrap(RequestIdMiddleware)
                .route(
                    "/ok",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                ),
        )
        .await;

        let request = test::TestRequest::get().uri("/ok").to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert!(uuid::Uuid::parse_str(request_id).is_ok());
    }
}
