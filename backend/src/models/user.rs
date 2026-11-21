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
    // The secret columns are skipped on the way out: a handler that ever
    // answers with this struct directly would otherwise serialize the
    // password hash, the TOTP secret, the backup codes, the encrypted mailbox
    // and every token digest. They stay readable and writable from Rust; only
    // the JSON boundary drops them.
    #[serde(skip_serializing)]
    pub email_blind_index: Vec<u8>,
    #[serde(skip_serializing)]
    pub email_encrypted: Vec<u8>,
    #[serde(skip_serializing)]
    pub encrypted_password: String,

    // Recoverable
    #[serde(skip_serializing)]
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
    #[serde(skip_serializing)]
    pub confirmation_token_digest: Option<String>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub confirmed_at: Option<OffsetDateTime>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub confirmation_sent_at: Option<OffsetDateTime>,
    #[serde(skip_serializing)]
    pub unconfirmed_email_blind_index: Option<Vec<u8>>,
    #[serde(skip_serializing)]
    pub unconfirmed_email_encrypted: Option<Vec<u8>>,

    // Lockable
    pub failed_attempts: i32,
    #[serde(skip_serializing)]
    pub unlock_token_digest: Option<String>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub locked_at: Option<OffsetDateTime>,

    // 2FA (TOTP)
    #[serde(skip_serializing)]
    pub otp_secret: Option<String>,
    #[serde(with = "crate::utils::wire_time::option")]
    pub otp_enabled_at: Option<OffsetDateTime>,
    #[serde(skip_serializing)]
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

#[cfg(test)]
mod serialization_tests {
    use super::*;

    fn stored_user() -> User {
        let now = OffsetDateTime::now_utc();
        User {
            id: Uuid::new_v4(),
            email_blind_index: vec![0u8; 32],
            email_encrypted: vec![1u8; 32],
            encrypted_password: "argon2id-hash".to_string(),
            reset_password_token_digest: Some("reset-digest".to_string()),
            reset_password_sent_at: Some(now),
            remember_created_at: None,
            sign_in_count: 1,
            current_sign_in_at: Some(now),
            last_sign_in_at: None,
            current_sign_in_ip: None,
            last_sign_in_ip: None,
            confirmation_token_digest: Some("confirm-digest".to_string()),
            confirmed_at: Some(now),
            confirmation_sent_at: None,
            unconfirmed_email_blind_index: None,
            unconfirmed_email_encrypted: Some(vec![2u8; 32]),
            failed_attempts: 0,
            unlock_token_digest: Some("unlock-digest".to_string()),
            locked_at: None,
            otp_secret: Some("JBSWY3DPEHPK3PXP".to_string()),
            otp_enabled_at: Some(now),
            otp_backup_codes: Some(vec![Some("backup-hash".to_string())]),
            encryption_key_version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn a_serialized_user_never_carries_the_secret_material() {
        // The struct derives Serialize as a Diesel convenience. A handler that
        // ever answers with a User directly must not leak the password hash,
        // the TOTP secret, the backup codes, the encrypted mailbox or the
        // token digests, so they are skipped at the serialization boundary.
        let json = serde_json::to_value(stored_user()).expect("a User serializes");
        for forbidden in [
            "encrypted_password",
            "otp_secret",
            "otp_backup_codes",
            "reset_password_token_digest",
            "confirmation_token_digest",
            "unlock_token_digest",
            "email_encrypted",
            "unconfirmed_email_encrypted",
        ] {
            assert!(
                json.get(forbidden).is_none(),
                "the serialized user must not carry {forbidden}"
            );
        }

        // The identity fields a legitimate response would need stay.
        assert!(json.get("id").is_some());
        assert!(json.get("confirmed_at").is_some());
    }
}
