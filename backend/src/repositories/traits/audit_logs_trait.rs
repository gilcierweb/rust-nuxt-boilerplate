#![allow(dead_code)]

use async_trait::async_trait;
use diesel::QueryResult;
use uuid::Uuid;

use crate::models::audit_log::{AuditLog, NewAuditLog};
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

/// The sort fields the audit log list accepts. Anything outside the list is a
/// client mistake and the handler answers 422.
pub const AUDIT_LOG_SORT_FIELDS: [&str; 3] = ["action", "resource_type", "created_at"];

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IAuditLogRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<AuditLog>>;
    async fn find(&self, id: &Uuid) -> QueryResult<AuditLog>;
    async fn create(&self, item: &NewAuditLog) -> QueryResult<AuditLog>;
    async fn find_latest_hash(&self) -> QueryResult<Option<String>>;
    async fn find_batch_ordered_by_created_at(
        &self,
        cursor_id: Option<Uuid>,
        limit: i64,
    ) -> QueryResult<Vec<AuditLog>>;
    async fn list_paginated(
        &self,
        params: &PaginationParams,
        search: Option<String>,
    ) -> QueryResult<PaginatedResponse<AuditLog>>;
}
