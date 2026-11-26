use std::rc::Rc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::{Error, ResponseError, web};
use futures::future::{LocalBoxFuture, Ready, ready};

use crate::errors::AppError;
use crate::repositories::container::AppContainer;
use crate::utils::constant_time::constant_time_eq;

#[derive(Clone, Default)]
pub struct RequireApiKey {
    exempt_paths: Vec<String>,
}

impl RequireApiKey {
    pub fn new(exempt_paths: Vec<&str>) -> Self {
        Self {
            exempt_paths: exempt_paths.into_iter().map(str::to_owned).collect(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireApiKey
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Transform = RequireApiKeyMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireApiKeyMiddleware {
            service: Rc::new(service),
            exempt_paths: self.exempt_paths.clone(),
        }))
    }
}

pub struct RequireApiKeyMiddleware<S> {
    service: Rc<S>,
    exempt_paths: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for RequireApiKeyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let exempt_paths = self.exempt_paths.clone();

        Box::pin(async move {
            let path = req.path().to_string();
            let is_exempt = exempt_paths.iter().any(|pattern| {
                if pattern.ends_with('*') {
                    let prefix = &pattern[..pattern.len() - 1];
                    path.starts_with(prefix)
                } else {
                    path == pattern.as_str()
                }
            });

            if is_exempt {
                return service
                    .call(req)
                    .await
                    .map(ServiceResponse::map_into_left_body);
            }

            let expected_keys = req
                .app_data::<web::Data<AppContainer>>()
                .map(|container| container.config.backend_api_keys.clone())
                .unwrap_or_default();

            let provided_key = extract_api_key(&req);

            // In development (no keys configured) we fail open so the
            // boilerplate works out of the box; production requires keys
            // (enforced by validate_or_panic).
            if expected_keys.is_empty() {
                // No keys configured: fail loudly with an actionable message
                // instead of a cryptic 401, so the operator sees exactly what
                // is missing. In production validate_or_panic already blocks
                // startup on this, so this path only happens in dev.
                tracing::warn!(
                    event = "api_key.no_keys_configured",
                    "BACKEND_API_KEYS is empty. Set it in .env (or run \
                     scripts/generate-secrets.sh) to enable /api/v1/admin/*."
                );
                let response = AppError::Internal(
                    "BACKEND_API_KEYS is not configured. Set it in .env or run \
                     scripts/generate-secrets.sh."
                        .to_string(),
                )
                .error_response()
                .map_into_right_body();
                return Ok(req.into_response(response));
            }

            let is_valid = provided_key
                .as_deref()
                .map(|provided| {
                    expected_keys
                        .iter()
                        .any(|expected| constant_time_eq(expected, provided))
                })
                .unwrap_or(false);

            if !is_valid {
                let response = AppError::Unauthorized(t!("middleware.unauthorized").into_owned())
                    .error_response()
                    .map_into_right_body();
                return Ok(req.into_response(response));
            }

            service
                .call(req)
                .await
                .map(ServiceResponse::map_into_left_body)
        })
    }
}

fn extract_api_key(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .or_else(|| {
            req.headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("ApiKey "))
                .map(str::to_owned)
        })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, HttpResponse, test, web};

    use super::RequireApiKey;
    use crate::repositories::mocks::mock_container;

    fn container_with_api_key() -> crate::repositories::container::AppContainer {
        let mut container = mock_container();
        let mut config = (*container.config).clone();
        config.backend_api_keys = vec!["test-api-key".to_string()];
        container.config = Arc::new(config);
        container
    }

    #[actix_web::test]
    async fn rejects_missing_api_key() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container_with_api_key()))
                .service(
                    web::scope("/internal-test")
                        .wrap(RequireApiKey::default())
                        .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
                ),
        )
        .await;

        let req = test::TestRequest::get().uri("/internal-test").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn rejects_invalid_api_key() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container_with_api_key()))
                .service(
                    web::scope("/internal-test")
                        .wrap(RequireApiKey::default())
                        .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/internal-test")
            .insert_header(("X-API-Key", "wrong-key"))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn accepts_valid_api_key() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container_with_api_key()))
                .service(
                    web::scope("/internal-test")
                        .wrap(RequireApiKey::default())
                        .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/internal-test")
            .insert_header(("X-API-Key", "test-api-key"))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn allows_exempt_path_without_api_key() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container_with_api_key()))
                .service(
                    web::scope("")
                        .wrap(RequireApiKey::new(vec!["/webhooks/*"]))
                        .service(
                            web::resource("/webhooks/stripe")
                                .to(|| async { HttpResponse::Ok().finish() }),
                        ),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/webhooks/stripe")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
