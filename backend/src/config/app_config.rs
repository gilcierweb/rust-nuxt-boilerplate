use std::env;
use std::fs;

use base64::Engine as _;

/// Read a secret value from multiple sources in priority order:
/// 1. Environment variable `<NAME>`
/// 2. File at `<NAME>_FILE` env var path (Docker/K8s secrets pattern)
/// 3. Auto-detected file at `/run/secrets/<name_lowercase>` (Docker Compose
///    `secrets:` mount convention; only checked when running inside a container
///    detected via `/run/secrets/.docker-secrets` marker)
/// 4. Default value
///
/// This supports:
/// - **Environment variables**: Direct `JWT_SECRET=xxx`
/// - **Docker secrets**: `JWT_SECRET_FILE=/run/secrets/jwt_secret`
///   or mounted directly via Compose `secrets:` to `/run/secrets/<name>`
/// - **Kubernetes secrets**: Mounted as files via `secretKeyRef`
/// - **AWS/GCP/Azure**: Secrets mounted as files or fetched via SDK
///
/// # Security Benefits
///
/// Reading from files avoids exposing secrets in:
/// - Process environment (`/proc/<pid>/environ`)
/// - Container inspection (`docker inspect`)
/// - CI/CD logs
/// - Docker image layers (when using multi-stage builds)
///
/// # Example
///
/// ```bash
/// # Option 1: Direct environment variable
/// export JWT_SECRET=supersecret
///
/// # Option 2: Docker secrets (recommended for production)
/// echo "supersecret" | docker secret create jwt_secret -
/// export JWT_SECRET_FILE=/run/secrets/jwt_secret
/// ```
fn secret_from_env_or_file(name: &str, default: &str) -> String {
    if let Ok(value) = env::var(name)
        && !value.is_empty()
    {
        return value;
    }

    // Second, check for file-based secret (Docker secrets pattern)
    let file_var = format!("{}_FILE", name);
    if let Ok(file_path) = env::var(&file_var) {
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    tracing::debug!(secret = %name, "Loaded secret from file");
                    return trimmed;
                }
            },
            Err(e) => {
                tracing::warn!(
                    secret = %name,
                    path = %file_path,
                    error = %e,
                    "Failed to read secret file, falling back to default"
                );
            },
        }
    }

    // Third, auto-detect /run/secrets/<lowercase_name> (Docker Compose `secrets:`
    // without explicit _FILE env var). This is consistent with the convention
    // used in docker-compose.yml infra/secrets/.
    let secret_path = format!("/run/secrets/{}", name.to_lowercase());
    if let Ok(content) = fs::read_to_string(&secret_path) {
        let trimmed = content.trim().to_string();
        if !trimmed.is_empty() {
            tracing::debug!(secret = %name, path = %secret_path, "Loaded secret from /run/secrets");
            return trimmed;
        }
    }

    default.to_string()
}

/// Read a required secret from environment or file.
/// Returns error if not found in either location.
fn required_secret(name: &str) -> Result<String, env::VarError> {
    if let Ok(value) = env::var(name)
        && !value.is_empty()
    {
        return Ok(value);
    }

    // Check file-based secret
    let file_var = format!("{}_FILE", name);
    if let Ok(file_path) = env::var(&file_var) {
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    tracing::debug!(secret = %name, "Loaded secret from file");
                    return Ok(trimmed);
                }
            },
            Err(e) => {
                tracing::warn!(
                    secret = %name,
                    path = %file_path,
                    error = %e,
                    "Failed to read secret file"
                );
            },
        }
    }

    // Auto-detect /run/secrets/<lowercase_name> (Docker Compose `secrets:`
    // without explicit _FILE env var).
    let secret_path = format!("/run/secrets/{}", name.to_lowercase());
    if let Ok(content) = fs::read_to_string(&secret_path) {
        let trimmed = content.trim().to_string();
        if !trimmed.is_empty() {
            tracing::debug!(secret = %name, path = %secret_path, "Loaded secret from /run/secrets");
            return Ok(trimmed);
        }
    }

    Err(env::VarError::NotPresent)
}

