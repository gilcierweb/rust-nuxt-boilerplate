use std::env;

/// All configuration values loaded from environment variables.
/// In production, use Docker secrets or a secrets manager like Vault.
#[derive(Debug, Clone)]
pub struct AppConfig {
    // Server
    pub host: String,
    pub port: u16,
    pub https_port: u16,
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub frontend_url: String,
    pub environment: Environment,

    // Database
    pub database_url: String,
    pub db_pool_size: u32,
    pub db_pool_min_idle: Option<u32>,
    pub db_pool_max_lifetime_secs: Option<u64>,
    pub db_pool_idle_timeout_secs: Option<u64>,
    pub db_pool_connection_timeout_secs: u64,

    // Redis
    pub redis_url: String,
    pub redis_pool_size: usize,

    // JWT
    pub jwt_secret: String,
    pub jwt_public_key: Option<String>,
    pub jwt_access_expiry_secs: i64,  // 15 minutes
    pub jwt_refresh_expiry_secs: i64, // 30 days

    // Security
    pub master_key: String,
    pub blind_index_key: String,
    pub current_encryption_key_version: u32,
    pub internal_api_keys: Vec<String>,
    pub csrf_secret_key: String,
    pub refresh_token_hash_salt: String,

    // Email (Resend)
    pub resend_api_key: String,
    pub email_from: String,
    pub email_from_name: String,

    // Bunny.net CDN / Storage
    pub bunny_storage_zone: String,
    pub bunny_storage_key: String,
    pub bunny_cdn_url: String,
    pub bunny_token_key: String, // for URL signing

    // Bunny.net Stream
    pub bunny_stream_library_id: String,
    pub bunny_stream_key: String,
    pub bunny_stream_webhook_secret: String,

    // Backblaze B2
    pub b2_key_id: String,
    pub b2_application_key: String,
    pub b2_bucket_id: String,
    pub b2_bucket_name: String,
    pub b2_endpoint: String,

    // Stripe
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub stripe_publishable_key: String,

    // Platform settings
    pub platform_commission_percent: f64,  // e.g. 20.0
    pub min_subscription_price_cents: i64, // e.g. 500 = $5.00
    pub max_subscription_price_cents: i64, // e.g. 50000 = $500.00
    pub min_withdrawal_amount_cents: i64,  // e.g. 2000 = $20.00

    // TOTP (2FA)
    pub totp_issuer: String,

    // File upload limits
    pub max_video_size_bytes: u64, // 10 GB
    pub max_photo_size_bytes: u64, // 50 MB
    pub max_audio_size_bytes: u64, // 500 MB

    // Request payload limits
    pub json_payload_limit: usize,    // JSON body size limit (bytes)
    pub form_payload_limit: usize,    // Form body size limit (bytes)

