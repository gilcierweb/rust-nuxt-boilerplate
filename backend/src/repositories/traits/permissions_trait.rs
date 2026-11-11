#![allow(dead_code)]

use async_trait::async_trait;
use diesel::QueryResult;
use uuid::Uuid;

use crate::models::permission::{NewPermission, Permission, PermissionPatch, PermissionWithRoles};
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

/// The sort fields the permission list accepts. Anything outside the list is a
/// client mistake and the handler answers 422 instead of silently falling back
/// to a default order.
pub const PERMISSION_SORT_FIELDS: [&str; 3] = ["name", "code", "updated_at"];

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IPermissionRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<Permission>>;
    async fn find(&self, id: &Uuid) -> QueryResult<Permission>;
    async fn find_by_code(&self, code: &str) -> QueryResult<Option<Permission>>;
    async fn create(&self, item: &NewPermission) -> QueryResult<Permission>;
    async fn update(&self, id: &Uuid, patch: &PermissionPatch) -> QueryResult<Permission>;
    async fn destroy(&self, id: &Uuid) -> QueryResult<usize>;
    async fn linked_role_count(&self, id: &Uuid) -> QueryResult<i64>;
    /// One permission with the roles carrying it, the same shape the list
    /// returns, so a create or an update answers with the full item.
    async fn find_with_roles(&self, id: &Uuid) -> QueryResult<PermissionWithRoles>;
    /// The catalog page: every permission with the roles carrying it, joined in
    /// one query so the admin interface never runs an N plus one.
    async fn list(
        &self,
        params: &PaginationParams,
        search: Option<String>,
    ) -> QueryResult<PaginatedResponse<PermissionWithRoles>>;
}