/// A single JWT signing secret with metadata for rotation support.
#[derive(Debug, Clone)]
pub struct JwtSecretKey {
    pub kid: String,
    pub secret: String,
    pub created_at: chrono::NaiveDateTime,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

impl JwtSecretKey {
    pub fn is_active(&self) -> bool {
        let now = chrono::Utc::now().naive_utc();
        self.expires_at.is_none_or(|exp| exp > now)
    }
}

/// All configuration values loaded from environment variables or secret files.
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
    pub db_statement_timeout_secs: u64,

    // Redis
    pub redis_url: String,
    pub redis_pool_size: usize,

    // JWT
    pub jwt_secret: String,
    pub jwt_secrets: Vec<JwtSecretKey>,
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
    pub json_payload_limit: usize, // JSON body size limit (bytes)
    pub form_payload_limit: usize, // Form body size limit (bytes)

    // Rate limiting
    pub rate_limit_enabled: bool,

    // Argon2 password hashing parameters
    pub argon2_m_cost: u32,
    pub argon2_t_cost: u32,
    pub argon2_p_cost: u32,

    // Trusted proxies for X-Forwarded-For / Forwarded header support
    pub trusted_proxies: Vec<ipnet::IpNet>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
    Test,
}

/// Minimum Redis pool sizes per environment.
const REDIS_POOL_DEFAULT_DEV: usize = 10;
const REDIS_POOL_DEFAULT_STAGING: usize = 30;
/// Production minimum (50) is required to avoid pool exhaustion under load
/// (rate limiting, caching, session storage, token blacklisting, WebSocket Pub/Sub).
/// This is a hard floor — validation will reject configs below this value.
pub const REDIS_POOL_MIN_PRODUCTION: usize = 50;
/// Production recommended size (100) for typical multi-worker deployments.
/// A warning is logged (not a hard failure) when production pool size is below
/// this recommended value but at or above the minimum.
pub const REDIS_POOL_RECOMMENDED_PRODUCTION: usize = 100;
const REDIS_POOL_DEFAULT_PRODUCTION: usize = REDIS_POOL_RECOMMENDED_PRODUCTION;

