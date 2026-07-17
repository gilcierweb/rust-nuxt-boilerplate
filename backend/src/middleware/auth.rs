#![allow(dead_code)]

use actix_web::{FromRequest, HttpMessage, HttpRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::future::{Ready, ready};

use crate::{
    errors::{AppError, AppResult},
    models::role::ROLE_ADMIN,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub profile_id: uuid::Uuid,
    pub role: i32,
    #[serde(default = "default_token_use")]
    pub token_use: String,
    pub exp: usize,
    pub iat: usize,
}

impl Claims {
    pub fn has_role(&self, role: &str) -> bool {
        match role.to_ascii_lowercase().as_str() {
            "admin" => self.role == ROLE_ADMIN.as_i32(),
            "operator" | "moderator" | "support" | "creator" | "agency" => {
                self.role == crate::models::role::ROLE_OPERATOR.as_i32()
            },
            "viewer" | "fan" => self.role == crate::models::role::ROLE_VIEWER.as_i32(),
            _ => false,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == ROLE_ADMIN.as_i32()
    }

    pub fn is_operator_or_higher(&self) -> bool {
        self.role >= crate::models::role::ROLE_OPERATOR.as_i32()
    }

    pub fn profile_id(&self) -> Result<uuid::Uuid, AppError> {
        Ok(self.profile_id)
    }
}

pub const ACCESS_TOKEN_USE: &str = "access";
pub const WEBSOCKET_TOKEN_USE: &str = "ws";

fn default_token_use() -> String {
    ACCESS_TOKEN_USE.to_string()
}

#[derive(Clone)]
pub struct PublicRoute {
    pub method: Option<actix_web::http::Method>,
    pub pattern: String,
}

impl PublicRoute {
    pub fn method(method: actix_web::http::Method, pattern: &str) -> Self {
        Self {
            method: Some(method),
            pattern: pattern.to_string(),
        }
    }

    pub fn any(pattern: &str) -> Self {
        Self {
            method: None,
            pattern: pattern.to_string(),
        }
    }

    fn matches(&self, method: &actix_web::http::Method, path: &str) -> bool {
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

pub fn bearer_exempt_routes() -> Vec<PublicRoute> {
    use actix_web::http::Method;

    vec![
        PublicRoute::method(Method::POST, "/api/v1/auth/login"),
        PublicRoute::method(Method::POST, "/api/v1/auth/login/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/register"),
        PublicRoute::method(Method::POST, "/api/v1/auth/register/"),
        PublicRoute::method(Method::GET, "/api/v1/auth/confirm"),
        PublicRoute::method(Method::GET, "/api/v1/auth/confirm/"),
        PublicRoute::method(Method::GET, "/api/v1/auth/session"),
        PublicRoute::method(Method::GET, "/api/v1/auth/session/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/refresh"),
        PublicRoute::method(Method::POST, "/api/v1/auth/refresh/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/logout"),
        PublicRoute::method(Method::POST, "/api/v1/auth/logout/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/recover"),
        PublicRoute::method(Method::POST, "/api/v1/auth/recover/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/reset"),
        PublicRoute::method(Method::POST, "/api/v1/auth/reset/"),
        PublicRoute::method(Method::GET, "/api/v1/health"),
        PublicRoute::method(Method::GET, "/api/v1/health/"),
        PublicRoute::method(Method::GET, "/api/v1/metrics"),
        PublicRoute::method(Method::GET, "/api/v1/metrics/"),
        PublicRoute::method(Method::POST, "/api/v1/webhooks/stripe"),
        PublicRoute::method(Method::POST, "/api/v1/webhooks/pix"),
        PublicRoute::method(Method::GET, "/api/v1/ws"),
    ]
}

pub struct AuthUser {
    claims: Claims,
}

impl AuthUser {
    pub fn claims(&self) -> &Claims {
        &self.claims
    }

    pub fn id(&self) -> uuid::Uuid {
        self.claims.sub
    }

    pub fn profile_id(&self) -> uuid::Uuid {
        self.claims.profile_id
    }

    pub fn role(&self) -> i32 {
        self.claims.role
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.claims.has_role(role)
    }

    pub fn is_admin(&self) -> bool {
        self.claims.is_admin()
    }

    pub fn is_operator_or_higher(&self) -> bool {
        self.claims.is_operator_or_higher()
    }

    pub fn profile_id_option(&self) -> Option<uuid::Uuid> {
        Some(self.claims.profile_id)
    }
}

impl FromRequest for AuthUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let claims = req.extensions().get::<Claims>().cloned();

        ready(
            claims
                .map(|c| AuthUser { claims: c })
                .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string())),
        )
    }
}

pub fn create_token(
    user_id: uuid::Uuid,
    profile_id: uuid::Uuid,
    role: i32,
    jwt_secret: &str,
    expiry_secs: i64,
) -> AppResult<String> {
    create_token_for_use(
        user_id,
        profile_id,
        role,
        jwt_secret,
        expiry_secs,
        ACCESS_TOKEN_USE,
    )
}

