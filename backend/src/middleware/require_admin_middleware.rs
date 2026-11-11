//! Gate of the admin scope.
//!
//! The gate opens the admin domain to the subjects holding a class level
//! allow over at least one resource of the domain. Scoped grants (the self
//! scoped customer role) never open it: they only apply to the instance
//! checks of the handlers.

use std::rc::Rc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::Method;
use actix_web::{Error, HttpMessage};
use futures::future::{LocalBoxFuture, Ready, ready};
use serde_json::json;

use crate::authz::ability::{CurrentAbility, DOMAIN_RESOURCES};

/// The middleware of the admin domain gate.
#[derive(Clone, Default)]
pub struct RequireAdminDomain;

impl RequireAdminDomain {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireAdminDomain
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Transform = RequireAdminDomainMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAdminDomainMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireAdminDomainMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireAdminDomainMiddleware<S>
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
            if req.method() == Method::OPTIONS {
                return svc.call(req).await.map(ServiceResponse::map_into_left_body);
            }

            let Some(ability) = req.extensions().get::<CurrentAbility>().cloned() else {
                let response = unauthorized_response().map_into_right_body();
                return Ok(req.into_response(response));
            };

            if !ability.can_check_domain(&DOMAIN_RESOURCES) {
                let response = forbidden_response().map_into_right_body();
                return Ok(req.into_response(response));
            }

            svc.call(req).await.map(ServiceResponse::map_into_left_body)
        })
    }
}

/// The 401 response of a request without an authenticated subject.
fn unauthorized_response() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Unauthorized().json(json!({
        "error": {
            "code": "UNAUTHORIZED",
            "message": t!("middleware.not_authenticated").into_owned(),
        }
    }))
}

/// The 403 response of a subject without any permission over the admin
/// domain.
fn forbidden_response() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Forbidden().json(json!({
        "error": {
            "code": "FORBIDDEN",
            "message": t!("middleware.admin_domain_required").into_owned(),
        }
    }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, HttpResponse, test, web};

    use super::RequireAdminDomain;
    use crate::middleware::auth::{JwtAuth, JwtAuthConfig, create_token};
    use crate::middleware::csrf_protection::CsrfProtection;
    use crate::repositories::mocks::{mock_app_config, mock_container};
    use crate::repositories::traits::users_trait::RoleGrantRow;
    use crate::repositories::users_repository::MockIUserRepository;

    /// An allow link of an unscoped role.
    fn allow(role: &str, code: &str) -> RoleGrantRow {
        RoleGrantRow {
            role_name: role.to_string(),
            resource_type: None,
            resource_id: None,
            permission_code: Some(code.to_string()),
            effect: Some(0),
        }
    }

    /// An allow link of a self scoped role.
    fn self_scoped(role: &str, code: &str) -> RoleGrantRow {
        RoleGrantRow {
            role_name: role.to_string(),
            resource_type: Some("user".to_string()),
            resource_id: None,
            permission_code: Some(code.to_string()),
            effect: Some(0),
        }
    }

    /// Build the `/admin` middleware stack in the SAME registration order as
    /// `routes::router` (CsrfProtection innermost for its `BoxBody` bound,
    /// RequireAdminDomain before JwtAuth so execution is JwtAuth ->
    /// RequireAdminDomain).
    ///
    /// Regression guard for the Actix reverse-execution trap: if
    /// RequireAdminDomain runs before JwtAuth, the ability is never present in
    /// the request extensions and EVERY admin request fails with 401 even
    /// with a valid token.
    ///
    /// The second argument is the number of grant reads expected: the tokenless
    /// and the preflight requests short-circuit in JwtAuth before the grants
    /// are loaded.
    macro_rules! admin_stack_app {
        ($grants:expr, $db_times:expr) => {{
            let grants = $grants;
            let mut mock_users = MockIUserRepository::new();
            mock_users
                .expect_get_user_grants()
                .times($db_times)
                .returning(move |_| Ok(grants.clone()));
            let mut container = mock_container();
            container.users = Arc::new(mock_users);
            test::init_service(
                App::new().app_data(web::Data::new(container)).service(
                    web::scope("/admin")
                        .wrap(CsrfProtection::new(vec![]))
                        .wrap(RequireAdminDomain::new())
                        .wrap(JwtAuth::new(JwtAuthConfig::new(vec![])))
                        .route(
                            "/ping",
                            web::route().to(|| async { HttpResponse::Ok().finish() }),
                        ),
                ),
            )
            .await
        }};
    }

    fn bearer() -> String {
        let config = mock_app_config();
        create_token(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            1,
            &config.jwt_secret,
            3600,
        )
        .expect("failed to mint the test token")
    }

    #[actix_web::test]
    async fn admin_grants_pass_the_gate() {
        let app = admin_stack_app!(vec![allow("admin", "roles:read")], 1);
        let req = test::TestRequest::get()
            .uri("/admin/ping")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", bearer()),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::OK,
            "a subject with class level domain grants must pass the gate"
        );
    }

    #[actix_web::test]
    async fn viewer_grants_pass_the_gate() {
        let app = admin_stack_app!(
            vec![
                allow("viewer", "users:read"),
                allow("viewer", "profiles:read"),
            ],
            1
        );
        let req = test::TestRequest::get()
            .uri("/admin/ping")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", bearer()),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn customer_cannot_open_admin_scope() {
        let app = admin_stack_app!(
            vec![
                self_scoped("customer", "profiles:read"),
                self_scoped("customer", "profiles:update"),
            ],
            1
        );
        let req = test::TestRequest::get()
            .uri("/admin/ping")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", bearer()),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::FORBIDDEN,
            "a self scoped subject must not open the admin scope"
        );
    }

    #[actix_web::test]
    async fn a_subject_without_grants_is_refused() {
        let app = admin_stack_app!(Vec::new(), 1);
        let req = test::TestRequest::get()
            .uri("/admin/ping")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", bearer()),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn preflight_passes_without_a_token() {
        let app = admin_stack_app!(Vec::new(), 0);
        let req = test::TestRequest::default()
            .method(actix_web::http::Method::OPTIONS)
            .uri("/admin/ping")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn a_missing_token_is_unauthorized() {
        let app = admin_stack_app!(Vec::new(), 0);
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
