#![allow(dead_code)]

use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures::future::{LocalBoxFuture, Ready, ready};
use std::{rc::Rc, sync::Arc};

use crate::{
    config::AppConfig,
    errors::ApiError,
    services::token::{Claims, verify_access_token},
};

/// Actix-Web middleware that validates the `Authorization: Bearer <token>` header.
/// On success, inserts `Claims` into request extensions for handlers to extract.
pub struct JwtAuth {
    config: Arc<AppConfig>,
    public_paths: Vec<String>,
}

impl JwtAuth {
    pub fn new(config: Arc<AppConfig>, public_paths: Vec<String>) -> Self {
        Self {
            config,
            public_paths,
        }
    }

    /// Check if the request path matches any public path pattern
    fn is_public_path(&self, path: &str) -> bool {
        self.public_paths.iter().any(|pattern| {
            if pattern.ends_with('*') {
                // Wildcard match for prefixes
                let prefix = &pattern[..pattern.len() - 1];
                path.starts_with(prefix)
            } else {
                // Exact match
                path == pattern || path.starts_with(&format!("{}/", pattern))
            }
        })
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
            public_paths: self.public_paths.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    config: Arc<AppConfig>,
    public_paths: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let secret = self.config.jwt_secret.clone();
        let public_key = self.config.jwt_public_key.clone();
        let public_paths = self.public_paths.clone();

        Box::pin(async move {
            let path = req.uri().path();

            // Allow OPTIONS requests (CORS preflight) without auth
            if req.method() == actix_web::http::Method::OPTIONS {
                return svc.call(req).await;
            }

            // Check if path is public (no auth required)
            let is_public = public_paths.iter().any(|pattern| {
                if pattern.ends_with('*') {
                    let prefix = &pattern[..pattern.len() - 1];
                    path.starts_with(prefix)
                } else {
                    path == pattern || path.starts_with(&format!("{}/", pattern))
                }
            });

            if is_public {
                return svc.call(req).await;
            }

            // Extract Bearer token from Authorization header
            let token = req
                .headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|t| t.to_string());

            match token {
                None => Err(actix_web::error::ErrorUnauthorized(
                    "Missing Authorization header",
                )),
                Some(t) => match verify_access_token(&t, &secret, public_key.as_deref()) {
                    Ok(claims) => {
                        req.extensions_mut().insert(claims);
                        svc.call(req).await
                    }
                    Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
                },
            }
        })
    }
}

/// Extract claims from request extensions (call after JwtAuth middleware).
pub fn extract_claims(req: &actix_web::HttpRequest) -> Result<Claims, ApiError> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| ApiError::Unauthorized("Not authenticated".to_string()))
}
