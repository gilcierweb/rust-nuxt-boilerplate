#![allow(dead_code)]

use std::rc::Rc;
use std::sync::Arc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::Method;
use actix_web::{Error, HttpMessage};
use actix_web_grants::authorities::AttachAuthorities;
use futures::future::{LocalBoxFuture, Ready, ready};
use serde_json::json;

use crate::AppState;
use crate::authz::grants_extractor::build_authorities_for_claims;
use crate::middleware::auth::{
    ACCESS_TOKEN_USE, Claims, JwtVerifyOutcome, bearer_exempt_routes, verify_token_with_secrets,
};
use crate::repositories::access_token_blacklist::{
    AccessTokenBlacklist, is_token_revoked_by_epoch,
};

#[derive(Clone)]
pub struct JwtAuthConfig {
    pub public_paths: Vec<String>,
    /// Skip all Redis-backed revocation checks (per-token blacklist and
    /// per-user credential epoch). Intended for test setups without Redis;
    /// production wiring must always attach a token blacklist.
    pub skip_blacklist_check: bool,
    pub token_blacklist: Option<Arc<AccessTokenBlacklist>>,
}

impl JwtAuthConfig {
    pub fn new(public_paths: Vec<String>) -> Self {
        Self {
            public_paths,
            skip_blacklist_check: false,
            token_blacklist: None,
        }
    }

    pub fn with_token_blacklist(mut self, blacklist: Arc<AccessTokenBlacklist>) -> Self {
        self.token_blacklist = Some(blacklist);
        self
    }

    pub fn skip_blacklist_check(mut self) -> Self {
        self.skip_blacklist_check = true;
        self
    }

    fn is_public_path(&self, path: &str) -> bool {
        self.public_paths.iter().any(|pattern| {
            if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len() - 1];
                path.starts_with(prefix)
            } else {
                path == pattern || path.starts_with(&format!("{}/", pattern))
            }
        })
    }
}

#[derive(Clone)]
pub struct JwtAuth {
    config: JwtAuthConfig,
}

