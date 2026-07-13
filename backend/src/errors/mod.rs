pub mod app_error;
pub mod app_error_impl;
pub mod app_result;

pub use app_error::AppError;
pub use app_result::AppResult;

// Backwards compatibility alias
pub type ApiError = AppError;
