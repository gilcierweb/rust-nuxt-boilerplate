use actix_web::{HttpResponse, ResponseError};
use diesel::result::Error as DieselError;
use serde_json::json;

use crate::errors::AppError;

impl From<DieselError> for AppError {
    fn from(error: DieselError) -> Self {
        AppError::Database(error)
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

        HttpResponse::build(self.status_code()).json(json!({
            "error": {
                "code":    self.error_code(),
                "message": self.public_message(),
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{ResponseError, body::to_bytes, http::StatusCode};
    use serde_json::Value;

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
