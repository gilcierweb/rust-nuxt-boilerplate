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
