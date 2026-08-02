#![allow(dead_code)]

use async_trait::async_trait;
use diesel::QueryResult;
use uuid::Uuid;

use crate::models::magic_link_token::{MagicLinkToken, NewMagicLinkToken};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IMagicLinkTokenRepository: Send + Sync {
    async fn create(&self, item: &NewMagicLinkToken) -> QueryResult<MagicLinkToken>;
    async fn find_by_digest(&self, token_digest: &str) -> QueryResult<Option<MagicLinkToken>>;
    async fn mark_consumed(&self, id: &Uuid) -> QueryResult<usize>;
    async fn delete_expired(&self) -> QueryResult<usize>;
}
