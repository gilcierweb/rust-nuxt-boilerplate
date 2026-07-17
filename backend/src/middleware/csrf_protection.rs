use actix_web::{
    Error as ActixError, HttpResponse,
    body::BoxBody,
    cookie::{Cookie, SameSite},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
};
use futures::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;

use crate::config::AppConfig;

const CSRF_COOKIE_NAME: &str = "csrf_token";

/// CSRF token expiry: 15 minutes.
/// After this, the token is rejected even if the cookie hasn't expired.
const CSRF_TOKEN_EXPIRY_SECS: u64 = 15 * 60;

/// Grace period after rotation: old token is still accepted for 30 seconds.
/// This prevents race conditions from concurrent state-changing requests.
const CSRF_ROTATION_GRACE_SECS: u64 = 30;

pub struct CsrfProtection {
    exclude_paths: Vec<String>,
}

impl CsrfProtection {
    pub fn new(exclude_paths: Vec<String>) -> Self {
        let mut defaults = vec![
            "/api/v1/auth/login".to_string(),
            "/api/v1/auth/login/".to_string(),
            "/api/v1/auth/register".to_string(),
            "/api/v1/auth/register/".to_string(),
            "/api/v1/auth/recover".to_string(),
            "/api/v1/auth/recover/".to_string(),
            "/api/v1/auth/reset".to_string(),
            "/api/v1/auth/reset/".to_string(),
            "/api/v1/auth/refresh".to_string(),
            "/api/v1/auth/refresh/".to_string(),
            "/api/v1/auth/logout".to_string(),
            "/api/v1/auth/logout/".to_string(),
            "/api/v1/auth/confirm".to_string(),
            "/api/v1/auth/confirm/".to_string(),
            "/api/v1/webhooks".to_string(),
            "/api/v1/webhooks/".to_string(),
        ];

        defaults.extend(exclude_paths);

        Self {
            exclude_paths: defaults,
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
                let path_excluded = exclude_paths.iter().any(|p| path.starts_with(p));

                let has_bearer_token = req
                    .headers()
                    .get("authorization")
                    .and_then(|h| h.to_str().ok())
                    .map(|h| h.starts_with("Bearer "))
                    .unwrap_or(false);

                !path_excluded && !has_bearer_token
            };

            let is_state_changing = {
                let method = req.method();
                method == Method::POST
                    || method == Method::PUT
                    || method == Method::PATCH
                    || method == Method::DELETE
            };

            if should_check && is_state_changing {
                let header_token = req
                    .headers()
                    .get("csrf-token")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string());

                let cookie_token = req.cookie(CSRF_COOKIE_NAME).map(|c| c.value().to_string());

                let secret_key = req
                    .app_data::<AppConfig>()
                    .map(|c| c.csrf_secret_key.clone())
                    .or_else(|| {
                        req.app_data::<actix_web::web::Data<crate::AppState>>()
                            .map(|s| s.config.csrf_secret_key.clone())
                    });

                let is_valid = match (&header_token, &cookie_token, &secret_key) {
                    (Some(header), Some(cookie), Some(key)) => {
                        header == cookie && validate_csrf_token(header, key)
                    },
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

            let mut res = svc.call(req).await?;

            // Rotate CSRF token on successful state-changing requests
            if should_check && is_state_changing {
                if let Some(config) = res.request().app_data::<AppConfig>().cloned() {
                    let new_token = generate_csrf_token(&config.csrf_secret_key);
                    let cookie = build_csrf_cookie(&config, &new_token);

                    res.response_mut().headers_mut().append(
                        actix_web::http::header::SET_COOKIE,
                        cookie.to_string().parse().unwrap(),
                    );
                }
            }

            // Set CSRF token cookie on GET responses if not already present
            if !is_state_changing && !res.response().headers().contains_key("set-cookie") {
                if let Some(config) = res.request().app_data::<AppConfig>().cloned() {
                    let csrf_token = generate_csrf_token(&config.csrf_secret_key);
                    let cookie = build_csrf_cookie(&config, &csrf_token);

                    res.response_mut().headers_mut().append(
                        actix_web::http::header::SET_COOKIE,
                        cookie.to_string().parse().unwrap(),
                    );
                }
            }

            Ok(res.map_into_boxed_body())
        })
    }
}

/// Generate a CSRF token with embedded timestamp: `{timestamp}.{nonce}.{hmac}`
fn generate_csrf_token(secret_key: &str) -> String {
    use hmac::{Hmac, Mac};
    use rand::RngCore;
    use rand::rngs::OsRng;
    use sha2::Sha256;
    use std::time::{SystemTime, UNIX_EPOCH};

    type HmacSha256 = Hmac<Sha256>;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut nonce = [0u8; 16];
    OsRng.fill_bytes(&mut nonce);

    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");

    mac.update(&timestamp.to_be_bytes());
    mac.update(&nonce);

    let result = mac.finalize();
    let sig = hex::encode(result.into_bytes());

    format!("{}.{}.{}", timestamp, hex::encode(nonce), sig)
}

