use diesel::{Queryable, Selectable};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::schema::users;

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    #[serde(default)]
    pub id: Uuid,
    pub email_blind_index: Vec<u8>,
    pub email_encrypted: Vec<u8>,
    pub encrypted_password: String,

    // Recoverable
    pub reset_password_token_digest: Option<String>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub reset_password_sent_at: Option<OffsetDateTime>,

    // Rememberable
    #[serde(with = "crate::utils::wire_time::option")]
    pub remember_created_at: Option<OffsetDateTime>,

    // Trackable
    pub sign_in_count: i32,
    #[serde(with = "crate::utils::wire_time::option")]
    pub current_sign_in_at: Option<OffsetDateTime>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub last_sign_in_at: Option<OffsetDateTime>,
    pub current_sign_in_ip: Option<IpNet>,
    pub last_sign_in_ip: Option<IpNet>,

    // Confirmable
    pub confirmation_token_digest: Option<String>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub confirmed_at: Option<OffsetDateTime>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub confirmation_sent_at: Option<OffsetDateTime>,
    pub unconfirmed_email_blind_index: Option<Vec<u8>>,
    pub unconfirmed_email_encrypted: Option<Vec<u8>>,

    // Lockable
    pub failed_attempts: i32,
    pub unlock_token_digest: Option<String>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub locked_at: Option<OffsetDateTime>,

    // 2FA (TOTP)
    pub otp_secret: Option<String>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub otp_enabled_at: Option<OffsetDateTime>,
    pub otp_backup_codes: Option<Vec<Option<String>>>,
    pub encryption_key_version: i32,

    #[serde(with = "crate::utils::wire_time::datetime")]
    pub created_at: OffsetDateTime,
    #[serde(with = "crate::utils::wire_time::datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewUser {
    pub id: Uuid,
    pub email_blind_index: Vec<u8>,
    pub email_encrypted: Vec<u8>,
    pub encrypted_password: String,
    pub confirmation_token_digest: Option<String>,
    pub unconfirmed_email_blind_index: Option<Vec<u8>>,
    pub unconfirmed_email_encrypted: Option<Vec<u8>>,
    pub encryption_key_version: i32,
    #[serde(with = "crate::utils::wire_time::datetime")]
    pub created_at: OffsetDateTime,
    #[serde(with = "crate::utils::wire_time::datetime")]
    pub updated_at: OffsetDateTime,
}

impl NewUser {
    pub fn new(
        email_blind_index: Vec<u8>,
        email_encrypted: Vec<u8>,
        encrypted_password: String,
        confirmation_token_digest: Option<String>,
        encryption_key_version: i32,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            email_blind_index,
            email_encrypted,
            encrypted_password,
            confirmation_token_digest,
            unconfirmed_email_blind_index: None,
            unconfirmed_email_encrypted: None,
            encryption_key_version,
            created_at: now,
            updated_at: now,
        }
    }
}

// #[diesel(postgres_type(name = "inet"))] bug diesel not working type inet postgresql
