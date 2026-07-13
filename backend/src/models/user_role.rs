use diesel::pg::Pg;
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::users_roles;

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, Selectable)]
#[diesel(check_for_backend(Pg))]
#[diesel(table_name = users_roles)]
pub struct UserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[diesel(table_name = users_roles)]
pub struct NewUserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
}
