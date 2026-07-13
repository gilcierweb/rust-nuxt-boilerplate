#![allow(dead_code)]

use async_trait::async_trait;
use diesel::QueryResult;
use uuid::Uuid;

use crate::models::user_role::{NewUserRole, UserRole};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IUserRoleRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<UserRole>>;
    async fn find(&self, user_id: &Uuid, role_id: &Uuid) -> QueryResult<UserRole>;
    async fn find_by_user(&self, user_id: &Uuid) -> QueryResult<Vec<UserRole>>;
    async fn find_by_role(&self, role_id: &Uuid) -> QueryResult<Vec<UserRole>>;
    async fn create(&self, item: &NewUserRole) -> QueryResult<UserRole>;
    async fn destroy(&self, user_id: &Uuid, role_id: &Uuid) -> QueryResult<usize>;
}
