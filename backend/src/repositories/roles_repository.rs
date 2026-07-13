use crate::DBPool;
use crate::db::schema::roles as roles_table;
use crate::models::role::{NewRole, Role};
use crate::repositories::base::BaseRepo;
pub use crate::repositories::traits::roles_trait::IRoleRepository;

pub struct RolesRepository {
    base: BaseRepo,
}

impl RolesRepository {
    pub fn new(pool: DBPool) -> Self {
        Self {
            base: BaseRepo::new(pool),
        }
    }
}

#[cfg(test)]
pub use crate::repositories::traits::roles_trait::MockIRoleRepository;

crate::impl_crud!(
    RolesRepository,
    base,
    IRoleRepository,
    Role,
    NewRole,
    roles_table::table
);
