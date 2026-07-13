#![allow(dead_code)]

use std::sync::Arc;

use crate::config::app_config::AppConfig;
use crate::db::database::DBPool;
use crate::repositories::audit_logs_repository::{AuditLogsRepository, IAuditLogRepository};
use crate::repositories::profiles_repository::{IProfileRepository, ProfilesRepository};
use crate::repositories::refresh_tokens_repository::{
    IRefreshTokenRepository, RefreshTokensRepository,
};
use crate::repositories::roles_repository::{IRoleRepository, RolesRepository};
use crate::repositories::user_roles_repository::{IUserRoleRepository, UserRolesRepository};
use crate::repositories::users_repository::{IUserRepository, UsersRepository};

pub struct AppContainer {
    pub config: Arc<AppConfig>,
    pub cache: Arc<crate::services::cache_service::CacheManager>,
    pub users: Arc<dyn IUserRepository>,
    pub profiles: Arc<dyn IProfileRepository>,
    pub refresh_tokens: Arc<dyn IRefreshTokenRepository>,
    pub user_roles: Arc<dyn IUserRoleRepository>,
    pub roles: Arc<dyn IRoleRepository>,
    pub domain_audit_logs: Arc<dyn IAuditLogRepository>,
}

impl AppContainer {
    pub fn new(pool: DBPool, redis_pool: deadpool_redis::Pool, config: AppConfig) -> Self {
        let cache = Arc::new(crate::services::cache_service::CacheManager::from_pool(
            redis_pool,
            std::time::Duration::from_secs(3600),
        ));

        Self {
            config: Arc::new(config),
            cache,
            users: Arc::new(UsersRepository::new(pool.clone())),
            profiles: Arc::new(ProfilesRepository::new(pool.clone())),
            refresh_tokens: Arc::new(RefreshTokensRepository::new(pool.clone())),
            user_roles: Arc::new(UserRolesRepository::new(pool.clone())),
            roles: Arc::new(RolesRepository::new(pool.clone())),
            domain_audit_logs: Arc::new(AuditLogsRepository::new(pool)),
        }
    }
}
