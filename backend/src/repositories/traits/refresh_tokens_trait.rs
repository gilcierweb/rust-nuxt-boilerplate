#![allow(dead_code)]

use async_trait::async_trait;
use diesel::QueryResult;
use uuid::Uuid;

use crate::models::refresh_token::{NewRefreshToken, RefreshToken};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IRefreshTokenRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<RefreshToken>>;
    async fn find(&self, id: &Uuid) -> QueryResult<RefreshToken>;
    async fn create(&self, item: &NewRefreshToken) -> QueryResult<RefreshToken>;
    async fn destroy(&self, id: &Uuid) -> QueryResult<usize>;

    async fn find_by_token_hash(&self, token_hash: &str) -> QueryResult<Option<RefreshToken>>;
    async fn revoke(&self, id: &Uuid) -> QueryResult<usize>;
    async fn revoke_all_for_user(&self, user_id: &Uuid) -> QueryResult<usize>;

    /// Atomically revoke an existing token and create a new one in a single transaction.
    /// Returns the new token on success, or None if the token was not found or already revoked.
    async fn rotate_token(
        &self,
        old_token_hash: &str,
        new_token: &NewRefreshToken,
    ) -> QueryResult<Option<RefreshToken>>;
}
