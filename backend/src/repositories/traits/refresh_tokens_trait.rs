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

    /// Find a valid (non-revoked, non-expired) refresh token by verifying the plaintext token
    /// against stored Argon2id hashes. Used for Argon2id token verification.
    async fn find_valid_by_token(&self, plaintext_token: &str) -> QueryResult<Option<RefreshToken>>;

    /// Atomically revoke an existing token and create a new one in a single transaction.
    /// Takes the old token plaintext, expiry seconds, and hash salt.
    /// Returns the new RefreshToken (with hash) and the new plain token for the cookie.
    /// Returns None if the old token was not found, already revoked, or expired.
    async fn rotate_token(
        &self,
        old_token_plaintext: &str,
        expires_in_seconds: i64,
        salt: &str,
    ) -> QueryResult<Option<(RefreshToken, String)>>;
}
