use std::rc::Rc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::{Error, ResponseError, web};
use futures::future::{LocalBoxFuture, Ready, ready};

use crate::errors::AppError;
use crate::repositories::container::AppContainer;

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
                .map(|container| container.config.internal_api_keys.clone())
                .unwrap_or_default();

            let provided_key = extract_api_key(&req);

            let is_valid = provided_key
                .as_deref()
                .map(|provided| {
                    !expected_keys.is_empty()
                        && expected_keys
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

/// Compare two API key strings in constant time.
///
/// Both the length check and the byte comparison are performed without
/// short-circuiting, so an attacker cannot infer the correct key length
/// or any individual byte by measuring response latency.
///
/// # Implementation
///
/// We use `subtle::ConstantTimeEq` from the `subtle` crate (already a
/// project dependency) for the byte-level comparison. For length equality
/// we also use `subtle::Choice` so the branch is not visible to the
/// optimiser or CPU branch-predictor.
///
/// `subtle` is already used in `token_service.rs` for refresh-token hash
/// verification, so no new dependency is introduced.
fn constant_time_eq(expected: &str, provided: &str) -> bool {
    use subtle::{Choice, ConstantTimeEq};

    let expected = expected.as_bytes();
    let provided = provided.as_bytes();

    // Lengths must match. Encode the result as a `subtle::Choice` (1 = equal)
    // so that the branch below is not evaluated until after the byte comparison.
    let lengths_match: Choice =
        subtle::ConstantTimeEq::ct_eq(&(expected.len() as u64), &(provided.len() as u64));

    // Byte comparison. We pad the shorter slice with zeros so that the loop
    // always runs for `max(len_a, len_b)` iterations regardless of input
    // lengths, preventing the optimiser from eliding work based on the length
    // difference. The `lengths_match` flag ensures the overall result is
    // `false` whenever lengths differ, even if the padded bytes accidentally
    // compare equal.
    let max_len = expected.len().max(provided.len());
    let mut diff = subtle::Choice::from(0u8); // accumulates any difference

    for i in 0..max_len {
        let a = if i < expected.len() { expected[i] } else { 0 };
        let b = if i < provided.len() { provided[i] } else { 0 };
        // ct_ne returns Choice(1) if bytes differ
        diff = diff | a.ct_ne(&b);
    }

    // Valid only when lengths match AND no bytes differ.
    // Both conditions are evaluated; neither causes an early exit.
    bool::from(lengths_match & !diff)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, HttpResponse, test, web};

    use super::RequireApiKey;
    use crate::repositories::test_utils::mocks::mock_container;

    fn container_with_api_key() -> crate::repositories::container::AppContainer {
        let mut container = mock_container();
        let mut config = (*container.config).clone();
        config.internal_api_keys = vec!["test-api-key".to_string()];
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

    // constant_time_eq unit tests are in the submodule below to avoid
    // conflict with the #[actix_web::test] context used by the async tests.
}

#[cfg(test)]
mod constant_time_tests {
    #[test]
    fn identical_keys() {
        assert!(super::constant_time_eq("secret-key-abc", "secret-key-abc"));
    }

    #[test]
    fn different_content_same_length() {
        assert!(!super::constant_time_eq("secret-key-abc", "secret-key-xyz"));
    }

    #[test]
    fn different_lengths() {
        // Must return false without leaking which side is longer.
        assert!(!super::constant_time_eq("short", "short-but-longer"));
        assert!(!super::constant_time_eq("short-but-longer", "short"));
    }

    #[test]
    fn empty_strings() {
        assert!(super::constant_time_eq("", ""));
    }

    #[test]
    fn one_empty() {
        assert!(!super::constant_time_eq("", "non-empty"));
        assert!(!super::constant_time_eq("non-empty", ""));
    }

    #[test]
    fn prefix_match_is_rejected() {
        // "abc" must not match "abcX" even though one is a prefix of the other.
        assert!(!super::constant_time_eq("abc", "abcX"));
        assert!(!super::constant_time_eq("abcX", "abc"));
    }
}
