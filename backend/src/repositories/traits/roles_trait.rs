#![allow(dead_code)]

use async_trait::async_trait;
use diesel::QueryResult;
use uuid::Uuid;

use crate::models::role::{NewRole, Role};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IRoleRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<Role>>;
    async fn find(&self, id: &Uuid) -> QueryResult<Role>;
    async fn create(&self, item: &NewRole) -> QueryResult<Role>;
    async fn update(&self, id: &Uuid, item: &NewRole) -> QueryResult<Role>;
    async fn destroy(&self, id: &Uuid) -> QueryResult<usize>;
}
