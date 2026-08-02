use chrono::{DateTime, Utc};
use diesel::prelude::*;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::magic_link_tokens;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = magic_link_tokens)]
pub struct MagicLinkToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_digest: String,
    pub request_ip: Option<IpNet>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = magic_link_tokens)]
pub struct NewMagicLinkToken {
    pub user_id: Uuid,
    pub token_digest: String,
    pub request_ip: Option<IpNet>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl NewMagicLinkToken {
    pub fn new(
        user_id: Uuid,
        token_digest: String,
        request_ip: Option<IpNet>,
        user_agent: Option<String>,
        expires_at: chrono::DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            token_digest,
            request_ip,
            user_agent,
            expires_at,
            consumed_at: None,
            created_at: Utc::now(),
        }
    }
}
