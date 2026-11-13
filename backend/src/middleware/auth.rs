#![allow(dead_code)]

use std::collections::HashSet;
use std::future::{Ready, ready};

use actix_web::{FromRequest, HttpMessage, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};
use crate::models::role::ROLE_ADMIN;

/// The canonical audience of this deployment, used when `JWT_AUDIENCE` names
/// nothing: the same service name that `frontend/wrangler.jsonc` deploys.
///
/// A token is minted for exactly one consumer, this API, so the `aud` claim
/// ties it to this application: without it a token signed by any active secret
/// of `jwt_secrets` would be accepted on every scope, and a second consumer
/// that ever shared a secret could replay its tokens against the admin routes.
/// A deployment may name a different audience through `JWT_AUDIENCE`; an empty
/// value falls back here and never disables the check.
pub const DEFAULT_TOKEN_AUDIENCE: &str = "rust-nuxt-boilerplate";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub profile_id: uuid::Uuid,
    /// Presentation role of the subject (1 admin, 2 operator, 3 viewer). It is
    /// derived from the effective permissions of the subject
    /// and never takes part in an authorization decision: only the grants read
    /// from the database decide what the subject may do.
    pub role: i32,
    #[serde(default = "default_token_use")]
    pub token_use: String,
    pub exp: usize,
    pub iat: usize,
    /// Not-before timestamp — equal to `iat`.
    /// Required so that `validate_nbf = true` actually enforces the check
    /// instead of silently passing because the claim is absent.
    pub nbf: usize,
    /// The consumer the token was minted for: this application. Required, so
    /// a token without it is refused instead of silently accepted.
    pub aud: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
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

    /// True when the claim carries the operator role or the admin one.
    ///
    /// The role numbers grow downwards (1 admin, 2 operator, 3 viewer), so the
    /// comparison is reversed.
    pub fn is_operator_or_higher(&self) -> bool {
        self.role <= crate::models::role::ROLE_OPERATOR.as_i32()
    }

    pub fn profile_id(&self) -> Result<uuid::Uuid, AppError> {
        Ok(self.profile_id)
    }
}

pub const ACCESS_TOKEN_USE: &str = "access";
pub const WEBSOCKET_TOKEN_USE: &str = "ws";

/// Outcome of JWT verification with multi-key fallback, for observability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtVerifyOutcome {
    /// Token verified with direct kid match (best case).
    DirectMatch,
    /// Token verified via fallback (kid mismatch or missing — possible rotation).
    FallbackMatch,
    /// Token rejected: kid not found and fallback also failed.
    Rejected,
}

/// Result of `verify_token_with_secrets` including outcome metadata.
pub struct JwtVerifyResult {
    pub claims: Claims,
    pub outcome: JwtVerifyOutcome,
}

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

