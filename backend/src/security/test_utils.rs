use crate::config::AppConfig;
use crate::config::app_config::Environment;

use super::key_manager::KeyManager;

pub fn test_config() -> AppConfig {
    AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        https_port: 8443,
        tls_cert_path: "cert.pem".to_string(),
        tls_key_path: "key.pem".to_string(),
        frontend_url: "http://localhost:3000".to_string(),
        environment: Environment::Test,
        database_url: "postgres://localhost/test".to_string(),
        db_pool_size: 1,
        db_pool_min_idle: Some(1),
        db_pool_max_lifetime_secs: Some(1800),
        db_pool_idle_timeout_secs: Some(600),
        db_pool_connection_timeout_secs: 10,
        redis_url: "redis://127.0.0.1:6379".to_string(),
        redis_pool_size: 10,
        jwt_secret: "secret".to_string(),
        jwt_public_key: None,
        jwt_access_expiry_secs: 3600,
        jwt_refresh_expiry_secs: 3600,
        master_key: "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=".to_string(),
        blind_index_key: "ZmVkY2JhOTg3NjU0MzIxMGZlZGNiYTk4NzY1NDMyMTA=".to_string(),
        current_encryption_key_version: 1,
        internal_api_keys: vec![],
        resend_api_key: "".to_string(),
        email_from: "".to_string(),
        email_from_name: "".to_string(),
        bunny_storage_zone: "".to_string(),
        bunny_storage_key: "".to_string(),
        bunny_cdn_url: "".to_string(),
        bunny_token_key: "".to_string(),
        bunny_stream_library_id: "".to_string(),
        bunny_stream_key: "".to_string(),
        bunny_stream_webhook_secret: "".to_string(),
        b2_key_id: "".to_string(),
        b2_application_key: "".to_string(),
        b2_bucket_id: "".to_string(),
        b2_bucket_name: "".to_string(),
        b2_endpoint: "".to_string(),
        stripe_secret_key: "".to_string(),
        stripe_webhook_secret: "".to_string(),
        stripe_publishable_key: "".to_string(),
        platform_commission_percent: 20.0,
        min_subscription_price_cents: 500,
        max_subscription_price_cents: 50_000,
        min_withdrawal_amount_cents: 2_000,
        totp_issuer: "Test".to_string(),
        max_video_size_bytes: 1024,
        max_photo_size_bytes: 1024,
        max_audio_size_bytes: 1024,
        csrf_secret_key: "test_csrf_secret_key_for_testing_purposes_only".to_string(),
    }
}

pub fn test_key_manager() -> KeyManager {
    let config = test_config();

    KeyManager::from_base64_keys(&config.master_key, &config.blind_index_key).unwrap()
}
