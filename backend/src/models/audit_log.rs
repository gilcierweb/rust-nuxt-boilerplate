use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::db::schema::audit_logs;

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = audit_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AuditLog {
    #[serde(default)]
    pub id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub actor_role_snapshot: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<IpNet>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub changes: Value,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, AsChangeset, Validate)]
#[diesel(table_name = audit_logs)]
pub struct NewAuditLog {
    pub actor_user_id: Option<Uuid>,
    #[validate(length(max = 255))]
    pub actor_role_snapshot: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub action: String,
    #[validate(length(min = 1, max = 255))]
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<IpNet>,
    #[validate(length(max = 500))]
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub changes: Value,
    pub metadata: Value,
}
