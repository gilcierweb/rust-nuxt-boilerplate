#![allow(dead_code)]

use std::sync::Arc;

use async_trait::async_trait;
use diesel::QueryResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::permission::RolePermissionItem;
use crate::models::role::{NewRole, Role};
use crate::security::SecurityService;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

/// A member of a role: the user, their decrypted email and the display name
/// the admin interface shows.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RoleMemberItem {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
}

/// The sort fields the role list accepts. Anything outside the list is a client
/// mistake and the handler answers 422 instead of silently falling back.
pub const ROLE_SORT_FIELDS: [&str; 4] = ["name", "resource_type", "created_at", "updated_at"];

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IRoleRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<Role>>;
    async fn find(&self, id: &Uuid) -> QueryResult<Role>;
    async fn find_by_name(&self, name: &str) -> QueryResult<Option<Role>>;
    async fn create(&self, item: &NewRole) -> QueryResult<Role>;
    async fn update(&self, id: &Uuid, item: &NewRole) -> QueryResult<Role>;
    async fn destroy(&self, id: &Uuid) -> QueryResult<usize>;
    async fn list_paginated(
        &self,
        params: &PaginationParams,
        search: Option<String>,
    ) -> QueryResult<PaginatedResponse<Role>>;

    /// The links of a role, as the admin interface reads them back.
    async fn list_permission_links(&self, role_id: &Uuid) -> QueryResult<Vec<RolePermissionItem>>;

    /// The users holding the role, with the decrypted email.
    async fn list_members(
        &self,
        role_id: &Uuid,
        security: Arc<SecurityService>,
    ) -> QueryResult<Vec<RoleMemberItem>>;

    /// The roles holding every one of the codes (the administrator sources).
    /// Scoped never enter: a scoped role grants no class level authority.
    async fn role_ids_holding_all_codes(&self, codes: &[String]) -> QueryResult<Vec<Uuid>>;

    /// Replaces every link of a role in one transaction: the old links are
    /// deleted and the new ones inserted, so a refused insert leaves the role
    /// with its previous set instead of an empty one. Returns the number of
    /// links written.
    async fn replace_permission_links(
        &self,
        role_id: &Uuid,
        items: &[RolePermissionItem],
    ) -> QueryResult<usize>;
}
