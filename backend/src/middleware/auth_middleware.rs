#![allow(dead_code)]

use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures::future::{LocalBoxFuture, Ready, ready};
use std::{rc::Rc, sync::Arc};

use crate::{
    AppState,
    middleware::auth::{Claims, verify_token},
    repositories::access_token_blacklist::AccessTokenBlacklist,
};

/// Actix-Web middleware that validates the `Authorization: Bearer <token>` header.
/// On success, inserts `Claims` into request extensions for handlers to extract.
pub struct JwtAuth {
    public_paths: Vec<String>,
    token_blacklist: Option<Arc<AccessTokenBlacklist>>,
}

impl JwtAuth {
    pub fn new(_config: Arc<crate::config::AppConfig>, public_paths: Vec<String>) -> Self {
        Self {
            public_paths,
            token_blacklist: None,
        }
    }

    pub fn with_token_blacklist(mut self, blacklist: Arc<AccessTokenBlacklist>) -> Self {
        self.token_blacklist = Some(blacklist);
        self
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
            public_paths: self.public_paths.clone(),
            token_blacklist: self.token_blacklist.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    public_paths: Vec<String>,
    token_blacklist: Option<Arc<AccessTokenBlacklist>>,
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
        let public_paths = self.public_paths.clone();
        let token_blacklist = self.token_blacklist.clone();

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
                Some(t) => {
                    // Check token blacklist
                    if let Some(blacklist) = &token_blacklist {
                        let token_hash = crate::repositories::access_token_blacklist::hash_token_for_blacklist(&t);
                        if blacklist.is_blacklisted(&token_hash).await.unwrap_or(false) {
                            return Err(actix_web::error::ErrorUnauthorized("Token revoked"));
                        }
                    }

                    // Get JWT secret from AppState
                    let state = req.app_data::<actix_web::web::Data<AppState>>();
                    let secret = state
                        .as_ref()
                        .map(|s| s.config.jwt_secret.clone())
                        .unwrap_or_default();

                    match verify_token(&t, &secret) {
                        Ok(claims) => {
                            req.extensions_mut().insert(claims);
                            svc.call(req).await
                        }
                        Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
                    }
                },
            }
        })
    }
}

/// Extract claims from request extensions (call after JwtAuth middleware).
pub fn extract_claims(req: &actix_web::HttpRequest) -> Result<Claims, crate::errors::AppError> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| crate::errors::AppError::Unauthorized("Not authenticated".to_string()))
}
