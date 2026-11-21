use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::schema::refresh_tokens;

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset, Selectable)]
#[diesel(table_name = refresh_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    #[serde(with = "crate::utils::wire_time::datetime")]
    pub expires_at: OffsetDateTime,
    #[serde(with = "crate::utils::wire_time::option")]
    pub revoked_at: Option<OffsetDateTime>,
    #[serde(with = "crate::utils::wire_time::datetime")]
    pub created_at: OffsetDateTime,
    #[serde(with = "crate::utils::wire_time::datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = refresh_tokens)]
pub struct NewRefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    #[serde(with = "crate::utils::wire_time::datetime")]
    pub expires_at: OffsetDateTime,
    #[serde(with = "crate::utils::wire_time::datetime")]
    pub created_at: OffsetDateTime,
    #[serde(with = "crate::utils::wire_time::datetime")]
    pub updated_at: OffsetDateTime,
}

impl RefreshToken {
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() && self.expires_at > OffsetDateTime::now_utc()
    }
}
