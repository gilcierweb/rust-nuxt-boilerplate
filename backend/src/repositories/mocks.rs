use std::sync::Arc;

use crate::config::app_config::AppConfig;
use crate::config::test_config::test_config;
use crate::repositories::audit_logs_repository::MockIAuditLogRepository;
use crate::repositories::container::AppContainer;
use crate::repositories::magic_link_token_repository::MockIMagicLinkTokenRepository;
use crate::repositories::profiles_repository::MockIProfileRepository;
use crate::repositories::refresh_tokens_repository::MockIRefreshTokenRepository;
use crate::repositories::roles_repository::MockIRoleRepository;
use crate::repositories::user_roles_repository::MockIUserRoleRepository;
use crate::repositories::users_repository::{MockIUserRepository, UsersRepository};
use crate::services::cache_service::CacheManager;

/// Create a dummy DB pool for fields that require a concrete type (e.g. users_tx).
/// The pool is never actually used in mock tests — it only satisfies the type system.
fn dummy_db_pool() -> crate::db::database::DBPool {
    use deadpool::managed::Pool;
    use diesel_async::pooled_connection::AsyncDieselConnectionManager;
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        "postgres://localhost:5432/dummy",
    );
    Pool::builder(manager)
        .max_size(1)
        .runtime(deadpool::Runtime::Tokio1)
        .build()
        .expect("Failed to create dummy pool")
}

/// Build a deterministic test `AppConfig` (shared with other modules).
pub fn mock_app_config() -> AppConfig {
    test_config()
}

pub fn mock_container() -> AppContainer {
    let redis_cfg = deadpool_redis::Config::from_url("redis://127.0.0.1:6379");
    let pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .unwrap();

    let cache = Arc::new(CacheManager::from_pool(
        pool.clone(),
        std::time::Duration::from_secs(60),
    ));

    let config = test_config();
    let email_service = Arc::new(crate::services::email_service::EmailService::new(&config));

    let users_repo = Arc::new(MockIUserRepository::new());

    AppContainer {
        config: Arc::new(config),
        cache,
        users: users_repo.clone(),
        users_tx: Arc::new(UsersRepository::new(dummy_db_pool())),
        profiles: Arc::new(MockIProfileRepository::new()),
        refresh_tokens: Arc::new(MockIRefreshTokenRepository::new()),
        user_roles: Arc::new(MockIUserRoleRepository::new()),
        roles: Arc::new(MockIRoleRepository::new()),
        domain_audit_logs: Arc::new(MockIAuditLogRepository::new()),
        magic_link_tokens: Arc::new(MockIMagicLinkTokenRepository::new()),
        access_token_blacklist: Arc::new(
            crate::repositories::access_token_blacklist::AccessTokenBlacklist::new(pool),
        ),
        email_service,
    }
}

/// Create a mock container with pre-configured user repository expectations.
pub fn mock_container_with_user(user: crate::models::user::User) -> AppContainer {
    let mut container = mock_container();

    let email_blind_index = user.email_blind_index.clone();
    let user_id = user.id;
    let user_for_find = user.clone();
    let user_for_email = user.clone();

    let mut mock_user_repo = MockIUserRepository::new();
    mock_user_repo
        .expect_find()
        .withf(move |id| *id == user_id)
        .times(1)
        .returning(move |_| Ok(user_for_find.clone()));
    mock_user_repo
        .expect_find_by_email()
        .withf(move |blind_index| blind_index == email_blind_index)
        .times(1)
        .returning(move |_| Ok(Some(user_for_email.clone())));

    container.users = Arc::new(mock_user_repo);
    container
}
