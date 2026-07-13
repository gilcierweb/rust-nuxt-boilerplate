#![allow(dead_code)]

use actix_web::{
    Error as ActixError, HttpResponse,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures::future::{LocalBoxFuture, Ready, ready};
use std::{marker::PhantomData, rc::Rc};

use crate::config::AppConfig;

pub struct CsrfProtection {
    exclude_paths: Vec<String>,
}

impl CsrfProtection {
    pub fn new(exclude_paths: Vec<String>) -> Self {
        Self { exclude_paths }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = CsrfProtectionMiddleware<S, B>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfProtectionMiddleware {
            service: Rc::new(service),
            exclude_paths: self.exclude_paths.clone(),
            _phantom: PhantomData,
        }))
    }
}

pub struct CsrfProtectionMiddleware<S, B> {
    service: Rc<S>,
    exclude_paths: Vec<String>,
    _phantom: PhantomData<B>,
}

impl<S, B> Service<ServiceRequest> for CsrfProtectionMiddleware<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let exclude_paths = self.exclude_paths.clone();

        Box::pin(async move {
            let should_check = {
                let config_opt = req.app_data::<AppConfig>().cloned();
                let is_prod = config_opt.map(|c| c.is_production_like()).unwrap_or(false);

                if !is_prod {
                    let res = svc.call(req).await?;
                    return Ok(res.map_into_boxed_body());
                }

                let path = req.uri().path();
                !exclude_paths.iter().any(|p| path.starts_with(p))
            };

            if should_check {
                let method = req.method();

                if method == actix_web::http::Method::POST
                    || method == actix_web::http::Method::PUT
                    || method == actix_web::http::Method::PATCH
                    || method == actix_web::http::Method::DELETE
                {
                    let csrf_token = req
                        .headers()
                        .get("X-CSRF-Token")
                        .and_then(|h| h.to_str().ok());

                    if csrf_token.is_none() {
                        let response = HttpResponse::Forbidden()
                            .json(serde_json::json!({
                                "error": {
                                    "code": "CSRF_TOKEN_MISSING",
                                    "message": "CSRF token required"
                                }
                            }))
                            .map_into_boxed_body();

                        let (req, _) = req.into_parts();
                        return Ok(ServiceResponse::new(req, response));
                    }
                }
            }

            let res = svc.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}
