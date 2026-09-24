use std::rc::Rc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::{Error, HttpRequest};
use futures::future::{LocalBoxFuture, Ready, ready};

use crate::middleware::auth_middleware::extract_claims;
use crate::models::role::ROLE_ADMIN;
use crate::repositories::container::AppContainer;

#[derive(Clone, Default)]
pub struct RequireAdmin;

impl RequireAdmin {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireAdmin
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Transform = RequireAdminMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAdminMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireAdminMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireAdminMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let method = req.method().clone();

            if method == actix_web::http::Method::OPTIONS {
                return svc.call(req).await.map(ServiceResponse::map_into_left_body);
            }

            // Convert ServiceRequest to HttpRequest for extract_claims
            let http_req: &HttpRequest = req.request();

            let claims = match extract_claims(http_req) {
                Ok(c) => c,
                Err(_) => {
                    let response =
                        actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
                            "error": {
                                "code": "UNAUTHORIZED",
                                "message": "Authentication required"
                            }
                        }));
                    return Err(actix_web::error::InternalError::from_response(
                        "unauthorized",
                        response,
                    )
                    .into());
                },
            };

            // Check if user has ROLE_ADMIN via JWT claim
            let has_admin_role = claims.role == ROLE_ADMIN.as_i32();

            // If not in JWT, check database roles (for backward compatibility)
            let is_admin = if has_admin_role {
                true
            } else {
                let container = req.app_data::<actix_web::web::Data<AppContainer>>();
                if let Some(container) = container {
                    match container.users.get_user_roles(&claims.sub).await {
                        Ok(roles) => roles.iter().any(|role| role.eq_ignore_ascii_case("admin")),
                        Err(_) => false,
                    }
                } else {
                    false
                }
            };

            if !is_admin {
                let response = actix_web::HttpResponse::Forbidden().json(serde_json::json!({
                    "error": {
                        "code": "FORBIDDEN",
                        "message": "Admin role required"
                    }
                }));
                return Err(actix_web::error::InternalError::from_response(
                    "admin_required",
                    response,
                )
                .into());
            }

            svc.call(req).await.map(ServiceResponse::map_into_left_body)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, HttpResponse, test, web};

    use super::RequireAdmin;
    use crate::middleware::auth::{JwtAuth, JwtAuthConfig, create_token};
    use crate::middleware::csrf_protection::CsrfProtection;
    use crate::models::role::ROLE_ADMIN;
    use crate::repositories::mocks::{mock_app_config, mock_container};
    use crate::repositories::users_repository::MockIUserRepository;

    /// Build the `/admin` middleware stack in the SAME registration order as
    /// `routes::router` (CsrfProtection innermost for its `BoxBody` bound,
    /// RequireAdmin before JwtAuth so execution is JwtAuth -> RequireAdmin).
    /// Regression guard for the Actix reverse-execution trap: if RequireAdmin
    /// runs before JwtAuth, `Claims` are never present in request extensions
    /// and EVERY admin request fails with 401 `UNAUTHORIZED` even with a
    /// valid admin token.
    ///
    /// `$db_times` bounds how often the user lookups may run: JwtAuth
    /// resolves authorities (roles + permissions) once per authenticated
    /// request, while RequireAdmin skips the DB when the JWT already
    /// carries the admin role.
    macro_rules! admin_stack_app {
        ($db_times:expr) => {{
            let mut mock_users = MockIUserRepository::new();
            mock_users
                .expect_get_user_roles()
                .times($db_times)
                .returning(|_| Ok(vec!["admin".to_string()]));
            mock_users
                .expect_get_user_permissions()
                .times($db_times)
                .returning(|_| Ok(Vec::new()));
            let mut container = mock_container();
            container.users = Arc::new(mock_users);
            test::init_service(
                App::new().app_data(web::Data::new(container)).service(
                    web::scope("/admin")
                        .wrap(CsrfProtection::new(vec![]))
                        .wrap(RequireAdmin::new())
                        .wrap(JwtAuth::new(JwtAuthConfig::new(vec![])))
                        .route(
                            "/ping",
                            web::get().to(|| async { HttpResponse::Ok().finish() }),
                        ),
                ),
            )
            .await
        }};
    }

    fn admin_bearer_token() -> String {
        let config = mock_app_config();
        create_token(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            ROLE_ADMIN.as_i32(),
            &config.jwt_secret,
            3600,
        )
        .expect("failed to mint admin test token")
    }

    #[actix_web::test]
    async fn admin_stack_allows_valid_admin_token() {
        let app = admin_stack_app!(1);
        let req = test::TestRequest::get()
            .uri("/admin/ping")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", admin_bearer_token()),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::OK,
            "valid admin Bearer token must pass JwtAuth + RequireAdmin"
        );
    }

    #[actix_web::test]
    async fn admin_stack_rejects_missing_token() {
        // Without a Bearer token JwtAuth rejects with a plain-text 401
        // before RequireAdmin runs, so only the status is asserted here.
        let app = admin_stack_app!(0);
        let req = test::TestRequest::get().uri("/admin/ping").to_request();
        let err = test::try_call_service(&app, req)
            .await
            .expect_err("expected the stack to reject a tokenless request");

        assert_eq!(
            err.error_response().status(),
            actix_web::http::StatusCode::UNAUTHORIZED
        );
    }
}
