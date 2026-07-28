use actix_web::{HttpResponse, get, web};
use actix_web_grants::authorities::AuthDetails;

use crate::{
    authz::ability::{AbilityAction, AbilityResource, authorize},
    errors::{AppError, AppResult},
    repositories::container::AppContainer,
    utils::pagination::PaginationParams,
};

pub use crate::repositories::traits::users_trait::AdminUserLookupItem;

#[get("/users")]
pub async fn list_users(
    details: AuthDetails,
    container: web::Data<AppContainer>,
    pagination: web::Query<PaginationParams>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Users, AbilityAction::Read)?;

    let pagination = pagination.into_inner().validated();
    let response = container
        .users
        .list_paginated(&pagination)
        .await
        .map_err(AppError::Database)?;

    Ok(HttpResponse::Ok().json(response))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_users);
}

#[cfg(test)]
pub fn test_config(cfg: &mut web::ServiceConfig) {
    use crate::middleware::test_authorities::TestAuthorities;

    cfg.service(web::scope("/admin").wrap(TestAuthorities).configure(config));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, body::to_bytes, http::StatusCode, test, web};
    use serde_json::Value;

    use crate::repositories::test_utils::mocks::mock_container;
    use crate::repositories::users_repository::MockIUserRepository;

    use super::test_config;

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
            .returning(|_| Ok(PaginatedResponse::new(Vec::new(), 0, 1, 20)));
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
