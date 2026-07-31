use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Default page size for list endpoints
pub const DEFAULT_PAGE_SIZE: i64 = 20;

/// Maximum allowed page size for admin endpoints
pub const MAX_PAGE_SIZE: i64 = 100;

/// Maximum allowed page number to prevent abuse
pub const MAX_PAGE: i64 = 10_000;

/// Sort direction
pub type SortDir = String;

/// List parameters (alias for PaginationParams)
pub type ListParams = PaginationParams;

/// Pagination and sorting parameters for list endpoints
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone, ToSchema)]
#[schema(
    example = json!({
        "page": 1,
        "per_page": 20,
        "sort_by": "created_at",
        "sort_dir": "desc"
    })
)]
pub struct PaginationParams {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: i64,
    /// Items per page (max 100)
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    /// Sort field name (e.g. "created_at", "email", "name")
    pub sort_by: Option<String>,
    /// Sort direction: "asc" or "desc"
    pub sort_dir: Option<String>,
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
            sort_by: None,
            sort_dir: None,
        }
    }

    /// Validates and clamps pagination parameters to safe bounds
    pub fn validated(&self) -> Self {
        Self {
            page: self.page.clamp(1, MAX_PAGE),
            per_page: self.per_page.clamp(1, MAX_PAGE_SIZE),
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir.clone(),
        }
    }

    /// Returns true if sorting is requested
    pub fn has_sort(&self) -> bool {
        self.sort_by
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }

    /// Returns normalized sort direction (lowercase), defaulting to "asc"
    pub fn sort_direction(&self) -> &'static str {
        match self.sort_dir.as_deref() {
            Some("desc") => "desc",
            _ => "asc",
        }
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: DEFAULT_PAGE_SIZE,
            sort_by: None,
            sort_dir: None,
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
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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

    /// Build a paginated response from a full list, applying in-memory sorting first
    pub fn from_sorted_list(
        mut data: Vec<T>,
        params: &PaginationParams,
        sort_fn: impl Fn(&mut Vec<T>, &str, bool),
    ) -> Self
    where
        T: serde::Serialize,
    {
        if params.has_sort() {
            let field = params.sort_by.as_deref().unwrap_or("");
            let desc = params.sort_direction() == "desc";
            sort_fn(&mut data, field, desc);
        }

        let total = data.len() as i64;
        let offset = params.offset() as usize;
        let limit = params.limit() as usize;

        let paginated_data: Vec<T> = data.into_iter().skip(offset).take(limit).collect();
        Self::new(paginated_data, total, params.page, params.per_page)
    }
}

/// Pagination metadata
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