/// Validate a CSRF token: check HMAC integrity and expiry.
/// Also accepts tokens within the grace period after rotation.
fn validate_csrf_token(token: &str, secret_key: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use std::time::{SystemTime, UNIX_EPOCH};

    type HmacSha256 = Hmac<Sha256>;

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    let timestamp_str = parts[0];
    let nonce_str = parts[1];
    let sig_str = parts[2];

    let timestamp = match timestamp_str.parse::<u64>() {
        Ok(ts) => ts,
        Err(_) => return false,
    };

    let nonce = match hex::decode(nonce_str) {
        Ok(n) => n,
        Err(_) => return false,
    };

    let sig = match hex::decode(sig_str) {
        Ok(s) => s,
        Err(_) => return false,
    };

    if nonce.len() != 16 {
        return false;
    }

    // Recompute HMAC and compare
    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(&timestamp.to_be_bytes());
    mac.update(&nonce);

    if mac.verify_slice(&sig).is_err() {
        return false;
    }

    // Check token expiry (with grace period for rotation)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let max_age = CSRF_TOKEN_EXPIRY_SECS + CSRF_ROTATION_GRACE_SECS;
    now.saturating_sub(timestamp) <= max_age
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
        .http_only(false)
        .path("/")
        .same_site(same_site)
        .max_age(actix_web::cookie::time::Duration::minutes(
            (CSRF_TOKEN_EXPIRY_SECS / 60) as i64,
        ));

    if is_production_like(config) {
        cookie = cookie.secure(true);
    }

    cookie.finish()
}

fn is_production_like(config: &AppConfig) -> bool {
    matches!(
        config.environment,
        crate::config::app_config::Environment::Staging
            | crate::config::app_config::Environment::Production
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_generation() {
        let token = generate_csrf_token("test-secret");
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);

        let timestamp = parts[0].parse::<u64>().unwrap();
        assert!(timestamp > 0);

        assert_eq!(parts[1].len(), 32); // 16 bytes hex
        assert_eq!(parts[2].len(), 64); // 32 bytes hex (HMAC-SHA256)
    }

    #[test]
    fn test_csrf_token_validate_valid() {
        let token = generate_csrf_token("test-secret");
        assert!(validate_csrf_token(&token, "test-secret"));
    }

    #[test]
    fn test_csrf_token_validate_wrong_secret() {
        let token = generate_csrf_token("test-secret");
        assert!(!validate_csrf_token(&token, "wrong-secret"));
    }

    #[test]
    fn test_csrf_token_validate_tampered() {
        let token = generate_csrf_token("test-secret");
        let mut parts: Vec<&str> = token.split('.').collect();
        // Tamper with the nonce
        parts[1] = "00000000000000000000000000000000";
        let tampered = parts.join(".");
        assert!(!validate_csrf_token(&tampered, "test-secret"));
    }

    #[test]
    fn test_csrf_token_validate_expired() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create a token with timestamp that exceeds max age
        let expired_timestamp = now - CSRF_TOKEN_EXPIRY_SECS - CSRF_ROTATION_GRACE_SECS - 100;

        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut nonce = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut nonce);

        let mut mac = HmacSha256::new_from_slice(b"test-secret").unwrap();
        mac.update(&expired_timestamp.to_be_bytes());
        mac.update(&nonce);
        let sig = hex::encode(mac.finalize().into_bytes());

        let token = format!("{}.{}.{}", expired_timestamp, hex::encode(nonce), sig);

        assert!(!validate_csrf_token(&token, "test-secret"));
    }

    #[test]
    fn test_csrf_token_validate_malformed() {
        assert!(!validate_csrf_token("not-a-token", "secret"));
        assert!(!validate_csrf_token("abc.def", "secret"));
        assert!(!validate_csrf_token("abc.def.ghi.jkl", "secret"));
        assert!(!validate_csrf_token("not_a_number.abc.def", "secret"));
    }

    #[test]
    fn test_csrf_token_different_each_call() {
        let t1 = generate_csrf_token("test-secret");
        let t2 = generate_csrf_token("test-secret");
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_csrf_token_grace_period() {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        use std::time::{SystemTime, UNIX_EPOCH};
        type HmacSha256 = Hmac<Sha256>;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Token that expired just within grace period
        let token_time = now - CSRF_TOKEN_EXPIRY_SECS - 5; // within grace

        let mut nonce = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut nonce);

        let mut mac = HmacSha256::new_from_slice(b"test-secret").unwrap();
        mac.update(&token_time.to_be_bytes());
        mac.update(&nonce);
        let sig = hex::encode(mac.finalize().into_bytes());

        let token = format!("{}.{}.{}", token_time, hex::encode(nonce), sig);

        // Should still be valid within grace period
        assert!(validate_csrf_token(&token, "test-secret"));
    }

    #[test]
    fn test_csrf_token_expiry_constants() {
        assert_eq!(CSRF_TOKEN_EXPIRY_SECS, 900); // 15 minutes
        assert_eq!(CSRF_ROTATION_GRACE_SECS, 30); // 30 seconds
    }
}
