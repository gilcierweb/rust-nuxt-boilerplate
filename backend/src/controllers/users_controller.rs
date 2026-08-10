use std::sync::Arc;

use actix_web::{HttpResponse, get, web};
use actix_web_grants::authorities::AuthDetails;

use crate::authz::ability::{AbilityAction, AbilityResource, authorize};
use crate::errors::{AppError, AppResult};
use crate::repositories::container::AppContainer;
pub use crate::repositories::traits::users_trait::{AdminUserItem, AdminUserLookupItem};
use crate::security::SecurityService;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

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
pub async fn list_users(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Users, AbilityAction::Read)?;

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
        (status = 403, description = "Missing `users:read` authority"),
        (status = 404, description = "User not found")
    ),
    security(("bearer_auth" = []))
)]
#[get("/users/{id}")]
pub async fn get_user(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    user_id: web::Path<uuid::Uuid>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Users, AbilityAction::Read)?;

    let security = Arc::new(
        SecurityService::from_config(&container.config)
            .map_err(|e| AppError::Internal(e.to_string()))?,
    );

    let item = container
        .users
        .find_by_id_with_profile(&user_id.into_inner(), security)
        .await
        .map_err(|error| map_repo_error(error, "User"))?;

    Ok(HttpResponse::Ok().json(item))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_users).service(get_user);
}

#[cfg(test)]
pub fn test_config(cfg: &mut web::ServiceConfig) {
    use crate::middleware::test_authorities::TestAuthorities;

    cfg.service(web::scope("/admin").wrap(TestAuthorities).configure(config));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use serde_json::Value;

    use super::test_config;
    use crate::repositories::mocks::mock_container;
    use crate::repositories::users_repository::MockIUserRepository;

    #[actix_web::test]
    async fn list_users_returns_forbidden_for_customer_without_read_authority() {
        let container = mock_container();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(test_config),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/users")
            .insert_header(("x-test-authorities", "ROLE_CUSTOMER"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn list_users_returns_ok_for_admin_with_read_authority() {
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

        let req = test::TestRequest::get()
            .uri("/admin/users")
            .insert_header(("x-test-authorities", "ROLE_ADMIN,users:read"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"].as_array().map(|items| items.len()), Some(0));
    }
}
