#![allow(dead_code)]

use async_trait::async_trait;
use diesel::QueryResult;
use uuid::Uuid;

use crate::models::audit_log::{AuditLog, NewAuditLog};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IAuditLogRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<AuditLog>>;
    async fn find(&self, id: &Uuid) -> QueryResult<AuditLog>;
    async fn create(&self, item: &NewAuditLog) -> QueryResult<AuditLog>;
    async fn update(&self, id: &Uuid, item: &NewAuditLog) -> QueryResult<AuditLog>;
    async fn destroy(&self, id: &Uuid) -> QueryResult<usize>;
}
