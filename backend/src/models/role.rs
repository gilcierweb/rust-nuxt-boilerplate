#![allow(dead_code)]

use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::roles;

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    #[serde(default)]
    pub id: Uuid,
    pub name: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = roles)]
pub struct NewRole {
    pub name: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum UserRole {
    Admin = 1,
    Operator = 2,
    Viewer = 3,
}

impl UserRole {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

impl TryFrom<i32> for UserRole {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(UserRole::Admin),
            2 => Ok(UserRole::Operator),
            3 => Ok(UserRole::Viewer),
            _ => Err("Invalid role value"),
        }
    }
}

impl From<UserRole> for i32 {
    fn from(role: UserRole) -> Self {
        role.as_i32()
    }
}

pub const ROLE_ADMIN: UserRole = UserRole::Admin;
pub const ROLE_OPERATOR: UserRole = UserRole::Operator;
pub const ROLE_VIEWER: UserRole = UserRole::Viewer;
