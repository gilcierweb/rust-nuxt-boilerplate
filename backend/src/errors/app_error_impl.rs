use actix_web::{HttpResponse, ResponseError};
use diesel::result::Error as DieselError;
use jsonwebtoken::errors::Error as JwtError;
use serde_json::json;

use crate::errors::AppError;
use crate::middleware::locale::current_request_locale;

impl From<DieselError> for AppError {
    fn from(error: DieselError) -> Self {
        AppError::Database(error)
    }
}

impl From<JwtError> for AppError {
    fn from(error: JwtError) -> Self {
        let msg = current_request_locale()
            .map(|rl| {
                let mut args = std::collections::HashMap::new();
                args.insert("error".to_string(), error.to_string());
                rl.t_blocking("errors.token_error", Some(&args))
            })
            .unwrap_or_else(|| {
                // Fallback to global rust_i18n (set_locale at startup) if the
                // thread-local wasn't populated (background tasks, tests).
                t!("errors.token_error", error = error.to_string()).into_owned()
            });
        AppError::Unauthorized(msg)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => actix_web::http::StatusCode::FORBIDDEN,
            AppError::Conflict(_) => actix_web::http::StatusCode::CONFLICT,
            AppError::Validation(_) => actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Database(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TooManyRequests(_) => actix_web::http::StatusCode::TOO_MANY_REQUESTS,
            AppError::RateLimited(_) => actix_web::http::StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if self.should_log_internal_details() {
            tracing::error!(
                error_code = self.error_code(),
                "Request failed with internal server error"
            );
        }

        // Try to read the per-request locale via the thread-local the
        // middleware populated. `ResponseError::error_response` does not get
        // the `HttpRequest`, so we rely on the thread-local here. If the
        // thread-local is unset (e.g., tests, startup), fall back to the
        // global `rust_i18n::set_locale` value.
        let message = current_request_locale()
            .map(|rl| {
                let key = self.public_message_key();
                let mut args = std::collections::HashMap::new();
                if matches!(self, AppError::NotFound(_)) {
                    args.insert(
                        "resource".to_string(),
                        if let AppError::NotFound(resource) = self {
                            if resource.trim().is_empty() {
                                "Resource".to_string()
                            } else {
                                resource.clone()
                            }
                        } else {
                            "Resource".to_string()
                        },
                    );
                }
                rl.t_blocking(&key, Some(&args))
            })
            .unwrap_or_else(|| self.public_message());

        HttpResponse::build(self.status_code()).json(json!({
            "error": {
                "code":    self.error_code(),
                "message": message,
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use actix_web::ResponseError;
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use serde_json::Value;

    use super::*;

    #[actix_rt::test]
    async fn internal_errors_are_sanitized_in_http_responses() {
        let response = AppError::Internal("sensitive internal detail".to_string()).error_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["code"], "INTERNAL_ERROR");
        assert_eq!(json["error"]["message"], "An internal error occurred");
    }
}
