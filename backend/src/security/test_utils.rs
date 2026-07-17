use crate::config::AppConfig;
use crate::config::app_config::Environment;
use crate::config::app_config::JwtSecretKey;

use super::key_manager::KeyManager;

/// Generate a deterministic base64-encoded key using a seeded RNG.
/// This ensures tests are reproducible while still using realistic key formats.
fn generate_deterministic_base64_key(byte_length: usize, seed: u64) -> String {
    use base64::Engine;
    use rand::RngCore;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut bytes = vec![0u8; byte_length];
    rng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::STANDARD.encode(&bytes)
}

/// Generate a deterministic string using a seeded RNG.
fn generate_deterministic_string(length: usize, seed: u64) -> String {
    use rand::Rng;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

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
        db_statement_timeout_secs: 30,
        redis_url: "redis://127.0.0.1:6379".to_string(),
        redis_pool_size: 10,
        jwt_secret: generate_deterministic_string(32, 0x1234567890ABCDEF),
        jwt_secrets: {
            let secret = generate_deterministic_string(32, 0x1234567890ABCDEF);
            let now = chrono::Utc::now().naive_utc();
            vec![JwtSecretKey {
                kid: "test-primary".to_string(),
                secret,
                created_at: now,
                expires_at: None,
            }]
        },
        jwt_public_key: None,
        jwt_access_expiry_secs: 3600,
        jwt_refresh_expiry_secs: 3600,
        master_key: generate_deterministic_base64_key(32, 0xBEEF),
        blind_index_key: generate_deterministic_base64_key(32, 0xCAFE),
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
        json_payload_limit: 1024 * 1024,
        form_payload_limit: 2 * 1024 * 1024,
        csrf_secret_key: generate_deterministic_string(32, 0xABCDEF),
        refresh_token_hash_salt: generate_deterministic_string(16, 0x1234),
        rate_limit_enabled: true,
    }
}

pub fn test_key_manager() -> KeyManager {
    let config = test_config();

    KeyManager::from_base64_keys(&config.master_key, &config.blind_index_key).unwrap()
}