#[allow(dead_code)]
pub fn create_ws_token(
    user_id: uuid::Uuid,
    profile_id: uuid::Uuid,
    role: i32,
    jwt_secret: &str,
    expiry_secs: i64,
) -> AppResult<String> {
    create_token_for_use(
        user_id,
        profile_id,
        role,
        jwt_secret,
        expiry_secs,
        WEBSOCKET_TOKEN_USE,
    )
}

fn create_token_for_use(
    user_id: uuid::Uuid,
    profile_id: uuid::Uuid,
    role: i32,
    jwt_secret: &str,
    expiry_secs: i64,
    token_use: &str,
) -> AppResult<String> {
    create_token_with_kid(
        user_id,
        profile_id,
        role,
        jwt_secret,
        expiry_secs,
        token_use,
        None,
    )
}

pub fn create_token_with_kid(
    user_id: uuid::Uuid,
    profile_id: uuid::Uuid,
    role: i32,
    jwt_secret: &str,
    expiry_secs: i64,
    token_use: &str,
    kid: Option<&str>,
) -> AppResult<String> {
    use chrono::Utc;
    use jsonwebtoken::encode;

    let now = Utc::now();
    let exp = (now + chrono::Duration::seconds(expiry_secs)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        profile_id,
        role,
        token_use: token_use.to_string(),
        exp,
        iat,
    };

    let mut header = jsonwebtoken::Header::default();
    if let Some(k) = kid {
        header.kid = Some(k.to_string());
    }

    encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token creation failed: {}", e)))
}

pub fn verify_token(token: &str, jwt_secret: &str) -> AppResult<Claims> {
    verify_token_for_use(token, jwt_secret, ACCESS_TOKEN_USE)
}

pub fn verify_ws_token(token: &str, jwt_secret: &str) -> AppResult<Claims> {
    verify_token_for_use(token, jwt_secret, WEBSOCKET_TOKEN_USE)
}

pub fn verify_token_with_secrets(
    token: &str,
    secrets: &[crate::config::app_config::JwtSecretKey],
    expected_use: &str,
) -> AppResult<Claims> {
    let token_header = {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::Unauthorized("Invalid token format".to_string()));
        }
        use base64::Engine;
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|_| AppError::Unauthorized("Invalid token header".to_string()))?;
        serde_json::from_slice::<jsonwebtoken::Header>(&decoded)
            .map_err(|_| AppError::Unauthorized("Invalid token header".to_string()))?
    };

    let kid = token_header.kid.as_deref();

    // If kid is present, try to find matching secret first
    if let Some(k) = kid {
        if let Some(secret_key) = secrets.iter().find(|s| s.kid == k && s.is_active()) {
            if let Ok(claims) = verify_token_for_use(token, &secret_key.secret, expected_use) {
                return Ok(claims);
            }
        }
    }

    // Fallback: try all active secrets
    for secret_key in secrets.iter().filter(|s| s.is_active()) {
        if let Ok(claims) = verify_token_for_use(token, &secret_key.secret, expected_use) {
            return Ok(claims);
        }
    }

    Err(AppError::Unauthorized("Invalid token".to_string()))
}

fn verify_token_for_use(token: &str, jwt_secret: &str, expected_use: &str) -> AppResult<Claims> {
    use jsonwebtoken::{DecodingKey, Validation, decode};

    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.required_spec_claims = HashSet::from([
        "exp".to_string(),
        "iat".to_string(),
        "sub".to_string(),
        "token_use".to_string(),
    ]);

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    if claims.token_use != expected_use {
        return Err(AppError::Unauthorized("Invalid token use".to_string()));
    }

    Ok(claims)
}

