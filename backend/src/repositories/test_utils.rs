#[cfg(test)]
pub mod mocks {
    use crate::config::app_config::{AppConfig, Environment};
    use crate::repositories::audit_logs_repository::MockIAuditLogRepository;
    use crate::repositories::container::AppContainer;
    use crate::repositories::profiles_repository::MockIProfileRepository;
    use crate::repositories::refresh_tokens_repository::MockIRefreshTokenRepository;
    use crate::repositories::roles_repository::MockIRoleRepository;
    use crate::repositories::user_roles_repository::MockIUserRoleRepository;
    use crate::repositories::users_repository::MockIUserRepository;
    use crate::services::cache_service::CacheManager;
    use std::sync::Arc;

    pub fn mock_app_config() -> AppConfig {
        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            https_port: 8443,
            tls_cert_path: "cert.pem".to_string(),
            tls_key_path: "key.pem".to_string(),
            frontend_url: "http://localhost:3000".to_string(),
            environment: Environment::Test,
            database_url: "postgres://justfans:password@localhost:5432/justfans_test".to_string(),
            db_pool_size: 1,
            db_pool_min_idle: Some(1),
            db_pool_max_lifetime_secs: Some(1800),
            db_pool_idle_timeout_secs: Some(600),
            db_pool_connection_timeout_secs: 10,
            redis_url: "redis://127.0.0.1:6379".to_string(),
            redis_pool_size: 10,
            jwt_secret: "secret_for_tests".to_string(),
            jwt_public_key: None,
            jwt_access_expiry_secs: 3600,
            jwt_refresh_expiry_secs: 3600,
            master_key: "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=".to_string(),
            blind_index_key: "ZmVkY2JhOTg3NjU0MzIxMGZlZGNiYTk4NzY1NDMyMTA=".to_string(),
            current_encryption_key_version: 1,
            internal_api_keys: vec![],
            resend_api_key: String::new(),
            email_from: String::new(),
            email_from_name: String::new(),
            bunny_storage_zone: String::new(),
            bunny_storage_key: String::new(),
            bunny_cdn_url: String::new(),
            bunny_token_key: String::new(),
            bunny_stream_library_id: String::new(),
            bunny_stream_key: String::new(),
            bunny_stream_webhook_secret: String::new(),
            b2_key_id: String::new(),
            b2_application_key: String::new(),
            b2_bucket_id: String::new(),
            b2_bucket_name: String::new(),
            b2_endpoint: String::new(),
            stripe_secret_key: String::new(),
            stripe_webhook_secret: String::new(),
            stripe_publishable_key: String::new(),
            platform_commission_percent: 20.0,
            min_subscription_price_cents: 500,
            max_subscription_price_cents: 50_000,
            min_withdrawal_amount_cents: 2_000,
            totp_issuer: "Test".to_string(),
            max_video_size_bytes: 1000,
            max_photo_size_bytes: 1000,
            max_audio_size_bytes: 1000,
            json_payload_limit: 1024 * 1024,
            form_payload_limit: 2 * 1024 * 1024,
            csrf_secret_key: "test_csrf_secret_key_for_testing_purposes_only".to_string(),
            refresh_token_hash_salt: "test_refresh_token_salt".to_string(),
            rate_limit_enabled: true,
        }
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

        AppContainer {
            config: Arc::new(mock_app_config()),
            cache,
            users: Arc::new(MockIUserRepository::new()),
            profiles: Arc::new(MockIProfileRepository::new()),
            refresh_tokens: Arc::new(MockIRefreshTokenRepository::new()),
            user_roles: Arc::new(MockIUserRoleRepository::new()),
            roles: Arc::new(MockIRoleRepository::new()),
            domain_audit_logs: Arc::new(MockIAuditLogRepository::new()),
            access_token_blacklist: Arc::new(crate::repositories::access_token_blacklist::AccessTokenBlacklist::new(pool)),
        }
    }
}
