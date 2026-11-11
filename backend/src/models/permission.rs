#![allow(dead_code)]

use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::schema::{permissions, roles_permissions};

pub use crate::authz::ability::PermissionEffect;

/// A permission of the rbac catalog: the machine code used by the engine and
/// the human readable name shown by the admin interface.
#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Selectable, ToSchema)]
#[diesel(table_name = permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Permission {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name = permissions)]
pub struct NewPermission {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

/// The link between a role and a permission. `effect` decides whether the
/// permission is granted (allow) or refused (deny); deny wins over allow.
#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Selectable, ToSchema)]
#[diesel(table_name = roles_permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RolePermissionLink {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub effect: i16,
    pub created_at: DateTime<Utc>,
}
