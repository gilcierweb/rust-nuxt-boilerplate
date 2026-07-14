#![allow(dead_code)]

use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::QueryResult;
use ipnet::IpNet;
use uuid::Uuid;

use crate::models::user::{NewUser, User};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IUserRepository: Send + Sync {
    async fn all(&self) -> QueryResult<Vec<User>>;
    async fn find(&self, id: &Uuid) -> QueryResult<User>;
    async fn create(&self, item: &NewUser) -> QueryResult<User>;
    async fn update(&self, id: &Uuid, item: &NewUser) -> QueryResult<User>;
    async fn destroy(&self, id: &Uuid) -> QueryResult<usize>;

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
    async fn record_failed_login(&self, user_id: &Uuid, max_attempts: i32) -> QueryResult<usize>;
    async fn record_successful_login(
        &self,
        user_id: &Uuid,
        ip: Option<IpNet>,
    ) -> QueryResult<usize>;
    async fn get_user_roles(&self, user_id: &Uuid) -> QueryResult<Vec<String>>;
    async fn get_user_permissions(&self, user_id: &Uuid) -> QueryResult<Vec<String>>;
    async fn create_password_reset_token(
        &self,
        user_id: &Uuid,
        token_digest: &str,
        sent_at: NaiveDateTime,
    ) -> QueryResult<usize>;
    async fn reset_password(&self, token_digest: &str, new_password: &str) -> QueryResult<usize>;
    async fn set_otp_secret(&self, user_id: &Uuid, secret: &str) -> QueryResult<usize>;
    async fn enable_2fa(&self, user_id: &Uuid, backup_codes: &[String]) -> QueryResult<usize>;
    async fn disable_2fa(&self, user_id: &Uuid) -> QueryResult<usize>;
}
