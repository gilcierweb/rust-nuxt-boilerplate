use std::sync::Arc;

use actix_web::{HttpResponse, get, put, web};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::authz::ability::{AbilityAction, AbilityResource, CurrentAbility};
use crate::errors::{AppError, AppResult};
use crate::repositories::container::AppContainer;
pub use crate::repositories::traits::users_trait::{AdminUserItem, AdminUserLookupItem};
use crate::security::SecurityService;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::validation::first_validation_error_message;

fn map_repo_error(error: diesel::result::Error, entity: &str) -> AppError {
    match error {
        diesel::result::Error::NotFound => AppError::NotFound(entity.to_string()),
        other => AppError::Database(other),
    }
}

/// List users (admin view) with pagination and optional sorting.
///
/// Requires the `users:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    tag = "users",
    params(
        ("page" = Option<i64>, Query, description = "Page number (1-based)"),
        ("per_page" = Option<i64>, Query, description = "Items per page (max 100)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by"),
        ("sort_dir" = Option<String>, Query, description = "Sort direction: `asc` or `desc`")
    ),
    responses(
        (status = 200, description = "Paginated list of users", body = PaginatedResponse<AdminUserLookupItem>),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `users:read` authority")
    ),
    security(("bearer_auth" = []))
)]
#[get("/users")]
pub async fn all_users(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::Users)?;

    let security = Arc::new(
        SecurityService::from_config(&container.config)
            .map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let pagination = pagination.into_inner().validated();
    let response = container
        .users
        .list_paginated(&pagination, security)
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(response))
}

/// Fetch a single user by UUID (admin view).
///
/// Requires the `users:read` authority.
/// Non-admin users can only access their own record.
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/{id}",
    tag = "users",
    params(
        ("id" = uuid::Uuid, Path, description = "User identifier")
    ),
    responses(
        (status = 200, description = "User found", body = AdminUserItem),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `users:read` authority or access denied"),
        (status = 404, description = "User not found")
    ),
    security(("bearer_auth" = []))
)]
#[get("/users/{id}")]
pub async fn find_user(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    user_id: web::Path<uuid::Uuid>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::Users)?;

    // Instance check: a scoped grant applies only inside its scope, so a
    // subject restricted to itself can never read another record.
    let tarfind_user_id = user_id.into_inner();
    ability.authorize_subject(AbilityAction::Read, AbilityResource::Users, tarfind_user_id)?;

    let security = Arc::new(
        SecurityService::from_config(&container.config)
            .map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let item = container
        .users
        .find_by_id_with_profile(&tarfind_user_id, security)
        .await
        .map_err(|error| map_repo_error(error, "User"))?;

    Ok(HttpResponse::Ok().json(item))
}

/// The replacement of every role assignment of a user.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UserRolesRequest {
    #[validate(length(max = 20, message = "admin.users.validation.role_ids_limit"))]
    role_ids: Vec<Uuid>,
}

/// Refuses a role id the catalog does not hold.
///
/// An unknown id is a client mistake, not a missing record: the answer is the
/// same 422 the validation errors use.
async fn validate_role_ids(container: &AppContainer, role_ids: &[Uuid]) -> AppResult<()> {
    for role_id in role_ids {
        let found = container
            .roles
            .find(role_id)
            .await
            .map_err(|error| match error {
                diesel::result::Error::NotFound => {
                    AppError::Validation(t!("admin.users.validation.role_unknown").into_owned())
                },
                other => AppError::Database(other),
            })?;
        let _ = found;
    }

    Ok(())
}

/// The roles of a user, as the admin interface reads them back.
///
/// Requires the `users:read` authority.
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/{id}/roles",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    responses(
        (status = 200, description = "Roles of the user", body = UserRolesResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `users:read` authority"),
        (status = 404, description = "User not found")
    ),
    security(("bearer_auth" = []))
)]
#[get("/users/{id}/roles")]
pub async fn find_user_roles(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Read, AbilityResource::Users)?;
    let user_id = id.into_inner();
    container
        .users
        .find(&user_id)
        .await
        .map_err(|error| AppError::from_diesel(error, "User"))?;

    let assignments = container
        .users
        .find_user_role_assignments(&user_id)
        .await
        .map_err(AppError::Database)?;

    let roles: Vec<RoleSummary> = assignments
        .into_iter()
        .map(|(id, name)| RoleSummary { id, name })
        .collect();
    Ok(HttpResponse::Ok().json(UserRolesResponse { roles }))
}