pub use super::auth_middleware::{JwtAuth, JwtAuthConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitCategory {
    AuthStrict,
    AuthSession,
    Default,
}

pub fn rate_limit_category(method: &actix_web::http::Method, path: &str) -> RateLimitCategory {
    let is_auth_path = path.starts_with("/api/v1/auth/");
    if !is_auth_path {
        return RateLimitCategory::Default;
    }

    let is_strict = matches!(
        (method.as_str(), path),
        ("POST", "/api/v1/auth/login")
            | ("POST", "/api/v1/auth/login/")
            | ("POST", "/api/v1/auth/register")
            | ("POST", "/api/v1/auth/register/")
            | ("POST", "/api/v1/auth/recover")
            | ("POST", "/api/v1/auth/recover/")
            | ("POST", "/api/v1/auth/reset")
            | ("POST", "/api/v1/auth/reset/")
    );

    if is_strict {
        return RateLimitCategory::AuthStrict;
    }

    let is_session = matches!(
        (method.as_str(), path),
        ("POST", "/api/v1/auth/refresh")
            | ("POST", "/api/v1/auth/refresh/")
            | ("GET", "/api/v1/auth/session")
            | ("GET", "/api/v1/auth/session/")
    );

    if is_session {
        return RateLimitCategory::AuthSession;
    }

    RateLimitCategory::Default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_has_role() {
        let claims = Claims {
            sub: uuid::Uuid::new_v4(),
            profile_id: uuid::Uuid::new_v4(),
            role: 1,
            token_use: "access".to_string(),
            exp: 0,
            iat: 0,
        };
        assert!(claims.is_admin());
        assert!(!claims.has_role("operator"));
    }

    #[test]
    fn test_bearer_exempt_routes() {
        let routes = bearer_exempt_routes();
        assert!(routes.iter().any(|r| r.pattern == "/api/v1/auth/login"));
        assert!(routes.iter().any(|r| r.pattern == "/api/v1/health"));
    }

    #[test]
    fn create_token_with_kid_sets_header_kid() {
        use crate::config::app_config::JwtSecretKey;
        use chrono::Utc;

        let sub = uuid::Uuid::new_v4();
        let profile_id = uuid::Uuid::new_v4();

        let secret = JwtSecretKey {
            kid: "key-2026-01".to_string(),
            secret: "test-secret-for-kid".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };

        let token = create_token_with_kid(
            sub,
            profile_id,
            0,
            &secret.secret,
            3600,
            "access",
            Some(&secret.kid),
        )
        .unwrap();

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);

        use base64::Engine;
        let header_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[0])
            .unwrap();
        let header: jsonwebtoken::Header = serde_json::from_slice(&header_bytes).unwrap();
        assert_eq!(header.kid.as_deref(), Some("key-2026-01"));
    }

    #[test]
    fn verify_token_with_secrets_matches_kid() {
        use crate::config::app_config::JwtSecretKey;
        use chrono::Utc;

        let sub = uuid::Uuid::new_v4();
        let profile_id = uuid::Uuid::new_v4();

        let secret_active = JwtSecretKey {
            kid: "active-key".to_string(),
            secret: "active-secret-value".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };
        let secret_expired = JwtSecretKey {
            kid: "expired-key".to_string(),
            secret: "expired-secret-value".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: Some(Utc::now().naive_utc()),
        };

        let secrets = vec![secret_expired, secret_active.clone()];

        let token = create_token_with_kid(
            sub,
            profile_id,
            0,
            &secret_active.secret,
            3600,
            "access",
            Some(&secret_active.kid),
        )
        .unwrap();
        let verified = verify_token_with_secrets(&token, &secrets, "access").unwrap();
        assert_eq!(verified.sub, sub);
        assert_eq!(verified.profile_id, profile_id);
    }

    #[test]
    fn verify_token_with_secrets_falls_back_to_other_secrets() {
        use crate::config::app_config::JwtSecretKey;
        use chrono::Utc;

        let sub = uuid::Uuid::new_v4();
        let profile_id = uuid::Uuid::new_v4();

        let secret1 = JwtSecretKey {
            kid: "key-1".to_string(),
            secret: "secret-number-one".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };
        let secret2 = JwtSecretKey {
            kid: "key-2".to_string(),
            secret: "secret-number-two".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };

        let secrets = vec![secret1.clone(), secret2];

        let token = create_token_with_kid(
            sub,
            profile_id,
            0,
            &secret1.secret,
            3600,
            "access",
            Some(&secret1.kid),
        )
        .unwrap();
        let verified = verify_token_with_secrets(&token, &secrets, "access").unwrap();
        assert_eq!(verified.sub, sub);
    }

    #[test]
    fn verify_token_with_secrets_rejects_expired_keys() {
        use crate::config::app_config::JwtSecretKey;
        use chrono::Utc;

        let sub = uuid::Uuid::new_v4();
        let profile_id = uuid::Uuid::new_v4();

        let expired = JwtSecretKey {
            kid: "expired".to_string(),
            secret: "expired-secret".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: Some(Utc::now().naive_utc()),
        };

        let secrets = vec![expired.clone()];
        let token = create_token_with_kid(
            sub,
            profile_id,
            0,
            &expired.secret,
            3600,
            "access",
            Some(&expired.kid),
        )
        .unwrap();
        let result = verify_token_with_secrets(&token, &secrets, "access");
        assert!(result.is_err());
    }

    #[test]
    fn verify_token_with_secrets_rejects_wrong_use() {
        use crate::config::app_config::JwtSecretKey;
        use chrono::Utc;

        let sub = uuid::Uuid::new_v4();
        let profile_id = uuid::Uuid::new_v4();

        let secret = JwtSecretKey {
            kid: "key-1".to_string(),
            secret: "some-secret".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };

        let secrets = vec![secret.clone()];
        let token = create_token_with_kid(
            sub,
            profile_id,
            0,
            &secret.secret,
            3600,
            "access",
            Some(&secret.kid),
        )
        .unwrap();
        let result = verify_token_with_secrets(&token, &secrets, "websocket");
        assert!(result.is_err());
    }
}