/// Routes that skip JWT Bearer authentication.
///
/// Includes:
/// - Credential endpoints (login, register, recover, reset, confirm,
///   magic-link) — issued before any token exists.
/// - Cookie-authenticated endpoints (session, refresh) — they authenticate
///   via the httpOnly `refresh_token` cookie, not a Bearer header.
/// - logout — Bearer is optional; it blacklists the presented token.
/// - Health, webhooks and WebSocket handshake endpoints.
pub fn bearer_exempt_routes() -> Vec<PublicRoute> {
    use actix_web::http::Method;

    vec![
        PublicRoute::method(Method::POST, "/api/v1/auth/login"),
        PublicRoute::method(Method::POST, "/api/v1/auth/login/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/register"),
        PublicRoute::method(Method::POST, "/api/v1/auth/register/"),
        PublicRoute::method(Method::GET, "/api/v1/auth/confirm"),
        PublicRoute::method(Method::GET, "/api/v1/auth/confirm/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/logout"),
        PublicRoute::method(Method::POST, "/api/v1/auth/logout/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/recover"),
        PublicRoute::method(Method::POST, "/api/v1/auth/recover/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/reset"),
        PublicRoute::method(Method::POST, "/api/v1/auth/reset/"),
        // Cookie-authenticated session endpoints: no Bearer header exists.
        PublicRoute::method(Method::GET, "/api/v1/auth/session"),
        PublicRoute::method(Method::GET, "/api/v1/auth/session/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/refresh"),
        PublicRoute::method(Method::POST, "/api/v1/auth/refresh/"),
        // Password recovery alias.
        PublicRoute::method(Method::POST, "/api/v1/auth/forgot-password"),
        PublicRoute::method(Method::POST, "/api/v1/auth/forgot-password/"),
        // Passwordless magic-link flow.
        PublicRoute::method(Method::POST, "/api/v1/auth/magic-link"),
        PublicRoute::method(Method::POST, "/api/v1/auth/magic-link/"),
        PublicRoute::method(Method::POST, "/api/v1/auth/magic-link/verify"),
        PublicRoute::method(Method::POST, "/api/v1/auth/magic-link/verify/"),
        PublicRoute::method(Method::GET, "/api/v1/health"),
        PublicRoute::method(Method::GET, "/api/v1/health/"),
        // NOTE: /api/v1/metrics is intentionally NOT exempt from Bearer auth.
        // It requires API key authentication (enforced by RequireApiKey middleware).
        PublicRoute::method(Method::POST, "/api/v1/webhooks/stripe"),
        PublicRoute::method(Method::POST, "/api/v1/webhooks/pix"),
        PublicRoute::method(Method::GET, "/api/v1/ws"),
    ]
}

pub struct AuthUser {
    claims: Claims,
}

impl AuthUser {
    pub fn new(claims: Claims) -> Self {
        Self { claims }
    }

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
            claims.map(|c| AuthUser { claims: c }).ok_or_else(|| {
                AppError::Unauthorized(t!("middleware.not_authenticated").into_owned())
            }),
        )
    }
}

pub fn create_token(
    user_id: uuid::Uuid,
    profile_id: uuid::Uuid,
    role: i32,
    jwt_secret: &str,
    expiry_secs: i64,
    audience: &str,
) -> AppResult<String> {
    create_token_for_use(
        user_id,
        profile_id,
        role,
        jwt_secret,
        expiry_secs,
        ACCESS_TOKEN_USE,
        audience,
    )
}

#[allow(dead_code)]
pub fn create_ws_token(
    user_id: uuid::Uuid,
    profile_id: uuid::Uuid,
    role: i32,
    jwt_secret: &str,
    expiry_secs: i64,
    audience: &str,
) -> AppResult<String> {
    create_token_for_use(
        user_id,
        profile_id,
        role,
        jwt_secret,
        expiry_secs,
        WEBSOCKET_TOKEN_USE,
        audience,
    )
}

fn create_token_for_use(
    user_id: uuid::Uuid,
    profile_id: uuid::Uuid,
    role: i32,
    jwt_secret: &str,
    expiry_secs: i64,
    token_use: &str,
    audience: &str,
) -> AppResult<String> {
    create_token_with_kid(
        user_id,
        profile_id,
        role,
        jwt_secret,
        expiry_secs,
        token_use,
        None,
        audience,
    )
}

