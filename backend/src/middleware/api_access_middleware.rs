#![allow(dead_code)]

use std::rc::Rc;

use actix_web::{
    Error, ResponseError,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::Method,
    web,
};
use futures::future::{LocalBoxFuture, Ready, ready};

use crate::{
    AppState, errors::AppError, middleware::auth::verify_token,
    repositories::container::AppContainer,
};

#[derive(Clone)]
pub struct PublicRoute {
    method: Option<Method>,
    pattern: String,
}

impl PublicRoute {
    pub fn method(method: Method, pattern: &str) -> Self {
        Self {
            method: Some(method),
            pattern: pattern.to_string(),
        }
    }

    fn matches(&self, method: &Method, path: &str) -> bool {
        if let Some(expected_method) = &self.method
            && expected_method != method
        {
            return false;
        }

        if self.pattern.ends_with('*') {
            let prefix = &self.pattern[..self.pattern.len() - 1];
            path.starts_with(prefix)
        } else {
            path == self.pattern
        }
    }
}

pub struct ApiAccessGate {
    public_routes: Vec<PublicRoute>,
}

impl ApiAccessGate {
    pub fn new(public_routes: Vec<PublicRoute>) -> Self {
        Self { public_routes }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiAccessGate
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Transform = ApiAccessGateMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiAccessGateMiddleware {
            service: Rc::new(service),
            public_routes: self.public_routes.clone(),
        }))
    }
}

pub struct ApiAccessGateMiddleware<S> {
    service: Rc<S>,
    public_routes: Vec<PublicRoute>,
}

impl<S, B> Service<ServiceRequest> for ApiAccessGateMiddleware<S>
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
        let public_routes = self.public_routes.clone();

        Box::pin(async move {
            let path = req.path().to_string();
            let method = req.method().clone();

            if method == Method::OPTIONS
                || public_routes
                    .iter()
                    .any(|route| route.matches(&method, &path))
            {
                return service
                    .call(req)
                    .await
                    .map(ServiceResponse::map_into_left_body);
            }

            let token = req
                .headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|header| header.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "));

            let secret = req
                .app_data::<web::Data<AppState>>()
                .map(|state| state.config.jwt_secret.clone())
                .or_else(|| {
                    req.app_data::<web::Data<AppContainer>>()
                        .map(|container| container.config.jwt_secret.clone())
                })
                .unwrap_or_default();

            let is_valid = token
                .map(|token| verify_token(token, &secret).is_ok())
                .unwrap_or(false);

            if !is_valid {
                let response = AppError::Unauthorized(t!("middleware.unauthorized").into_owned())
                    .error_response()
                    .map_into_right_body();
                return Ok(req.into_response(response));
            }

            // Check token blacklist
            if let Some(token) = token
                && let Some(container) = req.app_data::<web::Data<AppContainer>>()
            {
                let token_hash = crate::repositories::access_token_blacklist::hash_token_for_blacklist(token);
                if container.access_token_blacklist.is_blacklisted(&token_hash).await.unwrap_or(false) {
                    let response = AppError::Unauthorized(t!("middleware.token_revoked").into_owned())
                        .error_response()
                        .map_into_right_body();
                    return Ok(req.into_response(response));
                }
            }

            service
                .call(req)
                .await
                .map(ServiceResponse::map_into_left_body)
        })
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{App, HttpResponse, http::Method, test, web};

    use super::{ApiAccessGate, PublicRoute};
    use crate::{
        middleware::auth::create_token,
        repositories::test_utils::mocks::{mock_app_config, mock_container},
    };

    #[actix_web::test]
    async fn rejects_missing_bearer_token_on_protected_route() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(mock_container()))
                .wrap(ApiAccessGate::new(vec![]))
                .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/protected").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn allows_public_route_without_bearer_token() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(mock_container()))
                .wrap(ApiAccessGate::new(vec![PublicRoute::method(
                    Method::GET,
                    "/public",
                )]))
                .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/public").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn allows_valid_bearer_token_on_protected_route() {
        let config = mock_app_config();
        let token = create_token(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            1,
            &config.jwt_secret,
            3600,
        )
        .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(mock_container()))
                .wrap(ApiAccessGate::new(vec![]))
                .default_service(web::to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", token),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}