/// Replaces every role assignment of a user in one transaction.
///
/// Requires the `users:update` authority. Refuses an unknown role id, a role
/// granting authorities the actor does not hold (escalation), and the removal
/// of the last administrator holder or of the last assignment of the
/// administrator role.
#[utoipa::path(
    put,
    path = "/api/v1/admin/users/{id}/roles",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "User identifier")
    ),
    request_body = UserRolesRequest,
    responses(
        (status = 200, description = "Roles replaced", body = UserRolesResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Missing `users:update` authority"),
        (status = 404, description = "User not found"),
        (status = 422, description = "Unknown role, escalation, or an administrator lockout")
    ),
    security(("bearer_auth" = []))
)]
#[put("/users/{id}/roles")]
pub async fn update_user_roles(
    ability: CurrentAbility,
    container: web::Data<AppContainer>,
    id: web::Path<Uuid>,
    body: web::Json<UserRolesRequest>,
) -> AppResult<HttpResponse> {
    ability.authorize(AbilityAction::Update, AbilityResource::Users)?;
    let payload = body.into_inner();
    payload
        .validate()
        .map_err(|error| AppError::Validation(first_validation_error_message(&error)))?;

    let user_id = id.into_inner();
    container
        .users
        .find(&user_id)
        .await
        .map_err(|error| AppError::from_diesel(error, "User"))?;

    // Every role of the request must exist: an unknown id is a client mistake.
    validate_role_ids(&container, &payload.role_ids).await?;

    // Escalation: each assigned role must stay inside the actor's own authority.
    let actor_allow = ability.permissions().can;
    for role_id in &payload.role_ids {
        let role_allow = container
            .users
            .role_allow_codes(role_id)
            .await
            .map_err(AppError::Database)?;
        crate::authz::guards::ensure_no_escalation(&actor_allow, &role_allow)?;
    }

    // Lockout guards, computed on the state after the change.
    let admin_role_id = container
        .roles
        .find_by_name("admin")
        .await
        .map_err(AppError::Database)?
        .map(|role| role.id);
    let admin_codes: Vec<String> = crate::authz::guards::ADMINISTRATOR_CODES
        .iter()
        .map(|code| (*code).to_string())
        .collect();

    let holders_before = container
        .users
        .user_ids_holding_all_codes(&admin_codes)
        .await
        .map_err(AppError::Database)?;
    let keeps_administrator_codes = payload
        .role_ids
        .iter()
        .any(|role_id| admin_role_id.is_some_and(|admin| admin == *role_id));
    let holders_after: Vec<Uuid> = if keeps_administrator_codes {
        let mut holders: Vec<Uuid> = holders_before
            .iter()
            .copied()
            .filter(|holder| *holder != user_id)
            .collect();
        holders.push(user_id);
        holders
    } else {
        holders_before
            .iter()
            .copied()
            .filter(|holder| *holder != user_id)
            .collect()
    };
    crate::authz::guards::ensure_assignment_keeps_administrator(&holders_before, &holders_after)?;

    // The literal admin role cannot lose its last assignment.
    if let Some(admin) = admin_role_id {
        let held_admin_before = container
            .users
            .find_user_role_assignments(&user_id)
            .await
            .map_err(AppError::Database)?
            .iter()
            .any(|(role_id, _)| *role_id == admin);
        let holds_admin_after = payload.role_ids.contains(&admin);
        if held_admin_before && !holds_admin_after {
            let remaining = container
                .users
                .find_user_role_assignments(&user_id)
                .await
                .map_err(AppError::Database)?
                .len();
            if remaining <= 1 {
                return Err(AppError::Validation(
                    t!("admin.users.guard.last_admin_role_assignment").into_owned(),
                ));
            }
        }
    }

    container
        .users
        .set_user_roles(&user_id, &payload.role_ids)
        .await
        .map_err(AppError::Database)?;

    // The ability of the user changed on the next request: their cached grants
    // have to go.
    crate::controllers::auth_controller::invalidate_user_roles_cache(&container, &user_id).await;

    let assignments = container
        .users
        .find_user_role_assignments(&user_id)
        .await
        .map_err(AppError::Database)?;
    let roles: Vec<RoleSummary> = assignments
        .into_iter()
        .map(|(id, name)| RoleSummary { id, name })
        .collect();
    Ok(HttpResponse::Ok().json(UserRolesResponse { roles }))
}

/// A role as the admin interface lists it inside a user payload.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RoleSummary {
    id: Uuid,
    name: String,
}

/// The roles assigned to a user.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserRolesResponse {
    roles: Vec<RoleSummary>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(all_users)
        .service(find_user)
        .service(find_user_roles)
        .service(update_user_roles);
}

#[cfg(test)]
pub fn test_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/admin").configure(config));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use serde_json::Value;

    use actix_web::HttpMessage;

    use super::test_config;
    use crate::authz::test_helpers::ability_from_codes;
    use crate::repositories::mocks::mock_container;
    use crate::repositories::users_repository::MockIUserRepository;

    #[actix_web::test]
    async fn all_users_returns_forbidden_for_customer_without_read_authority() {
        let container = mock_container();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get().uri("/admin/users").to_request();
        // An authenticated subject without grants: the request carries an
        // empty ability, so the handler refuses with 403.
        req.extensions_mut()
            .insert(ability_from_codes(uuid::Uuid::new_v4(), &[]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn all_users_returns_ok_for_admin_with_read_authority() {
        use crate::utils::pagination::PaginatedResponse;

        let mut container = mock_container();

        let mut users_repo = MockIUserRepository::new();
        users_repo
            .expect_list_paginated()
            .times(1)
            .returning(|_, _| Ok(PaginatedResponse::new(Vec::new(), 0, 1, 20)));
        container.users = Arc::new(users_repo);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get().uri("/admin/users").to_request();
        req.extensions_mut()
            .insert(ability_from_codes(uuid::Uuid::new_v4(), &["users:read"]));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"].as_array().map(|items| items.len()), Some(0));
    }
}