/// Mints a token, optionally stamping the key id in the header.
///
/// Eight explicit parameters, one per claim the token carries plus the key id:
/// the minting call sites live in the auth handlers and in tests, none of them
/// has a claims object to hand, and grouping the identity into a struct would
/// hide at a glance which claims this application actually issues.
#[allow(clippy::too_many_arguments)]
pub fn create_token_with_kid(
    user_id: uuid::Uuid,
    profile_id: uuid::Uuid,
    role: i32,
    jwt_secret: &str,
    expiry_secs: i64,
    token_use: &str,
    kid: Option<&str>,
    audience: &str,
) -> AppResult<String> {
    use chrono::Utc;
    use jsonwebtoken::encode;

    let now = Utc::now();
    let exp = (now + chrono::Duration::seconds(expiry_secs)).timestamp() as usize;
    let iat = now.timestamp() as usize;
    let jti = uuid::Uuid::new_v4().to_string();

    let claims = Claims {
        sub: user_id,
        profile_id,
        role,
        token_use: token_use.to_string(),
        exp,
        iat,
        // nbf = iat: the token is valid immediately upon issuance.
        // This field MUST be present so that `validate_nbf = true` in the
        // validator actually enforces the check rather than silently passing
        // because the claim is absent.
        nbf: iat,
        aud: audience.to_string(),
        jti: Some(jti),
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

pub fn verify_token(token: &str, jwt_secret: &str, audience: &str) -> AppResult<Claims> {
    verify_token_for_use(token, jwt_secret, ACCESS_TOKEN_USE, audience)
}

pub fn verify_ws_token(token: &str, jwt_secret: &str, audience: &str) -> AppResult<Claims> {
    verify_token_for_use(token, jwt_secret, WEBSOCKET_TOKEN_USE, audience)
}

pub fn verify_token_with_secrets(
    token: &str,
    secrets: &[crate::config::app_config::JwtSecretKey],
    expected_use: &str,
    audience: &str,
) -> AppResult<JwtVerifyResult> {
    let token_header = {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::Unauthorized(
                t!("auth.token.invalid_format").into_owned(),
            ));
        }
        use base64::Engine;
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|_| AppError::Unauthorized(t!("auth.token.invalid_header").into_owned()))?;
        serde_json::from_slice::<jsonwebtoken::Header>(&decoded)
            .map_err(|_| AppError::Unauthorized(t!("auth.token.invalid_header").into_owned()))?
    };

    let kid = token_header.kid.as_deref();
    let active_count = secrets.iter().filter(|s| s.is_active()).count();

    // If kid is present, try to find matching secret first
    if let Some(k) = kid {
        if let Some(secret_key) = secrets.iter().find(|s| s.kid == k && s.is_active()) {
            match verify_token_for_use(token, &secret_key.secret, expected_use, audience) {
                Ok(claims) => {
                    tracing::debug!(
                        event = "auth.jwt_verify",
                        kid = k,
                        matched_kid = k,
                        strategy = "direct",
                        "JWT verified with direct kid match"
                    );
                    return Ok(JwtVerifyResult {
                        claims,
                        outcome: JwtVerifyOutcome::DirectMatch,
                    });
                },
                Err(_) => {
                    tracing::warn!(
                        event = "auth.jwt_verify_failed",
                        kid = k,
                        matched_kid = %secret_key.kid,
                        strategy = "direct",
                        "JWT kid matched but token verification failed (wrong secret or tampered token)"
                    );
                    return Err(AppError::Unauthorized(
                        t!("middleware.invalid_token").into_owned(),
                    ));
                },
            }
        }

        // kid present but no matching active key — check if key exists but is expired
        if secrets.iter().any(|s| s.kid == k) {
            tracing::warn!(
                event = "auth.jwt_kid_expired",
                kid = k,
                strategy = "fallback",
                active_secrets = active_count,
                "JWT kid exists but key is expired/inactive, falling back to all active secrets"
            );
        } else {
            tracing::warn!(
                event = "auth.jwt_unknown_kid",
                kid = k,
                strategy = "fallback",
                active_secrets = active_count,
                "JWT kid not found in any secret key, falling back to all active secrets"
            );
        }
    }

    // Fallback: try all active secrets
    for secret_key in secrets.iter().filter(|s| s.is_active()) {
        if let Ok(claims) = verify_token_for_use(token, &secret_key.secret, expected_use, audience)
        {
            if kid.is_some() {
                tracing::warn!(
                    event = "auth.jwt_fallback_match",
                    token_kid = kid.unwrap_or_default(),
                    matched_kid = %secret_key.kid,
                    strategy = "fallback",
                    "JWT verified via fallback (kid mismatch) — possible key rotation or token issued with unknown key"
                );
            } else {
                tracing::info!(
                    event = "auth.jwt_fallback_match",
                    token_kid = "",
                    matched_kid = %secret_key.kid,
                    strategy = "fallback",
                    "JWT verified via fallback (no kid in token header)"
                );
            }
            return Ok(JwtVerifyResult {
                claims,
                outcome: JwtVerifyOutcome::FallbackMatch,
            });
        }
    }

    tracing::warn!(
        event = "auth.jwt_rejected",
        token_kid = kid.unwrap_or_default(),
        strategy = "all_failed",
        active_secrets = active_count,
        "JWT verification failed: no matching secret found"
    );

    Err(AppError::Unauthorized(
        t!("middleware.invalid_token").into_owned(),
    ))
}

