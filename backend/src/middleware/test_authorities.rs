//! Test-only middleware that attaches `actix_web_grants::authorities::AuthDetails`
//! from a test header `x-test-authorities` (comma-separated).
//!
//! This replaces the `GrantsMiddleware::with_extractor` approach used in tests
//! now that production code attaches `AuthDetails` directly from the JWT claims
//! inside `JwtAuth` middleware.
//!
//! This middleware must be wrapped on a scope **inside** `JwtAuth` because
//! `AuthDetails` is read via the `FromRequest` impl of `AuthDetails`, which
//! borrows extensions immutably. It must apply AFTER `JwtAuth` has inserted
//! claims into extensions.

use std::collections::HashSet;
use std::rc::Rc;

use actix_web::{
    Error,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use actix_web_grants::authorities::AttachAuthorities;
use futures::future::{LocalBoxFuture, Ready, ready};

pub struct TestAuthorities;

impl<S, B> Transform<S, ServiceRequest> for TestAuthorities
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = TestAuthoritiesMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TestAuthoritiesMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct TestAuthoritiesMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for TestAuthoritiesMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        let authorities: HashSet<String> = req
            .headers()
            .get("x-test-authorities")
            .and_then(|value| value.to_str().ok())
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
                    .collect::<HashSet<String>>()
            })
            .unwrap_or_default();

        req.attach(authorities);

        Box::pin(async move {
            service
                .call(req)
                .await
                .map(ServiceResponse::map_into_left_body)
        })
    }
}
