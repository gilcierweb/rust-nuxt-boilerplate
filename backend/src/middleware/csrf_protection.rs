use actix_web::{
    cookie::{Cookie, SameSite},
    Error as ActixError, HttpResponse,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
};
use futures::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;

use crate::config::AppConfig;

const CSRF_COOKIE_NAME: &str = "csrf_token";

pub struct CsrfProtection {
    exclude_paths: Vec<String>,
}

impl CsrfProtection {
    pub fn new(exclude_paths: Vec<String>) -> Self {
        // Default public auth endpoints that should be excluded from CSRF
        // These are endpoints called without prior session
        let mut defaults = vec![
            // Login/Register
            "/api/v1/auth/login".to_string(),
            "/api/v1/auth/login/".to_string(),
            "/api/v1/auth/register".to_string(),
            "/api/v1/auth/register/".to_string(),
            // Password recovery
            "/api/v1/auth/recover".to_string(),
            "/api/v1/auth/recover/".to_string(),
            "/api/v1/auth/reset".to_string(),
            "/api/v1/auth/reset/".to_string(),
            // Token refresh and logout (can be called without prior session)
            "/api/v1/auth/refresh".to_string(),
            "/api/v1/auth/refresh/".to_string(),
            "/api/v1/auth/logout".to_string(),
            "/api/v1/auth/logout/".to_string(),
            // Email confirmation
            "/api/v1/auth/confirm".to_string(),
            "/api/v1/auth/confirm/".to_string(),
            // Webhooks (have their own verification mechanisms)
            "/api/v1/webhooks".to_string(),
            "/api/v1/webhooks/".to_string(),
        ];
        
        // Add any additional paths passed by the caller
        defaults.extend(exclude_paths);
        
        Self { 
            exclude_paths: defaults
        }
    }
}

impl<S> Transform<S, ServiceRequest> for CsrfProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = ActixError> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = ActixError;
    type InitError = ();
    type Transform = CsrfProtectionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfProtectionMiddleware {
            service: Rc::new(service),
            exclude_paths: self.exclude_paths.clone(),
        }))
    }
}

pub struct CsrfProtectionMiddleware<S> {
    service: Rc<S>,
    exclude_paths: Vec<String>,
}

impl<S> Service<ServiceRequest> for CsrfProtectionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = ActixError> + 'static,
    S::Future: 'static,
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
                let path = req.uri().path();
                !exclude_paths.iter().any(|p| path.starts_with(p))
            };

            if should_check {
                let method = req.method();

                if method == Method::POST
                    || method == Method::PUT
                    || method == Method::PATCH
                    || method == Method::DELETE
                {
                    // Get CSRF token from header
                    let header_token = req
                        .headers()
                        .get("csrf-token")
                        .and_then(|h| h.to_str().ok())
                        .map(|s| s.to_string());

                    // Get CSRF token from cookie
                    let cookie_token = req
                        .cookie(CSRF_COOKIE_NAME)
                        .map(|c| c.value().to_string());

                    let is_valid = match (&header_token, &cookie_token) {
                        (Some(header), Some(cookie)) => {
                            // Validate that header matches cookie
                            header == cookie
                        }
                        _ => false,
                    };

                    if !is_valid {
                        let response = HttpResponse::Forbidden()
                            .json(serde_json::json!({
                                "error": {
                                    "code": "CSRF_TOKEN_INVALID",
                                    "message": "CSRF token invalid or missing"
                                }
                            }))
                            .map_into_boxed_body();

                        let (req, _) = req.into_parts();
                        return Ok(ServiceResponse::new(req, response));
                    }
                }
            }

            let mut res = svc.call(req).await?;

            // Set CSRF token cookie if not already present
            let config_opt = res.request().app_data::<AppConfig>().cloned();
            if let Some(config) = config_opt {
                if !res.response().headers().contains_key("set-cookie") {
                    // Generate CSRF token using HMAC with secret key
                    let csrf_token = generate_csrf_token(&config.csrf_secret_key);
                    let cookie = build_csrf_cookie(&config, &csrf_token);

                    res.response_mut()
                        .headers_mut()
                        .append(
                            actix_web::http::header::SET_COOKIE,
                            cookie.to_string().parse().unwrap(),
                        );
                }
            }

            Ok(res.map_into_boxed_body())
        })
    }
}

fn generate_csrf_token(secret_key: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use std::time::{SystemTime, UNIX_EPOCH};

    type HmacSha256 = Hmac<Sha256>;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(&timestamp.to_be_bytes());

    let result = mac.finalize();
    format!("{:x}", result.into_bytes())
}

fn build_csrf_cookie(config: &AppConfig, token: &str) -> Cookie<'static> {
    let same_site = match std::env::var("AUTH_COOKIE_SAME_SITE")
        .unwrap_or_else(|_| "lax".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "none" if !is_production_like(config) => SameSite::Lax,
        "none" => SameSite::None,
        "strict" => SameSite::Strict,
        _ => SameSite::Lax,
    };

    let mut cookie = Cookie::build(CSRF_COOKIE_NAME, token.to_owned())
        .http_only(false) // Must be readable by JavaScript for CSRF
        .path("/")
        .same_site(same_site)
        .max_age(actix_web::cookie::time::Duration::hours(24));

    // Don't set Secure flag in development (HTTP)
    if is_production_like(config) {
        cookie = cookie.secure(true);
    }

    cookie.finish()
}

fn is_production_like(config: &AppConfig) -> bool {
    matches!(
        config.environment,
        crate::config::app_config::Environment::Staging | crate::config::app_config::Environment::Production
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_generation() {
        let token = generate_csrf_token("test-secret");
        assert_eq!(token.len(), 64); // HMAC-SHA256 hex = 64 chars
    }
}
