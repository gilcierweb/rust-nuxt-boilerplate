use std::collections::HashMap;

use actix_web::{HttpResponse, get, web};
use actix_web_grants::authorities::AuthDetails;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    authz::ability::{AbilityAction, AbilityResource, authorize},
    errors::{AppError, AppResult},
    repositories::container::AppContainer,
    security::SecurityService,
};

#[derive(Debug, Serialize)]
struct AdminUserLookupItem {
    id: Uuid,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
    full_name: Option<String>,
    nickname: Option<String>,
}

#[get("/users")]
pub async fn list_users(
    details: AuthDetails,
    container: web::Data<AppContainer>,
) -> AppResult<HttpResponse> {
    authorize(&details, AbilityResource::Users, AbilityAction::Read)?;

    let users = container.users.all().await.map_err(AppError::Database)?;
    let profiles = container.profiles.all().await.map_err(AppError::Database)?;
    let security = SecurityService::from_config(container.config.as_ref())?;

    let profiles_by_user_id = profiles
        .into_iter()
        .map(|profile| (profile.user_id, profile))
        .collect::<HashMap<Uuid, _>>();

    let mut items = Vec::with_capacity(users.len());

    for user in users {
        let email = security.decrypt_user_email(&user)?;
        let profile = profiles_by_user_id.get(&user.id);

        items.push(AdminUserLookupItem {
            id: user.id,
            email,
            first_name: profile.and_then(|p| p.first_name.clone()),
            last_name: profile.and_then(|p| p.last_name.clone()),
            full_name: profile.and_then(|p| p.full_name.clone()),
            nickname: profile.and_then(|p| p.nickname.clone()),
        });
    }

    Ok(HttpResponse::Ok().json(items))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_users);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;

    use actix_web::{App, body::to_bytes, dev::ServiceRequest, http::StatusCode, test, web};
    use serde_json::Value;

    use crate::repositories::profiles_repository::MockIProfileRepository;
    use crate::repositories::test_utils::mocks::mock_container;
    use crate::repositories::users_repository::MockIUserRepository;

    use super::config;

    async fn test_extract_authorities(
        req: &ServiceRequest,
    ) -> Result<HashSet<String>, actix_web::Error> {
        let authorities = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.trim())
            .map(|value| {
                if value == "Bearer customer" {
                    HashSet::from(["ROLE_CUSTOMER".to_string()])
                } else if value == "Bearer admin" {
                    HashSet::from(["ROLE_ADMIN".to_string(), "users:read".to_string()])
                } else {
                    HashSet::new()
                }
            })
            .unwrap_or_default();

        Ok(authorities)
    }

    #[actix_web::test]
    async fn list_users_returns_forbidden_for_customer_without_read_authority() {
        let container = mock_container();
        let app = test::init_service(
            App::new().app_data(web::Data::new(container)).service(
                web::scope("/admin")
                    .wrap(actix_web_grants::GrantsMiddleware::with_extractor(
                        test_extract_authorities,
                    ))
                    .configure(config),
            ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/users")
            .insert_header((actix_web::http::header::AUTHORIZATION, "Bearer customer"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn list_users_returns_ok_for_admin_with_read_authority() {
        let mut container = mock_container();

        let mut users_repo = MockIUserRepository::new();
        users_repo
            .expect_all()
            .times(1)
            .returning(|| Ok(Vec::new()));
        container.users = Arc::new(users_repo);

        let mut profiles_repo = MockIProfileRepository::new();
        profiles_repo
            .expect_all()
            .times(1)
            .returning(|| Ok(Vec::new()));
        container.profiles = Arc::new(profiles_repo);

        let app = test::init_service(
            App::new().app_data(web::Data::new(container)).service(
                web::scope("/admin")
                    .wrap(actix_web_grants::GrantsMiddleware::with_extractor(
                        test_extract_authorities,
                    ))
                    .configure(config),
            ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin/users")
            .insert_header((actix_web::http::header::AUTHORIZATION, "Bearer admin"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = to_bytes(resp.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.as_array().map(|items| items.len()), Some(0));
    }
}
