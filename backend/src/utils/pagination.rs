use serde::{Deserialize, Serialize};

/// Default page size for list endpoints
pub const DEFAULT_PAGE_SIZE: i64 = 20;

/// Maximum allowed page size for admin endpoints
pub const MAX_PAGE_SIZE: i64 = 100;

/// Maximum allowed page number to prevent abuse
pub const MAX_PAGE: i64 = 10_000;

/// Pagination parameters for list endpoints
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct PaginationParams {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Items per page (max 100)
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

impl PaginationParams {
    /// Returns offset for database queries
    #[allow(dead_code)]
    pub fn offset(&self) -> i64 {
        ((self.page - 1) * self.per_page).max(0)
    }

    /// Returns limit for database queries (clamped to MAX_PAGE_SIZE)
    #[allow(dead_code)]
    pub fn limit(&self) -> i64 {
        self.per_page.clamp(1, MAX_PAGE_SIZE)
    }

    /// Creates pagination params with defaults and validation
    #[allow(dead_code)]
    pub fn new(page: i64, per_page: i64) -> Self {
        Self {
            page: page.clamp(1, MAX_PAGE),
            per_page: per_page.clamp(1, MAX_PAGE_SIZE),
        }
    }

    /// Validates and clamps pagination parameters to safe bounds
    pub fn validated(&self) -> Self {
        Self {
            page: self.page.clamp(1, MAX_PAGE),
            per_page: self.per_page.clamp(1, MAX_PAGE_SIZE),
        }
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: DEFAULT_PAGE_SIZE,
        }
    }
}

#[allow(dead_code)]
fn default_page() -> i64 {
    1
}

#[allow(dead_code)]
fn default_per_page() -> i64 {
    DEFAULT_PAGE_SIZE
}

/// Paginated response wrapper
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    #[allow(dead_code)]
    pub fn new(data: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = if per_page == 0 {
            1
        } else {
            ((total as f64 / per_page as f64).ceil() as i64).max(1)
        };
        Self {
            data,
            pagination: PaginationMeta {
                page,
                per_page,
                total,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
        }
    }
}

/// Pagination metadata
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Cursor-based pagination params
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct CursorParams {
    pub cursor: Option<String>,
    #[serde(default = "default_per_page")]
    pub limit: i64,
}

impl CursorParams {
    #[allow(dead_code)]
    pub fn limit(&self) -> i64 {
        self.limit.clamp(1, MAX_PAGE_SIZE)
    }
}

/// Cursor-based paginated response
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct CursorResponse<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl<T> CursorResponse<T> {
    #[allow(dead_code)]
    pub fn new(data: Vec<T>, next_cursor: Option<String>, has_more: bool) -> Self {
        Self {
            data,
            next_cursor,
            has_more,
        }
    }
}