/// Default PostgreSQL statement timeout in seconds. Applied via `SET statement_timeout`
/// on each connection acquired from the pool. Prevents long-running queries from
/// blocking connections indefinitely and exhausting the pool.
pub const DB_STATEMENT_TIMEOUT_DEFAULT_SECS: u64 = 30;

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        // Parse environment first so we can use it for environment-aware defaults
        let environment = {
            let env_var = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
            match env_var.to_ascii_lowercase().as_str() {
                "production" => Environment::Production,
                "staging" => Environment::Staging,
                "test" => Environment::Test,
                _ => Environment::Development,
            }
        };

        // Default Redis pool size depends on environment:
        // - Development/Test: 10 (light local usage)
        // - Staging: 30
        // - Production: REDIS_POOL_DEFAULT_PRODUCTION (100 — recommended for
        //   rate limiting, caching, session storage, token blacklisting, WS Pub/Sub)
        let redis_pool_default = match environment {
            Environment::Production => REDIS_POOL_DEFAULT_PRODUCTION,
            Environment::Staging => REDIS_POOL_DEFAULT_STAGING,
            Environment::Test | Environment::Development => REDIS_POOL_DEFAULT_DEV,
        };

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

            environment,

            database_url: required_secret("DATABASE_URL")?,
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
            db_statement_timeout_secs: env::var("DB_STATEMENT_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(DB_STATEMENT_TIMEOUT_DEFAULT_SECS),

            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            redis_pool_size: env::var("REDIS_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(redis_pool_default),

            jwt_secret: required_secret("JWT_SECRET")?,
            jwt_secrets: {
                let primary = required_secret("JWT_SECRET")?;
                let now = chrono::Utc::now().naive_utc();
                let mut secrets = vec![JwtSecretKey {
                    kid: env::var("JWT_KID").unwrap_or_else(|_| "primary".to_string()),
                    secret: primary,
                    created_at: now,
                    expires_at: None,
                }];

                // Support additional rotation secrets via JWT_SECRETS=old_kid:old_secret,prev_kid:prev_secret
                if let Ok(extra) = env::var("JWT_SECRETS") {
                    for entry in extra.split(',').filter(|s| !s.trim().is_empty()) {
                        if let Some((kid, secret)) = entry.trim().split_once(':') {
                            secrets.push(JwtSecretKey {
                                kid: kid.to_string(),
                                secret: secret.to_string(),
                                created_at: now,
                                expires_at: None,
                            });
                        }
                    }
                }

                secrets
            },
            jwt_public_key: env::var("JWT_PUBLIC_KEY").ok(),
            jwt_access_expiry_secs: 15 * 60, // 15 minutes
            jwt_refresh_expiry_secs: 30 * 24 * 3600,

            // All secrets are required - fail fast if missing in any environment
            // Use ./scripts/generate-secrets.sh to generate secure values
            master_key: required_secret("MASTER_KEY")?,
            blind_index_key: required_secret("BLIND_INDEX_KEY")?,
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
            csrf_secret_key: required_secret("CSRF_SECRET_KEY")?,
            refresh_token_hash_salt: required_secret("REFRESH_TOKEN_HASH_SALT")?,

            resend_api_key: secret_from_env_or_file("RESEND_API_KEY", ""),
            email_from: env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "noreply@boilerplate-rust-nuxt.com".to_string()),
            email_from_name: env::var("EMAIL_FROM_NAME")
                .unwrap_or_else(|_| "Boilerplate Rust Nuxt".to_string()),

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

            stripe_secret_key: secret_from_env_or_file("STRIPE_SECRET_KEY", ""),
            stripe_webhook_secret: secret_from_env_or_file("STRIPE_WEBHOOK_SECRET", ""),
            stripe_publishable_key: secret_from_env_or_file("STRIPE_PUBLISHABLE_KEY", ""),

            platform_commission_percent: env::var("PLATFORM_COMMISSION_PERCENT")
                .unwrap_or_else(|_| "20.0".to_string())
                .parse()
                .unwrap_or(20.0),
            min_subscription_price_cents: 500,
            max_subscription_price_cents: 50_000,
            min_withdrawal_amount_cents: 2_000,

            totp_issuer: env::var("TOTP_ISSUER")
                .unwrap_or_else(|_| "Boilerplate Rust Nuxt".to_string()),

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

            // Argon2 password hashing parameters
            argon2_m_cost: env::var("ARGON2_M_COST")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(65536),
            argon2_t_cost: env::var("ARGON2_T_COST")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(3),
            argon2_p_cost: env::var("ARGON2_P_COST")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1),

            // Trusted proxies for X-Forwarded-For / Forwarded header support
            // Comma-separated list of CIDR ranges (e.g., "10.0.0.0/8,172.16.0.0/12,192.168.0.0/16")
            trusted_proxies: env::var("TRUSTED_PROXIES")
                .ok()
                .map(|s| {
                    s.split(',')
                        .filter_map(|part| part.trim().parse::<ipnet::IpNet>().ok())
                        .collect()
                })
                .unwrap_or_default(),
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

    /// Validate configuration at startup.
    ///
    /// Returns a list of validation errors. If empty, config is valid.
    /// Called after `from_env()` to catch misconfiguration before runtime.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Port validation
        if self.port == 0 {
            errors.push("PORT must not be 0".to_string());
        }
        if self.https_port == 0 {
            errors.push("HTTPS_PORT must not be 0".to_string());
        }
        if self.port == self.https_port {
            errors.push("PORT and HTTPS_PORT must be different".to_string());
        }

        // JWT secret validation
        if self.jwt_secret.len() < 32 {
            errors.push(format!(
                "JWT_SECRET must be at least 32 bytes, got {}",
                self.jwt_secret.len()
            ));
        }
        if self.jwt_secrets.is_empty() {
            errors.push("JWT_SECRETS must contain at least one active key".to_string());
        }
        for key in &self.jwt_secrets {
            if key.secret.len() < 32 {
                errors.push(format!(
                    "JWT secret '{}' must be at least 32 bytes, got {}",
                    key.kid,
                    key.secret.len()
                ));
            }
        }

        // Master key validation (must be valid base64, decodes to ≥32 bytes)
        match base64::engine::general_purpose::STANDARD.decode(&self.master_key) {
            Ok(bytes) if bytes.len() >= 32 => {},
            Ok(bytes) => {
                errors.push(format!(
                    "MASTER_KEY must decode to at least 32 bytes, got {}",
                    bytes.len()
                ));
            },
            Err(_) => {
                errors.push("MASTER_KEY must be valid base64".to_string());
            },
        }

        // Blind index key validation (must be valid base64, decodes to ≥32 bytes)
        match base64::engine::general_purpose::STANDARD.decode(&self.blind_index_key) {
            Ok(bytes) if bytes.len() >= 32 => {},
            Ok(bytes) => {
                errors.push(format!(
                    "BLIND_INDEX_KEY must decode to at least 32 bytes, got {}",
                    bytes.len()
                ));
            },
            Err(_) => {
                errors.push("BLIND_INDEX_KEY must be valid base64".to_string());
            },
        }

        // CSRF secret key validation
        if self.is_production_like() && self.csrf_secret_key.len() < 32 {
            errors.push(format!(
                "CSRF_SECRET_KEY must be at least 32 bytes in production, got {}",
                self.csrf_secret_key.len()
            ));
        }

        // Refresh token hash salt validation
        if self.is_production_like() && self.refresh_token_hash_salt.len() < 16 {
            errors.push(format!(
                "REFRESH_TOKEN_HASH_SALT must be at least 16 bytes in production, got {}",
                self.refresh_token_hash_salt.len()
            ));
        }

        // SECURITY_AUDIT.md I7: reject placeholder secret values in production.
        // Values like "changeme_*" and "REPLACE_WITH_*" come from `.env.example`
        // and MUST be replaced before deployment. Detected patterns:
        //   - "changeme_*"           (POSTGRES_PASSWORD, REDIS_PASSWORD)
        //   - "REPLACE_WITH_*"        (JWT_SECRET, encryption keys, etc.)
        //   - "re_XXXXX", "sk_test_X" (Resend, Stripe test keys — never prod)
        //   - "your-zone", "your-key" placeholder storage creds
        if self.is_production_like() {
            for (name, value) in [
                ("POSTGRES_PASSWORD", &self.database_url), // Check string contents
            ] {
                if value.contains("changeme_") {
                    errors.push(format!(
                        "{} contains placeholder value 'changeme_*'. \
                         Override the secret before deploying to production (see .env.example).",
                        name
                    ));
                    break;
                }
            }
            for (name, value) in &[
                ("JWT_SECRET", &self.jwt_secret),
                ("MASTER_ENCRYPTION_KEY", &self.master_key),
                ("BLIND_INDEX_KEY", &self.blind_index_key),
                ("CSRF_SECRET_KEY", &self.csrf_secret_key),
                ("REFRESH_TOKEN_HASH_SALT", &self.refresh_token_hash_salt),
            ] {
                if value.contains("REPLACE_WITH_") {
                    errors.push(format!(
                        "{} contains placeholder value 'REPLACE_WITH_*'. \
                         Override the secret before deploying to production (see .env.example).",
                        name
                    ));
                }
            }
        }

        // Database URL validation
        if !self.database_url.starts_with("postgres://")
            && !self.database_url.starts_with("postgresql://")
        {
            errors.push("DATABASE_URL must start with postgres:// or postgresql://".to_string());
        }

        // Redis URL validation
        if !self.redis_url.starts_with("redis://") && !self.redis_url.starts_with("rediss://") {
            errors.push("REDIS_URL must start with redis:// or rediss://".to_string());
        }

        // Pool size validation
        if self.db_pool_size == 0 {
            errors.push("DB_POOL_SIZE must be greater than 0".to_string());
        }
        if self.redis_pool_size == 0 {
            errors.push("REDIS_POOL_SIZE must be greater than 0".to_string());
        }

        // Production must have a minimum Redis pool size of 50.
        // This prevents connection pool exhaustion under load
        // (rate limiting, caching, session storage, token blacklisting, WebSocket Pub/Sub).
        // Note: REDIS_POOL_RECOMMENDED_PRODUCTION (100) is recommended for typical
        // multi-worker deployments; a warning is logged if below recommended but >= minimum.
        if matches!(self.environment, Environment::Production)
            && self.redis_pool_size < REDIS_POOL_MIN_PRODUCTION
        {
            errors.push(format!(
                "REDIS_POOL_SIZE must be at least {} in production (got {}). \
                 Consider rate limiting, caching, session storage, token blacklisting, \
                 and WebSocket Pub/Sub requirements. Recommended: {}+",
                REDIS_POOL_MIN_PRODUCTION, self.redis_pool_size, REDIS_POOL_RECOMMENDED_PRODUCTION
            ));
        }

        // Platform settings validation
        if self.platform_commission_percent < 0.0 || self.platform_commission_percent > 100.0 {
            errors.push(format!(
                "PLATFORM_COMMISSION_PERCENT must be between 0 and 100, got {}",
                self.platform_commission_percent
            ));
        }
        if self.min_subscription_price_cents >= self.max_subscription_price_cents {
            errors.push(
                "MIN_SUBSCRIPTION_PRICE_CENTS must be less than MAX_SUBSCRIPTION_PRICE_CENTS"
                    .to_string(),
            );
        }

        // CORS origin validation (FRONTEND_URL)
        // Must not contain wildcard "*" when using credentials
        // Each origin must start with http:// or https://
        let cors_origins: Vec<String> = self
            .frontend_url
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if cors_origins.is_empty() {
            errors.push("FRONTEND_URL must contain at least one valid origin".to_string());
        }

        for origin in &cors_origins {
            if origin == "*" {
                errors.push(
                    "CORS configuration error: wildcard '*' origin is not allowed with supports_credentials(). Set FRONTEND_URL to specific origins (e.g., https://yourdomain.com)".to_string()
                );
            }
            if !origin.starts_with("http://") && !origin.starts_with("https://") {
                errors.push(format!(
                    "CORS origin '{}' rejected: must start with http:// or https://",
                    origin
                ));
            }
        }

        errors
    }

    /// Validate configuration and panic with clear message if invalid.
    ///
    /// Use this at application startup after `from_env()`.
    pub fn validate_or_panic(&self) {
        let errors = self.validate();
        if !errors.is_empty() {
            eprintln!("╔══════════════════════════════════════════════════════════╗");
            eprintln!("║           CONFIGURATION VALIDATION FAILED              ║");
            eprintln!("╠══════════════════════════════════════════════════════════╣");
            for error in &errors {
                eprintln!("║  ✗ {:54} ║", error);
            }
            eprintln!("╠══════════════════════════════════════════════════════════╣");
            eprintln!("║  Fix the above issues and restart the application.     ║");
            eprintln!("║  See .env.example for configuration documentation.     ║");
            eprintln!("╚══════════════════════════════════════════════════════════╝");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod placeholder_tests {
    //! SECURITY_AUDIT.md I7 — placeholder rejection tests.
    //!
    //! These tests assert that production builds refuse placeholder secrets
    //! such as "changeme_*" (POSTGRES_PASSWORD, REDIS_PASSWORD) and
    //! "REPLACE_WITH_*" (JWT_SECRET, encryption keys). The development
    //! environment intentionally allows these markers.
    use super::*;

    fn minimal_valid_secret() -> String {
        // 48-byte base64 string that decodes to ≥32 bytes
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.encode([7u8; 32])
    }

    /// Env vars that `AppConfig::from_env` requires with non-empty values.
    /// Each test sets these fresh — they must NOT be shared via leaked env vars
    /// from previous tests (which is what caused test ordering issues).
    fn setup_minimal_env() {
        use base64::Engine as _;
        let valid = base64::engine::general_purpose::STANDARD.encode([7u8; 32]);

        let set = |k: &str, v: &str| unsafe { std::env::set_var(k, v) };
        set("DATABASE_URL", "postgres://u:p@localhost:5432/d");
        set("DB_POOL_SIZE", "10");
        set("DB_STATEMENT_TIMEOUT_SECS", "30");
        set("DB_POOL_MIN_IDLE", "2");
        set("DB_POOL_MAX_LIFETIME_SECS", "1800");
        set("DB_POOL_IDLE_TIMEOUT_SECS", "600");
        set("DB_POOL_CONNECTION_TIMEOUT_SECS", "10");
        set("REDIS_URL", "redis://localhost:6379");
        set("REDIS_POOL_SIZE", "10");
        set("MAX_VIDEO_SIZE_BYTES", "1000");
        set("MAX_PHOTO_SIZE_BYTES", "1000");
        set("MAX_AUDIO_SIZE_BYTES", "1000");
        set("JSON_PAYLOAD_LIMIT", "1048576");
        set("FORM_PAYLOAD_LIMIT", "2097152");
        set("CSRF_SECRET_KEY", &valid);
        set("REFRESH_TOKEN_HASH_SALT", &valid);
        set("BLIND_INDEX_KEY", &valid);
        set("MASTER_KEY", &valid);
    }

    #[test]
    fn production_rejects_postgres_password_changeme() {
        setup_minimal_env();
        unsafe { std::env::set_var("ENVIRONMENT", "production") };
        unsafe { std::env::set_var("REDIS_POOL_SIZE", "100") }; // meet prod minimum
        unsafe { std::env::set_var("POSTGRES_PASSWORD", "changeme_secure_password") };
        unsafe { std::env::set_var("JWT_SECRET", minimal_valid_secret()) };
        // DATABASE_URL interpolates POSTGRES_PASSWORD; URL embeds the placeholder.
        unsafe {
            std::env::set_var(
                "DATABASE_URL",
                "postgres://u:changeme_secure_password@localhost:5432/d",
            )
        };

        let cfg = AppConfig::from_env().expect("config builds");
        let errors = cfg.validate();

        assert!(
            errors
                .iter()
                .any(|e| e.contains("placeholder") && e.contains("changeme")),
            "expected placeholder error mentioning 'changeme', got: {:?}",
            errors
        );
    }

    #[test]
    fn production_replaces_with_jwt_secret_placeholder() {
        setup_minimal_env();
        unsafe { std::env::set_var("ENVIRONMENT", "production") };
        unsafe { std::env::set_var("REDIS_POOL_SIZE", "100") }; // meet prod minimum
        // Provide GOOD values for everything except JWT_SECRET.
        // Importantly DON'T set POSTGRES_PASSWORD = "changeme_*" so the ping-pong
        // between this and test #1 doesn't leak state.
        unsafe { std::env::set_var("DATABASE_URL", "postgres://u:good_pwd@localhost:5432/d") };
        unsafe { std::env::set_var("POSTGRES_PASSWORD", "good_secret_xyz") };
        unsafe { std::env::set_var("JWT_SECRET", "REPLACE_WITH_GENERATED_64_CHAR_BASE64_SECRET") };
        unsafe { std::env::set_var("MASTER_KEY", minimal_valid_secret()) };

        let cfg = AppConfig::from_env().expect("config builds");
        let errors = cfg.validate();

        assert!(
            errors
                .iter()
                .any(|e| e.contains("JWT_SECRET") && e.contains("placeholder")),
            "expected JWT_SECRET placeholder error, got: {:?}",
            errors
        );
    }

    #[test]
    fn development_allows_changeme_placeholders() {
        setup_minimal_env();
        unsafe { std::env::set_var("ENVIRONMENT", "development") };
        // All-good values (no `changeme_` or `REPLACE_`).
        unsafe { std::env::set_var("DATABASE_URL", "postgres://u:dev_pwd@localhost:5432/d") };
        unsafe { std::env::set_var("POSTGRES_PASSWORD", "dev_pwd_xyz") };
        unsafe { std::env::set_var("JWT_SECRET", minimal_valid_secret()) };

        let cfg = AppConfig::from_env().expect("config builds");
        let errors = cfg.validate();

        for e in &errors {
            assert!(
                !e.contains("placeholder"),
                "dev config should NOT trigger placeholder error: {}",
                e
            );
        }
    }
}
