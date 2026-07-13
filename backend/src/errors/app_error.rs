use derive_more::derive::Display;
use diesel::result::Error as DieselError;
use std::error::Error;

#[derive(Debug, Display)]
pub enum AppError {
    #[display("{_0}")]
    NotFound(String),

    #[display("{_0}")]
    BadRequest(String),

    #[display("{_0}")]
    Unauthorized(String),

    #[display("{_0}")]
    Forbidden(String),

    #[display("{_0}")]
    Conflict(String),

    #[display("{_0}")]
    Validation(String),

    #[display("{_0}")]
    Database(DieselError),

    #[display("{_0}")]
    Internal(String),

    #[display("{_0}")]
    #[allow(dead_code)]
    TooManyRequests(String),

    #[display("{_0}")]
    #[allow(dead_code)]
    RateLimited(String),
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Database(e) => Some(e),
            _ => None,
        }
    }
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Database(_) => "DB_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::TooManyRequests(_) => "TOO_MANY_REQUESTS",
            AppError::RateLimited(_) => "RATE_LIMITED",
        }
    }

    pub fn public_message(&self) -> String {
        match self {
            AppError::Database(_) => t!("errors.database").into_owned(),
            AppError::Internal(_) => t!("errors.internal").into_owned(),
            AppError::Validation(message) => {
                if message.trim().is_empty() {
                    t!("errors.validation").into_owned()
                } else {
                    message.clone()
                }
            }
            AppError::BadRequest(message) => {
                if message.trim().is_empty() {
                    t!("errors.bad_request").into_owned()
                } else {
                    message.clone()
                }
            }
            AppError::Unauthorized(message) => {
                if message.trim().is_empty() {
                    t!("errors.unauthorized").into_owned()
                } else {
                    message.clone()
                }
            }
            AppError::Forbidden(message) => {
                if message.trim().is_empty() {
                    t!("errors.forbidden").into_owned()
                } else {
                    message.clone()
                }
            }
            AppError::NotFound(resource) => {
                if resource.trim().is_empty() {
                    t!("errors.not_found", resource = "Resource").into_owned()
                } else {
                    t!("errors.not_found", resource = resource.as_str()).into_owned()
                }
            }
            AppError::Conflict(message) => {
                if message.trim().is_empty() {
                    t!("errors.conflict").into_owned()
                } else {
                    message.clone()
                }
            }
            AppError::TooManyRequests(_) => t!("errors.too_many_requests").into_owned(),
            AppError::RateLimited(_) => t!("errors.rate_limited").into_owned(),
        }
    }

    pub fn should_log_internal_details(&self) -> bool {
        matches!(self, AppError::Database(_) | AppError::Internal(_))
    }
}
