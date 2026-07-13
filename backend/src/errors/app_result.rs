use crate::errors::AppError;

pub type AppResult<T> = Result<T, AppError>;