    // Rate limiting
    pub rate_limit_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
    Test,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            https_port: env::var("HTTPS_PORT")
                .unwrap_or_else(|_| "8443".to_string())
                .parse()
                .unwrap_or(8443),
            tls_cert_path: env::var("TLS_CERT_PATH").unwrap_or_else(|_| "cert.pem".to_string()),
            tls_key_path: env::var("TLS_KEY_PATH").unwrap_or_else(|_| "key.pem".to_string()),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),

            // Parse environment first to check for required secrets
            environment: {
                let env_var = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
                match env_var.to_ascii_lowercase().as_str() {
                    "production" => Environment::Production,
                    "staging" => Environment::Staging,
                    "test" => Environment::Test,
                    _ => Environment::Development,
                }
            },

            database_url: env::var("DATABASE_URL")?,
            db_pool_size: env::var("DB_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            db_pool_min_idle: env::var("DB_POOL_MIN_IDLE")
                .ok()
                .and_then(|s| s.parse::<u32>().ok()),
            db_pool_max_lifetime_secs: env::var("DB_POOL_MAX_LIFETIME_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok()),
            db_pool_idle_timeout_secs: env::var("DB_POOL_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok()),
            db_pool_connection_timeout_secs: env::var("DB_POOL_CONNECTION_TIMEOUT_SECS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),

            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            redis_pool_size: env::var("REDIS_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),

            jwt_secret: env::var("JWT_SECRET")?,
            jwt_public_key: env::var("JWT_PUBLIC_KEY").ok(),
            jwt_access_expiry_secs: 2 * 60 * 60,
            jwt_refresh_expiry_secs: 30 * 24 * 3600,

            // Production requires secrets - fail fast if missing
            master_key: {
                let key = env::var("MASTER_KEY");
                let env_check = env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string())
                    .to_ascii_lowercase();
                let requires_strict_secrets =
                    matches!(env_check.as_str(), "production" | "staging");
                if requires_strict_secrets && key.is_err() {
                    panic!("MASTER_KEY must be set in staging/production");
                }
                key.unwrap_or_else(|_| "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY=".to_string())
            },
            blind_index_key: {
                let key = env::var("BLIND_INDEX_KEY");
                let env_check = env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string())
                    .to_ascii_lowercase();
                let requires_strict_secrets =
                    matches!(env_check.as_str(), "production" | "staging");
                if requires_strict_secrets && key.is_err() {
                    panic!("BLIND_INDEX_KEY must be set in staging/production");
                }
                key.unwrap_or_else(|_| "ZmVkY2JhOTg3NjU0MzIxMGZlZGNiYTk4NzY1NDMyMTA=".to_string())
            },
            current_encryption_key_version: env::var("CURRENT_ENCRYPTION_KEY_VERSION")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            internal_api_keys: env::var("INTERNAL_API_KEYS")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect(),
            csrf_secret_key: env::var("CSRF_SECRET_KEY")
                .unwrap_or_else(|_| "default_csrf_secret_key_change_in_production".to_string()),
            refresh_token_hash_salt: {
                let salt = env::var("REFRESH_TOKEN_HASH_SALT");
                let env_check = env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string())
                    .to_ascii_lowercase();
                let requires_strict_secrets =
                    matches!(env_check.as_str(), "production" | "staging");
                if requires_strict_secrets && salt.is_err() {
                    panic!("REFRESH_TOKEN_HASH_SALT must be set in staging/production");
                }
                salt.unwrap_or_else(|_| "refresh_token_salt_change_in_production".to_string())
            },

            resend_api_key: env::var("RESEND_API_KEY").unwrap_or_default(),
            email_from: env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "noreply@boilerplate-rust-nuxt.com".to_string()),
            email_from_name: env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Boilerplate Rust Nuxt".to_string()),

            bunny_storage_zone: env::var("BUNNY_STORAGE_ZONE").unwrap_or_default(),
            bunny_storage_key: env::var("BUNNY_STORAGE_KEY").unwrap_or_default(),
            bunny_cdn_url: env::var("BUNNY_CDN_URL")
                .unwrap_or_else(|_| "https://cdn.boilerplate-rust-nuxt.com".to_string()),
            bunny_token_key: env::var("BUNNY_TOKEN_KEY").unwrap_or_default(),

            bunny_stream_library_id: env::var("BUNNY_STREAM_LIBRARY_ID").unwrap_or_default(),
            bunny_stream_key: env::var("BUNNY_STREAM_KEY").unwrap_or_default(),
            bunny_stream_webhook_secret: env::var("BUNNY_STREAM_WEBHOOK_SECRET")
                .unwrap_or_default(),

            b2_key_id: env::var("B2_KEY_ID").unwrap_or_default(),
            b2_application_key: env::var("B2_APPLICATION_KEY").unwrap_or_default(),
            b2_bucket_id: env::var("B2_BUCKET_ID").unwrap_or_default(),
            b2_bucket_name: env::var("B2_BUCKET_NAME").unwrap_or_default(),
            b2_endpoint: env::var("B2_ENDPOINT")
                .unwrap_or_else(|_| "https://s3.us-west-004.backblazeb2.com".to_string()),

            stripe_secret_key: env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
            stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default(),
            stripe_publishable_key: env::var("STRIPE_PUBLISHABLE_KEY").unwrap_or_default(),

            platform_commission_percent: env::var("PLATFORM_COMMISSION_PERCENT")
                .unwrap_or_else(|_| "20.0".to_string())
                .parse()
                .unwrap_or(20.0),
            min_subscription_price_cents: 500,
            max_subscription_price_cents: 50_000,
            min_withdrawal_amount_cents: 2_000,

            totp_issuer: env::var("TOTP_ISSUER").unwrap_or_else(|_| "Boilerplate Rust Nuxt".to_string()),

            max_video_size_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            max_photo_size_bytes: 50 * 1024 * 1024,        // 50 MB
            max_audio_size_bytes: 500 * 1024 * 1024,       // 500 MB

            // Default JSON payload limit is 16 MB - sufficient for file upload metadata
            // Form payload limit is 20 MB for multipart form-data with metadata
            json_payload_limit: env::var("JSON_PAYLOAD_LIMIT")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(16 * 1024 * 1024),
            form_payload_limit: env::var("FORM_PAYLOAD_LIMIT")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(20 * 1024 * 1024),

            // Rate limiting - enabled by default in all environments
            // Can be disabled via RATE_LIMIT_ENABLED=false
            rate_limit_enabled: env::var("RATE_LIMIT_ENABLED")
                .ok()
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(true),
        })
    }

    pub fn is_production(&self) -> bool {
        self.is_production_like()
    }

    pub fn is_production_like(&self) -> bool {
        matches!(
            self.environment,
            Environment::Staging | Environment::Production
        )
    }
}
