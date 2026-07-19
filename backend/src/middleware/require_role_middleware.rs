#![allow(dead_code)]

use crate::{
    errors::{AppError, AppResult},
    middleware::{auth::AuthUser, auth_middleware::extract_claims},
    models::role::ROLE_ADMIN,
    repositories::container::AppContainer,
};
use actix_web::HttpRequest;

/// Guard: ensure authenticated user has a specific role.
/// Usage in handlers: `require_role(&req, "creator")?;`
pub fn require_role(req: &HttpRequest, role: &str) -> Result<(), AppError> {
    let claims = extract_claims(req)?;
    if claims.has_role(role) || claims.is_admin() {
        Ok(())
    } else {
        Err(AppError::Forbidden(t!(
            "middleware.role_required",
            role = role
        )
        .into_owned()))
    }
}

/// Guard: ensure the authenticated user IS the resource owner or an admin.
pub fn require_owner_or_admin(
    req: &HttpRequest,
    owner_profile_id: uuid::Uuid,
) -> Result<(), AppError> {
    let claims = extract_claims(req)?;
    let requester = claims.profile_id()?;
    if requester == owner_profile_id || claims.is_admin() {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            t!("middleware.access_denied").into_owned(),
        ))
    }
}

/// Centralized guard for admin-only endpoints.
/// Accepts explicit admin claim OR authoritative admin role from database.
pub async fn ensure_admin(user: &AuthUser, container: &AppContainer) -> AppResult<()> {
    if user.claims().role == ROLE_ADMIN.as_i32() {
        return Ok(());
    }

    let roles = container
        .users
        .get_user_roles(&user.claims().sub)
        .await
        .map_err(AppError::Database)?;

    if roles.iter().any(|role| role.eq_ignore_ascii_case("admin")) {
        Ok(())
    } else {
        Err(AppError::Forbidden(t!("middleware.admin_required").into_owned()))
    }
}
