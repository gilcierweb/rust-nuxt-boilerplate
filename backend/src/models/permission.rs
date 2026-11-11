#![allow(dead_code)]

use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::authz::ability::{CRUD_ACTIONS, DOMAIN_RESOURCES};
use crate::db::schema::{permissions, roles_permissions};

pub use crate::authz::ability::PermissionEffect;

impl PermissionCatalog {
    /// Builds the catalog from the enums: the same lists the engine validates
    /// against, so a new variant reaches the selects without a second edit.
    pub fn from_enums() -> Self {
        Self {
            resources: DOMAIN_RESOURCES
                .iter()
                .map(|resource| CatalogEntry {
                    code: resource.as_code().to_string(),
                    label_key: resource.label_key().to_string(),
                })
                .collect(),
            actions: CRUD_ACTIONS
                .iter()
                .map(|action| CatalogEntry {
                    code: action.as_code().to_string(),
                    label_key: action.label_key().to_string(),
                })
                .collect(),
        }
    }
}

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

/// The link of a role and a permission as the admin interface writes and reads
/// it: the machine code and the stored effect.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct RolePermissionItem {
    pub code: String,
    pub effect: i16,
}

impl RolePermissionItem {
    /// The modeled effect of the link, or `None` for a value the model does
    /// not define (fail closed: the caller refuses it).
    pub fn effect(&self) -> Option<PermissionEffect> {
        match self.effect {
            0 => Some(PermissionEffect::Can),
            1 => Some(PermissionEffect::Cannot),
            _ => None,
        }
    }
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

/// The fields of a permission the admin interface may write.
#[derive(Serialize, Deserialize, Debug, Clone, Default, AsChangeset, ToSchema)]
#[diesel(table_name = permissions)]
pub struct PermissionPatch {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// A role carrying a permission, with the effect of the link. The effect is
/// the stored SMALLINT: the model is the source of truth of the values and the
/// wire contract stays a number the frontend compares against 0 and 1.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct PermissionRole {
    pub role_id: Uuid,
    pub role_name: String,
    pub effect: i16,
}

/// A permission of the catalog as the admin interface lists it: the machine
/// code split into its parts and every role carrying it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct PermissionWithRoles {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub resource: String,
    pub action: String,
    pub roles: Vec<PermissionRole>,
}

/// One selectable value of a select box. The label is never duplicated here:
/// `label_key` is the i18n key the frontend resolves through vue-i18n.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct CatalogEntry {
    pub code: String,
    pub label_key: String,
}

/// The resources and the actions a permission code can name, derived from the
/// enums so the selects and the validation share one source of truth.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default, ToSchema)]
pub struct PermissionCatalog {
    pub resources: Vec<CatalogEntry>,
    pub actions: Vec<CatalogEntry>,
}