impl JwtAuth {
    pub fn new(config: JwtAuthConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(JwtAuthConfig::new(
            bearer_exempt_routes()
                .into_iter()
                .map(|r| r.pattern)
                .collect(),
        ))
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    config: JwtAuthConfig,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
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
        let svc = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            let path = req.uri().path().to_string();
            let method = req.method().clone();

            if method == Method::OPTIONS {
                return svc.call(req).await.map(ServiceResponse::map_into_left_body);
            }

            if config.is_public_path(&path) {
                return svc.call(req).await.map(ServiceResponse::map_into_left_body);
            }

            let token = req
                .headers()
                .get(actix_web::http::header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|t| t.to_string());

            match token {
                None => Err(actix_web::error::ErrorUnauthorized(
                    t!("middleware.missing_header").into_owned(),
                )),
                Some(token) => {
                    if !config.skip_blacklist_check
                        && let Some(blacklist) = &config.token_blacklist
                    {
                        let token_hash =
                            crate::repositories::access_token_blacklist::hash_token_for_blacklist(
                                &token,
                            );
                        match blacklist.is_blacklisted(&token_hash).await {
                            Ok(true) => {
                                return Err(actix_web::error::InternalError::from_response(
                                    "token_revoked",
                                    token_revoked_response(),
                                )
                                .into());
                            },
                            Ok(false) => {},
                            Err(error) => {
                                // Fail CLOSED: if revocation status cannot be
                                // checked, the token must not be accepted.
                                tracing::error!(
                                    event = "auth.blacklist_unavailable",
                                    error = %error,
                                    "Token revocation status unavailable (Redis); failing closed"
                                );
                                return Err(actix_web::error::InternalError::from_response(
                                    "auth_status_unavailable",
                                    auth_status_unavailable_response(),
                                )
                                .into());
                            },
                        }
                    }

                    // Optimized JWT secrets lookup:
                    // 1. Single `app_data` lookup for `AppState` (the common case)
                    // 2. Clone `Arc<Vec<JwtSecretKey>>` which is O(1) instead of cloning the Vec
                    // 3. Fallback to `AppContainer` only when `AppState` is unavailable
                    let secrets = req
                        .app_data::<actix_web::web::Data<AppState>>()
                        .map(|s| s.jwt_secrets.clone())
                        .or_else(|| {
                            req.app_data::<actix_web::web::Data<crate::repositories::container::AppContainer>>()
                                .map(|c| {
                                    Arc::new(c.config.jwt_secrets.clone())
                                })
                        })
                        .ok_or_else(|| {
                            tracing::error!(
                                event = "auth.jwt_config_missing",
                                "JWT secret not available: neither AppState nor AppContainer is configured in the middleware chain"
                            );
                            actix_web::error::ErrorInternalServerError(t!("errors.internal").into_owned())
                        })?;

                    match verify_token_with_secrets(&token, &secrets, ACCESS_TOKEN_USE) {
                        Ok(result) => {
                            let metrics = req
                                .app_data::<actix_web::web::Data<AppState>>()
                                .map(|s| s.metrics.clone());
                            if let Some(m) = metrics {
                                match result.outcome {
                                    JwtVerifyOutcome::DirectMatch => m.record_jwt_direct_match(),
                                    JwtVerifyOutcome::FallbackMatch => {
                                        m.record_jwt_fallback_match()
                                    },
                                    JwtVerifyOutcome::Rejected => m.record_jwt_rejected(),
                                }
                            }

                            // Reject tokens issued before the user's last
                            // credential change (password/2FA). The epoch
                            // marker lives in Redis with a TTL matching the
                            // access-token lifetime, so it disappears once
                            // every pre-change token has expired naturally.
                            if !config.skip_blacklist_check
                                && let Some(blacklist) = &config.token_blacklist
                            {
                                match blacklist.credential_epoch(&result.claims.sub).await {
                                    Ok(epoch)
                                        if is_token_revoked_by_epoch(result.claims.iat, epoch) =>
                                    {
                                        tracing::info!(
                                            event = "auth.token_revoked_by_epoch",
                                            user_id = %result.claims.sub,
                                            "Access token predates credential change; rejecting"
                                        );
                                        return Err(
                                            actix_web::error::InternalError::from_response(
                                                "token_revoked",
                                                token_revoked_response(),
                                            )
                                            .into(),
                                        );
                                    },
                                    Ok(_) => {},
                                    Err(error) => {
                                        tracing::error!(
                                            event = "auth.epoch_unavailable",
                                            error = %error,
                                            "Credential epoch unavailable (Redis); failing closed"
                                        );
                                        return Err(
                                            actix_web::error::InternalError::from_response(
                                                "auth_status_unavailable",
                                                auth_status_unavailable_response(),
                                            )
                                            .into(),
                                        );
                                    },
                                }
                            }

                            req.extensions_mut().insert(result.claims.clone());

                            let authorities = build_authorities_for_claims(
                                &result.claims,
                                req.app_data::<actix_web::web::Data<
                                    crate::repositories::container::AppContainer,
                                >>(),
                            )
                            .await;

                            req.attach(authorities);
                            svc.call(req).await.map(ServiceResponse::map_into_left_body)
                        },
                        Err(_) => {
                            let metrics = req
                                .app_data::<actix_web::web::Data<AppState>>()
                                .map(|s| s.metrics.clone());
                            if let Some(m) = metrics {
                                m.record_jwt_rejected();
                            }
                            let response = actix_web::HttpResponse::Unauthorized().json(json!({
                                "error": {
                                    "code": "TOKEN_EXPIRED",
                                    "message": t!("middleware.invalid_token").into_owned(),
                                }
                            }));
                            Err(actix_web::error::InternalError::from_response(
                                "invalid_token",
                                response,
                            )
                            .into())
                        },
                    }
                },
            }
        })
    }
}

pub fn extract_claims(req: &actix_web::HttpRequest) -> Result<Claims, crate::errors::AppError> {
    req.extensions().get::<Claims>().cloned().ok_or_else(|| {
        crate::errors::AppError::Unauthorized(t!("middleware.not_authenticated").into_owned())
    })
}

/// 401 response for tokens rejected by revocation checks (blacklist or
/// credential epoch). The `TOKEN_REVOKED` code tells the frontend to skip
/// its refresh-retry path — the refresh tokens are revoked alongside.
fn token_revoked_response() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Unauthorized().json(json!({
        "error": {
            "code": "TOKEN_REVOKED",
            "message": t!("middleware.token_revoked").into_owned(),
        }
    }))
}

/// 503 response used when revocation status cannot be verified (Redis
/// unavailable). Mirrors the rate limiter's fail-closed shape: the request
/// is neither accepted nor treated as an auth failure the client could
/// "refresh" its way out of.
fn auth_status_unavailable_response() -> actix_web::HttpResponse {
    actix_web::HttpResponse::ServiceUnavailable().json(json!({
        "error": {
            "code": "AUTH_STATUS_UNAVAILABLE",
            "message": t!("middleware.token_status_unavailable").into_owned(),
        }
    }))
}
