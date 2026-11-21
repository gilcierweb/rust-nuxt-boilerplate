#![allow(dead_code)]

use async_trait::async_trait;
use diesel::QueryResult;
use uuid::Uuid;

use crate::models::refresh_token::{NewRefreshToken, RefreshToken};
use crate::repositories::refresh_tokens_repository::RotationOutcome;

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
    /// against stored hashes. Supports both HMAC-SHA256 (new) and Argon2id (legacy) formats.
    async fn find_valid_by_token(
        &self,
        plaintext_token: &str,
        hash_key: &str,
    ) -> QueryResult<Option<RefreshToken>>;

    /// Atomically revoke an existing token and create a new one in a single
    /// transaction. Takes the old token plaintext, expiry seconds, and hash salt.
    ///
    /// The outcome distinguishes the cases the caller must treat differently:
    /// `Rotated` carries the new row and its plaintext, `ReuseDetected` means
    /// the presented token had already been revoked inside the grace window
    /// (the whole family is now revoked and the caller must invalidate the
    /// user's access tokens), and `NotFound` means nothing matched.
    async fn rotate_token(
        &self,
        old_token_plaintext: &str,
        expires_in_seconds: i64,
        salt: &str,
    ) -> QueryResult<RotationOutcome>;
}
