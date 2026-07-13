use chrono::{DateTime, NaiveDate, Utc};
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::db::schema::profiles;

fn default_encryption_key_version() -> i32 {
    1
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Profile {
    #[serde(default)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub nickname: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub birthday: Option<NaiveDate>,
    #[serde(skip_serializing, default)]
    pub cpf_encrypted: Option<Vec<u8>>,
    #[serde(skip_serializing, default)]
    pub cpf_blind_index: Option<Vec<u8>>,
    #[serde(skip_serializing, default)]
    pub phone_encrypted: Option<Vec<u8>>,
    #[serde(skip_serializing, default)]
    pub phone_blind_index: Option<Vec<u8>>,
    #[serde(skip_serializing, default)]
    pub whatsapp_encrypted: Option<Vec<u8>>,
    #[serde(skip_serializing, default)]
    pub whatsapp_blind_index: Option<Vec<u8>>,
    pub status: bool,
    pub social_network: Value,
    #[serde(skip_serializing, default = "default_encryption_key_version")]
    pub encryption_key_version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = profiles)]
pub struct NewProfile {
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub nickname: Option<String>,
    pub bio: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub cpf_encrypted: Option<Vec<u8>>,
    pub cpf_blind_index: Option<Vec<u8>>,
    pub phone_encrypted: Option<Vec<u8>>,
    pub phone_blind_index: Option<Vec<u8>>,
    pub whatsapp_encrypted: Option<Vec<u8>>,
    pub whatsapp_blind_index: Option<Vec<u8>>,
    pub avatar: Option<String>,
    pub status: bool,
    pub social_network: Value,
    pub encryption_key_version: i32,
}

impl NewProfile {
    pub fn for_user(user_id: Uuid, encryption_key_version: i32) -> Self {
        Self {
            user_id,
            first_name: None,
            last_name: None,
            full_name: None,
            nickname: None,
            bio: None,
            birthday: None,
            cpf_encrypted: None,
            cpf_blind_index: None,
            phone_encrypted: None,
            phone_blind_index: None,
            whatsapp_encrypted: None,
            whatsapp_blind_index: None,
            avatar: None,
            status: true,
            social_network: serde_json::json!({}),
            encryption_key_version,
        }
    }
}
