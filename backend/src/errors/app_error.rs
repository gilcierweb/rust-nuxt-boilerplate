use std::error::Error;

use derive_more::derive::Display;
use diesel::result::Error as DieselError;

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
    /// 404 with no specific resource, used by the catch-all handler.
    /// The empty resource makes the i18n layer fall back to the generic
    /// "resource not found" message (see `public_message`).
    pub fn not_found() -> Self {
        AppError::NotFound(String::new())
    }

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

    /// Returns the literal message to expose in the HTTP response. Uses the
    /// global `rust_i18n` (whose locale was set at startup) — used when no
    /// per-request locale is available.
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
            },
            AppError::BadRequest(message) => {
                if message.trim().is_empty() {
                    t!("errors.bad_request").into_owned()
                } else {
                    message.clone()
                }
            },
            AppError::Unauthorized(message) => {
                if message.trim().is_empty() {
                    t!("errors.unauthorized").into_owned()
                } else {
                    message.clone()
                }
            },
            AppError::Forbidden(message) => {
                if message.trim().is_empty() {
                    t!("errors.forbidden").into_owned()
                } else {
                    message.clone()
                }
            },
            AppError::NotFound(resource) => {
                if resource.trim().is_empty() {
                    t!("errors.not_found", resource = "Resource").into_owned()
                } else {
                    t!("errors.not_found", resource = resource.as_str()).into_owned()
                }
            },
            AppError::Conflict(message) => {
                if message.trim().is_empty() {
                    t!("errors.conflict").into_owned()
                } else {
                    message.clone()
                }
            },
            AppError::TooManyRequests(_) => t!("errors.too_many_requests").into_owned(),
            AppError::RateLimited(_) => t!("errors.rate_limited").into_owned(),
        }
    }

    /// Resolve the message *key* + *pattern args* for this error, suitable
    /// for passing to `RequestLocale::t` so the per-request locale can render
    /// the correct translation. Returns `(key, args)`.
    ///
    /// For errors that carry an explicit user-provided message (e.g.
    /// `Validation`, `BadRequest`, `Conflict`), the message is returned
    /// verbatim via the empty key sentinel `""` so callers can short-circuit
    /// translation.
    pub fn public_message_key(&self) -> String {
        match self {
            AppError::Database(_) => "errors.database".to_string(),
            AppError::Internal(_) => "errors.internal".to_string(),
            AppError::Validation(message) => {
                if message.trim().is_empty() {
                    "errors.validation".to_string()
                } else {
                    message.clone()
                }
            },
            AppError::BadRequest(message) => {
                if message.trim().is_empty() {
                    "errors.bad_request".to_string()
                } else {
                    message.clone()
                }
            },
            AppError::Unauthorized(message) => {
                if message.trim().is_empty() {
                    "errors.unauthorized".to_string()
                } else {
                    message.clone()
                }
            },
            AppError::Forbidden(message) => {
                if message.trim().is_empty() {
                    "errors.forbidden".to_string()
                } else {
                    message.clone()
                }
            },
            AppError::NotFound(resource) => {
                if resource.trim().is_empty() {
                    "errors.not_found".to_string()
                } else {
                    resource.clone()
                }
            },
            AppError::Conflict(message) => {
                if message.trim().is_empty() {
                    "errors.conflict".to_string()
                } else {
                    message.clone()
                }
            },
            AppError::TooManyRequests(_) => "errors.too_many_requests".to_string(),
            AppError::RateLimited(_) => "errors.rate_limited".to_string(),
        }
    }

    pub fn should_log_internal_details(&self) -> bool {
        matches!(self, AppError::Database(_) | AppError::Internal(_))
    }

    /// Convert a Diesel query error to an `AppError`, mapping `NotFound` to
    /// `AppError::NotFound(entity)` and everything else to `AppError::Database`.
    ///
    /// Use this in controllers instead of duplicating the match pattern:
    /// ```rust
    /// .map_err(|e| AppError::from_diesel(e, "Role"))?;
    /// ```
    pub fn from_diesel(error: DieselError, entity: &str) -> Self {
        match error {
            DieselError::NotFound => AppError::NotFound(entity.to_string()),
            other => AppError::Database(other),
        }
    }
}
