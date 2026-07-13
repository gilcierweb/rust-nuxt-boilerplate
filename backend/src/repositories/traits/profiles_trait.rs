#![allow(dead_code)]

use async_trait::async_trait;
use diesel::QueryResult;
use uuid::Uuid;

use crate::models::profile::{NewProfile, Profile};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IProfileRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<Profile>>;
    async fn find(&self, id: &Uuid) -> QueryResult<Profile>;
    async fn create(&self, item: &NewProfile) -> QueryResult<Profile>;
    async fn update(&self, id: &Uuid, item: &NewProfile) -> QueryResult<Profile>;
    async fn destroy(&self, id: &Uuid) -> QueryResult<usize>;

    async fn find_by_user_id(&self, user_id: &Uuid) -> QueryResult<Option<Profile>>;
    async fn find_by_cpf_blind_index(&self, cpf_blind_index: &[u8])
    -> QueryResult<Option<Profile>>;
    async fn find_by_phone_blind_index(
        &self,
        phone_blind_index: &[u8],
    ) -> QueryResult<Option<Profile>>;
    async fn find_by_whatsapp_blind_index(
        &self,
        whatsapp_blind_index: &[u8],
    ) -> QueryResult<Option<Profile>>;
}
