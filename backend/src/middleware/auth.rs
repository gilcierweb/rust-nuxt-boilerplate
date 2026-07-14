use actix_web::{FromRequest, http::header::AUTHORIZATION};
use std::future::{Ready, ready};

use crate::{
    AppState,
    errors::{AppError, AppResult},
    models::role::ROLE_ADMIN,
};

/// JWT Claims structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Check if the user has a specific role by name.
    /// Maps role names to their i32 values from the Role enum.
    pub fn has_role(&self, role: &str) -> bool {
        match role.to_ascii_lowercase().as_str() {
            "admin" => self.role == ROLE_ADMIN.as_i32(),
            "operator" | "moderator" | "support" | "creator" | "agency" => {
                self.role == crate::models::role::ROLE_OPERATOR.as_i32()
            }
            "viewer" | "fan" => self.role == crate::models::role::ROLE_VIEWER.as_i32(),
            _ => false,
        }
    }

    /// Check if the user has admin role
    pub fn is_admin(&self) -> bool {
        self.role == ROLE_ADMIN.as_i32()
    }

    /// Get profile_id as Uuid result
    pub fn profile_id(&self) -> Result<uuid::Uuid, AppError> {
        Ok(self.profile_id)
    }
}

const ACCESS_TOKEN_USE: &str = "access";
const WEBSOCKET_TOKEN_USE: &str = "ws";

fn default_token_use() -> String {
    ACCESS_TOKEN_USE.to_string()
}

/// Extractor for authenticated user claims
/// Reads and validates JWT token directly from Authorization header
pub struct AuthUser {
    claims: Claims,
}

impl AuthUser {
    pub fn claims(&self) -> &Claims {
        &self.claims
    }
}

impl FromRequest for AuthUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        // Extract Bearer token from Authorization header
        let token = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        match token {
            None => ready(Err(AppError::Unauthorized(
                "Missing or invalid Authorization header".to_string(),
            ))),
            Some(t) => {
                // Get JWT secret from AppState
                let state = req.app_data::<actix_web::web::Data<AppState>>();
                let secret = state
                    .as_ref()
                    .map(|s| s.config.jwt_secret.clone())
                    .unwrap_or_default();

                match verify_token(t, &secret) {
                    Ok(claims) => ready(Ok(AuthUser { claims })),
                    Err(e) => ready(Err(e)),
                }
            }
        }
    }
}

/// Create a JWT token for a user
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

/// Create a short-lived JWT for WebSocket handshakes only.
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
    use chrono::Utc;
    use jsonwebtoken::{EncodingKey, Header, encode};

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

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token creation failed: {}", e)))
}

/// Verify a JWT token
pub fn verify_token(token: &str, jwt_secret: &str) -> AppResult<Claims> {
    verify_token_for_use(token, jwt_secret, ACCESS_TOKEN_USE)
}

/// Verify a WebSocket-only JWT token.
pub fn verify_ws_token(token: &str, jwt_secret: &str) -> AppResult<Claims> {
    verify_token_for_use(token, jwt_secret, WEBSOCKET_TOKEN_USE)
}

fn verify_token_for_use(token: &str, jwt_secret: &str, expected_use: &str) -> AppResult<Claims> {
    use jsonwebtoken::{DecodingKey, Validation, decode};

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    if claims.token_use != expected_use {
        return Err(AppError::Unauthorized("Invalid token".to_string()));
    }

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::{create_token, create_ws_token, verify_token, verify_ws_token};

    #[test]
    fn access_tokens_are_rejected_by_websocket_verifier() {
        let token =
            create_token(uuid::Uuid::new_v4(), uuid::Uuid::new_v4(), 1, "secret", 60).unwrap();

        assert!(verify_ws_token(&token, "secret").is_err());
    }

    #[test]
    fn websocket_tokens_are_rejected_by_access_verifier() {
        let token =
            create_ws_token(uuid::Uuid::new_v4(), uuid::Uuid::new_v4(), 1, "secret", 60).unwrap();

        assert!(verify_token(&token, "secret").is_err());
    }
}