fn verify_token_for_use(
    token: &str,
    jwt_secret: &str,
    expected_use: &str,
    audience: &str,
) -> AppResult<Claims> {
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

    // A blank audience would accept exactly the tokens that name no audience,
    // the hole the claim closes. The config never hands one (an empty
    // JWT_AUDIENCE falls back to the default), so this is the invariant at
    // the verify edge rather than a second config read.
    let audience = if audience.is_empty() {
        DEFAULT_TOKEN_AUDIENCE
    } else {
        audience
    };

    // Explicitly pin the algorithm to HS256.
    //
    // `Validation::default()` also uses HS256, but the explicit form:
    // 1. Guards against future library-default changes.
    // 2. Prevents the `"alg": "none"` downgrade attack — `jsonwebtoken` v9+
    //    rejects `none` by default, but explicit pinning adds a second layer.
    // 3. Makes the security contract self-documenting.
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.set_audience(&[audience]);
    validation.required_spec_claims = HashSet::from([
        "exp".to_string(),
        "iat".to_string(),
        "nbf".to_string(),
        "sub".to_string(),
        "token_use".to_string(),
        "aud".to_string(),
    ]);

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized(t!("middleware.invalid_token").into_owned()))?;

    if claims.token_use != expected_use {
        return Err(AppError::Unauthorized(
            t!("auth.token.invalid_use").into_owned(),
        ));
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

    fn claims_with_role(role: i32) -> Claims {
        Claims {
            sub: uuid::Uuid::new_v4(),
            profile_id: uuid::Uuid::new_v4(),
            role,
            token_use: "access".to_string(),
            exp: 0,
            iat: 0,
            nbf: 0,
            aud: DEFAULT_TOKEN_AUDIENCE.to_string(),
            jti: None,
        }
    }

    #[test]
    fn test_claims_has_role() {
        let claims = claims_with_role(1);
        assert!(claims.is_admin());
        assert!(!claims.has_role("operator"));
    }

    #[test]
    fn is_operator_or_higher_is_admin_and_operator() {
        use crate::models::role::{ROLE_ADMIN, ROLE_OPERATOR, ROLE_VIEWER};

        assert!(
            claims_with_role(ROLE_ADMIN.as_i32()).is_operator_or_higher(),
            "the administrator is operator or higher"
        );
        assert!(
            claims_with_role(ROLE_OPERATOR.as_i32()).is_operator_or_higher(),
            "the operator is operator or higher"
        );
        assert!(
            !claims_with_role(ROLE_VIEWER.as_i32()).is_operator_or_higher(),
            "the viewer is below the operator role"
        );
    }

    #[test]
    fn test_bearer_exempt_routes() {
        let routes = bearer_exempt_routes();
        assert!(routes.iter().any(|r| r.pattern == "/api/v1/auth/login"));
        assert!(routes.iter().any(|r| r.pattern == "/api/v1/health"));

        // Cookie-authenticated endpoints must stay reachable without a
        // Bearer token once JwtAuth wraps the /auth scope.
        assert!(routes.iter().any(|r| r.pattern == "/api/v1/auth/session"));
        assert!(routes.iter().any(|r| r.pattern == "/api/v1/auth/refresh"));

        // Public auth flows issued without any credentials.
        assert!(
            routes
                .iter()
                .any(|r| r.pattern == "/api/v1/auth/forgot-password")
        );
        assert!(
            routes
                .iter()
                .any(|r| r.pattern == "/api/v1/auth/magic-link")
        );
        assert!(
            routes
                .iter()
                .any(|r| r.pattern == "/api/v1/auth/magic-link/verify")
        );
    }

    #[test]
    fn create_token_with_kid_sets_header_kid() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

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
            DEFAULT_TOKEN_AUDIENCE,
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

    /// Mints a token by hand so the payload can carry a claim the production
    /// minting does not emit (a missing audience, a foreign audience). Every
    /// other required claim is present, so the only reason to refuse the
    /// token is the audience check under test.
    fn hand_minted_token(secret: &str, kid: &str, aud: Option<&str>) -> String {
        use chrono::Utc;
        use jsonwebtoken::{EncodingKey, Header, encode};

        let now = Utc::now().timestamp();
        let mut payload = serde_json::json!({
            "sub": uuid::Uuid::new_v4().to_string(),
            "profile_id": uuid::Uuid::new_v4().to_string(),
            "role": 1,
            "token_use": "access",
            "exp": now + 3600,
            "iat": now,
            "nbf": now,
        });
        if let Some(audience) = aud {
            payload["aud"] = serde_json::json!(audience);
        }
        let header = Header {
            kid: Some(kid.to_string()),
            ..Default::default()
        };
        encode(
            &header,
            &payload,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("failed to mint the hand made token")
    }

    fn payload_of(token: &str) -> serde_json::Value {
        use base64::Engine;

        let payload = token.split('.').nth(1).expect("three parts");
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .expect("url safe payload");
        serde_json::from_slice(&bytes).expect("json payload")
    }

    #[test]
    fn a_token_without_an_audience_claim_is_rejected() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

        let secret = JwtSecretKey {
            kid: "aud-key".to_string(),
            secret: "audience-secret-value".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };
        let token = hand_minted_token(&secret.secret, &secret.kid, None);

        assert!(
            verify_token_with_secrets(&token, &[secret], "access", DEFAULT_TOKEN_AUDIENCE).is_err(),
            "a token minted without an audience must not be accepted"
        );
    }

    #[test]
    fn a_token_minted_for_another_audience_is_rejected() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

        let secret = JwtSecretKey {
            kid: "aud-key".to_string(),
            secret: "audience-secret-value".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };
        let token = hand_minted_token(&secret.secret, &secret.kid, Some("another-service"));

        assert!(
            verify_token_with_secrets(&token, &[secret], "access", DEFAULT_TOKEN_AUDIENCE).is_err(),
            "a token minted for another audience must not be accepted here"
        );
    }

    #[test]
    fn a_minted_token_carries_the_application_audience() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

        let secret = JwtSecretKey {
            kid: "aud-key".to_string(),
            secret: "audience-secret-value".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };
        let token = create_token_with_kid(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            0,
            &secret.secret,
            3600,
            "access",
            Some(&secret.kid),
            DEFAULT_TOKEN_AUDIENCE,
        )
        .unwrap();

        assert_eq!(
            payload_of(&token).get("aud").and_then(|v| v.as_str()),
            Some("rust-nuxt-boilerplate"),
            "the minted token must name this application as its audience"
        );
        assert_eq!(DEFAULT_TOKEN_AUDIENCE, "rust-nuxt-boilerplate");
    }

    #[test]
    fn a_blank_audience_never_accepts_a_token_with_none() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

        let secret = JwtSecretKey {
            kid: "aud-key".to_string(),
            secret: "audience-secret-value".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };
        // The hole this closes: a token that names the empty audience,
        // verified with a blank one, would match. The verify edge substitutes
        // the default, so the token is refused.
        let token = hand_minted_token(&secret.secret, &secret.kid, Some(""));

        assert!(
            verify_token_with_secrets(&token, &[secret], "access", "").is_err(),
            "a blank audience must fall back to the default, never accept a token of the empty audience"
        );
    }

    #[test]
    fn verify_token_with_secrets_matches_kid() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

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
            DEFAULT_TOKEN_AUDIENCE,
        )
        .unwrap();
        let result =
            verify_token_with_secrets(&token, &secrets, "access", DEFAULT_TOKEN_AUDIENCE).unwrap();
        assert_eq!(result.claims.sub, sub);
        assert_eq!(result.claims.profile_id, profile_id);
        assert_eq!(result.outcome, JwtVerifyOutcome::DirectMatch);
    }

    #[test]
    fn verify_token_with_secrets_falls_back_to_other_secrets() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

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
            DEFAULT_TOKEN_AUDIENCE,
        )
        .unwrap();
        let result =
            verify_token_with_secrets(&token, &secrets, "access", DEFAULT_TOKEN_AUDIENCE).unwrap();
        assert_eq!(result.claims.sub, sub);
        assert_eq!(result.outcome, JwtVerifyOutcome::DirectMatch);
    }

    #[test]
    fn verify_token_with_secrets_rejects_expired_keys() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

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
            DEFAULT_TOKEN_AUDIENCE,
        )
        .unwrap();
        let result = verify_token_with_secrets(&token, &secrets, "access", DEFAULT_TOKEN_AUDIENCE);
        assert!(result.is_err());
    }

    #[test]
    fn verify_token_with_secrets_rejects_wrong_use() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

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
            DEFAULT_TOKEN_AUDIENCE,
        )
        .unwrap();
        let result =
            verify_token_with_secrets(&token, &secrets, "websocket", DEFAULT_TOKEN_AUDIENCE);
        assert!(result.is_err());
    }

    #[test]
    fn verify_token_with_secrets_unknown_kid_falls_back() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

        let sub = uuid::Uuid::new_v4();
        let profile_id = uuid::Uuid::new_v4();

        let secret = JwtSecretKey {
            kid: "real-key".to_string(),
            secret: "real-secret".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };

        let secrets = vec![secret.clone()];

        // Token issued with a kid that doesn't exist in secrets — fallback to all
        let token = create_token_with_kid(
            sub,
            profile_id,
            0,
            &secret.secret,
            3600,
            "access",
            Some("unknown-kid"),
            DEFAULT_TOKEN_AUDIENCE,
        )
        .unwrap();
        let result =
            verify_token_with_secrets(&token, &secrets, "access", DEFAULT_TOKEN_AUDIENCE).unwrap();
        assert_eq!(result.claims.sub, sub);
        assert_eq!(result.outcome, JwtVerifyOutcome::FallbackMatch);
    }

    #[test]
    fn verify_token_with_secrets_no_kid_header_falls_back() {
        use chrono::Utc;

        use crate::config::app_config::JwtSecretKey;

        let sub = uuid::Uuid::new_v4();
        let profile_id = uuid::Uuid::new_v4();

        let secret = JwtSecretKey {
            kid: "key-no-header".to_string(),
            secret: "some-secret".to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: None,
        };

        let secrets = vec![secret.clone()];

        // Token with no kid in header
        let token = create_token_with_kid(
            sub,
            profile_id,
            0,
            &secret.secret,
            3600,
            "access",
            None,
            DEFAULT_TOKEN_AUDIENCE,
        )
        .unwrap();
        let result =
            verify_token_with_secrets(&token, &secrets, "access", DEFAULT_TOKEN_AUDIENCE).unwrap();
        assert_eq!(result.claims.sub, sub);
        assert_eq!(result.outcome, JwtVerifyOutcome::FallbackMatch);
    }
}
