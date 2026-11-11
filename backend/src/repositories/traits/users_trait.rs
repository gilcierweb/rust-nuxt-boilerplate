#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::QueryResult;
use diesel::QueryableByName;
use diesel_async::AsyncPgConnection;
use futures::future::BoxFuture;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::role::Role;
use crate::models::user::{NewUser, User};
use crate::security::SecurityService;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

/// Raw grant row produced by `get_user_grants`: one entry per role permission
/// link of the user, with the role scope carried along. `permission_code` and
/// `effect` are nullable because roles without links still surface as rows.
#[derive(Debug, Clone, QueryableByName)]
pub struct RoleGrantRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub role_name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub resource_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub resource_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_code: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::SmallInt>)]
    pub effect: Option<i16>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IUserRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<User>>;
    async fn find(&self, id: &Uuid) -> QueryResult<User>;
    async fn create(&self, item: &NewUser) -> QueryResult<User>;
    async fn update(&self, id: &Uuid, item: &NewUser) -> QueryResult<User>;
    async fn destroy(&self, id: &Uuid) -> QueryResult<usize>;
    async fn count(&self) -> QueryResult<i64>;

    async fn find_by_username_or_email(
        &self,
        username_or_email: &str,
        email_blind_index: &[u8],
    ) -> QueryResult<Option<User>>;
    async fn find_by_email(&self, email_blind_index: &[u8]) -> QueryResult<Option<User>>;
    async fn find_by_reset_token_digest(&self, token_digest: &str) -> QueryResult<Option<User>>;
    async fn update_login_info(
        &self,
        id: &Uuid,
        current_sign_in_at: Option<NaiveDateTime>,
        last_sign_in_at: Option<NaiveDateTime>,
        current_sign_in_ip: Option<IpNet>,
        last_sign_in_ip: Option<IpNet>,
    ) -> QueryResult<User>;
    async fn update_password(&self, id: &Uuid, encrypted_password: &str) -> QueryResult<usize>;
    async fn update_reset_token(
        &self,
        id: &Uuid,
        token_digest: Option<String>,
        sent_at: Option<NaiveDateTime>,
    ) -> QueryResult<usize>;
    async fn update_pending_email(
        &self,
        user_id: &Uuid,
        blind_index: &[u8],
        encrypted_email: &[u8],
        token_digest: &str,
        sent_at: NaiveDateTime,
    ) -> QueryResult<usize>;

    async fn confirm_email(&self, token_digest: &str) -> QueryResult<usize>;
    async fn record_failed_login(
        &self,
        user_id: &Uuid,
        max_attempts: i32,
        refresh_token_hash_salt: &str,
    ) -> QueryResult<usize>;
    async fn record_successful_login(
        &self,
        user_id: &Uuid,
        ip: Option<IpNet>,
    ) -> QueryResult<usize>;
    async fn get_user_roles(&self, user_id: &Uuid) -> QueryResult<Vec<String>>;
    async fn get_user_permissions(&self, user_id: &Uuid) -> QueryResult<Vec<String>>;
    /// The roles of the given name that can be assigned to a new account.
    ///
    /// A role bound to a specific subject (`resource_id` set) is never
    /// assignable: it belongs to another account.
    async fn find_assignable_roles_by_name(&self, name: &str) -> QueryResult<Vec<Role>>;
    async fn get_user_grants(
        &self,
        user_id: &Uuid,
    ) -> QueryResult<Vec<crate::repositories::traits::users_trait::RoleGrantRow>>;
    async fn user_ids_holding_all_codes(&self, codes: &[String]) -> QueryResult<Vec<Uuid>>;

    /// The roles assigned to a user, as the admin interface reads them back.
    async fn get_user_role_assignments(&self, user_id: &Uuid) -> QueryResult<Vec<(Uuid, String)>>;

    /// Replaces every role assignment of a user in one transaction: the old
    /// links are deleted and the new ones inserted, so a refused insert leaves
    /// the user with the previous roles instead of none.
    async fn set_user_roles(&self, user_id: &Uuid, role_ids: &[Uuid]) -> QueryResult<usize>;

    /// The effective allow codes of a role, expanded over the wildcards
    /// (`Manage` and `All` expand to the catalog).
    async fn role_allow_codes(&self, role_id: &Uuid) -> QueryResult<HashSet<String>>;
    async fn create_password_reset_token(
        &self,
        user_id: &Uuid,
        token_digest: &str,
        sent_at: NaiveDateTime,
    ) -> QueryResult<usize>;
    async fn reset_password(&self, token_digest: &str, new_password: &str) -> QueryResult<usize>;
    async fn set_otp_secret(&self, user_id: &Uuid, secret: &str) -> QueryResult<usize>;
    async fn enable_2fa(
        &self,
        user_id: &Uuid,
        backup_codes: &[Option<String>],
    ) -> QueryResult<usize>;
    async fn disable_2fa(&self, user_id: &Uuid) -> QueryResult<usize>;
    async fn update_backup_codes(
        &self,
        user_id: &Uuid,
        backup_codes: &[Option<String>],
    ) -> QueryResult<usize>;
    async fn list_paginated(
        &self,
        params: &PaginationParams,
        security: Arc<SecurityService>,
    ) -> QueryResult<PaginatedResponse<AdminUserLookupItem>>;

    async fn find_by_id_with_profile(
        &self,
        id: &Uuid,
        security: Arc<SecurityService>,
    ) -> QueryResult<AdminUserItem>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IUserRepositoryTransaction: Send + Sync {
    async fn run_transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, Result<T, E>>
            + Send
            + 'static,
        T: Send + 'static,
        E: From<diesel::result::Error> + Send + 'static;
}

/// DTO for paginated user list (includes profile data)
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AdminUserLookupItem {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub nickname: Option<String>,
}

/// DTO for single user detail (includes profile data)
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AdminUserItem {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub nickname: Option<String>,
    pub display_name: Option<String>,
}
