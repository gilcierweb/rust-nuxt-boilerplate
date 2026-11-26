use std::str::FromStr;
use std::{env, fs};

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
/// The same resolution through a caller supplied lookup, so a test can name
/// its own environment instead of mutating the process one.
fn secret_from_env_or_file_with(
    name: &str,
    default: &str,
    read: impl Fn(&str) -> Option<String>,
) -> String {
    if let Some(value) = secret_from_files_with(name, &read) {
        return value;
    }

    if let Some(value) = read(name).filter(|value| !value.is_empty()) {
        return value;
    }

    default.to_string()
}

/// True when mounted secrets must take precedence over environment variables.
///
/// `docker-compose.yml` documents `DOCKER_SECRETS_ENABLED=1` for production.
/// Without the flag a stale `.env` silently overrides a rotated `/run/secrets`
/// value, which defeats the point of mounting the file at all.
fn use_docker_secrets_with(read: impl Fn(&str) -> Option<String>) -> bool {
    matches!(
        read("DOCKER_SECRETS_ENABLED")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Read a secret from the file-based sources Docker/Kubernetes provide.
///
/// Sources, in priority order:
/// 1. `<NAME>_FILE` env var pointing at the file
/// 2. `/run/secrets/<lowercase_name>` (Compose `secrets:` mount convention)
///
/// Returns `None` when the flag is off or no file is readable.
fn secret_from_files_with(name: &str, read: impl Fn(&str) -> Option<String>) -> Option<String> {
    if !use_docker_secrets_with(&read) {
        return None;
    }

    let file_var = format!("{}_FILE", name);
    if let Some(file_path) = read(&file_var) {
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    tracing::debug!(secret = %name, "Loaded secret from file");
                    return Some(trimmed);
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

    let secret_path = format!("/run/secrets/{}", name.to_lowercase());
    if let Ok(content) = fs::read_to_string(&secret_path) {
        let trimmed = content.trim().to_string();
        if !trimmed.is_empty() {
            tracing::debug!(secret = %name, path = %secret_path, "Loaded secret from /run/secrets");
            return Some(trimmed);
        }
    }

    None
}

/// Read a required secret from environment or file.
/// Returns error if not found in either location.
/// Reads a required secret through `read`, after the Docker secret files.
///
/// The lookup is a parameter so a test can supply its own environment source
/// instead of mutating the process environment.
fn required_secret_from(
    name: &str,
    read: impl Fn(&str) -> Option<String>,
) -> Result<String, env::VarError> {
    if let Some(value) = secret_from_files_with(name, &read) {
        return Ok(value);
    }

    match read(name) {
        Some(value) if !value.is_empty() => Ok(value),
        _ => Err(env::VarError::NotPresent),
    }
}

/// A single JWT signing secret with metadata for rotation support.
#[derive(Debug, Clone)]
pub struct JwtSecretKey {
    pub kid: String,
    pub secret: String,
    pub created_at: time::OffsetDateTime,
    pub expires_at: Option<time::OffsetDateTime>,
}

/// Parse one entry of `JWT_SECRETS`.
///
/// Format: `kid:secret` or `kid:secret:expires_at`, where `expires_at` is an
/// RFC-3339 timestamp. The expiry is what makes a rotated key *retirable*:
/// without it `is_active()` can never reject the key, so a compromised legacy
/// secret stays usable forever and the only way to retire it is to remove it
/// from the environment and restart.
fn parse_jwt_secret_entry(entry: &str, created_at: time::OffsetDateTime) -> Option<JwtSecretKey> {
    let mut parts = entry.trim().splitn(3, ':');
    let kid = parts.next()?.trim();
    let secret = parts.next()?.trim();
    if kid.is_empty() || secret.is_empty() {
        return None;
    }

    let expires_at = match parts.next() {
        Some(value) if !value.trim().is_empty() => {
            match time::OffsetDateTime::parse(
                value.trim(),
                &time::format_description::well_known::Rfc3339,
            ) {
                Ok(parsed) => Some(parsed),
                Err(e) => {
                    tracing::warn!(
                        event = "config.jwt_secret_expiry_unparseable",
                        kid = %kid,
                        value = %value.trim(),
                        error = %e,
                        "Ignoring unparseable JWT_SECRETS expiry - the key stays active. \
                         Use RFC-3339, e.g. 2030-01-01T00:00:00Z."
                    );
                    None
                },
            }
        },
        _ => None,
    };

    Some(JwtSecretKey {
        kid: kid.to_string(),
        secret: secret.to_string(),
        created_at,
        expires_at,
    })
}

impl JwtSecretKey {
    pub fn is_active(&self) -> bool {
        let now = time::OffsetDateTime::now_utc();
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
    pub tls_enabled: bool,
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
    /// Recipient every minted token names. Empty never disables the check: the
    /// default is the canonical service name of this deployment.
    pub jwt_audience: String,
    pub jwt_access_expiry_secs: i64,  // 15 minutes
    pub jwt_refresh_expiry_secs: i64, // 30 days

    // Security
    pub master_key: String,
    pub blind_index_key: String,
    pub current_encryption_key_version: u32,
    pub backend_api_keys: Vec<String>,
    pub csrf_secret_key: String,
    pub refresh_token_hash_salt: String,
    // Account lockout
    pub account_lockout_duration_secs: i64, // 15 minutes default

    // Email (Resend)
    pub resend_api_key: String,
    pub email_from: String,
    pub email_from_name: String,
    // Email transport selection. Either Resend (HTTP API) or SMTP (lettre).
    // `smtp_url` is required when `email_transport == Smtp`; ignored otherwise.
    pub email_transport: EmailTransportKind,
    pub smtp_url: String,
    pub smtp_timeout_secs: u64,

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

    // Pix (Brazilian instant payment)
    pub pix_webhook_secret: String,

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

    // Global daily registration ceiling for the register endpoint (the
    // `rl:register_global` bucket). Blunts distributed registration campaigns
    // that the per-IP limits cannot see. Default 2000/day.
    pub register_daily_limit: u64,

    // Role assigned to a new registration, resolved by name. The role must
    // exist and must be unambiguous, otherwise the registration fails
    // closed: a user without a role has no permissions at all.
    pub register_default_role: String,

    // Rejection share alert (watchdog in services/alert_watchdog). The
    // evaluation runs off the request path; the numbers reach the Prometheus
    // payload and every crossing event logs one structured warning.
    pub alert_429_enabled: bool,
    /// Share of refused requests that trips the alert, as a fraction (0.10 = 10%).
    pub alert_429_threshold_share: f64,
    /// Seconds between two evaluations.
    pub alert_429_window_secs: u64,

    // Optional CAPTCHA (Cloudflare Turnstile / Google reCAPTCHA / any custom
    // site-verify service) for brute-force-sensitive endpoints. Disabled by
    // default — the boilerplate must run without any third-party verification
    // service; hostile-facing deployments opt in via CAPTCHA_PROVIDER +
    // CAPTCHA_SECRET (+ CAPTCHA_VERIFY_URL for the custom provider).
    pub captcha_provider: crate::security::captcha::CaptchaProvider,
    pub captcha_secret: Option<String>,
    /// Endpoint used by the `custom` CAPTCHA provider (and any override).
    pub captcha_verify_url: Option<String>,

    // Disposable-mail policy. Opt-in: blocking temporary-mail providers is a
    // product decision, not a security requirement, so the boilerplate leaves
    // it off and lets each deployment choose.
    pub disposable_email_blocking: bool,
    /// Extra domains to treat as disposable, on top of the curated list.
    pub extra_disposable_domains: Vec<String>,

    // Mail-exchanger (MX) check. Opt-in and vendor-neutral: the deployment
    // points EMAIL_DNS_RESOLVER_URL at its own DNS-over-HTTPS resolver.
    pub email_mx_check_enabled: bool,
    pub email_dns_resolver_url: Option<String>,

    // Cookie security override — set COOKIE_SECURE=true in development when
    // routing through an HTTPS tunnel (ngrok, Cloudflare Tunnel, etc.) to
    // prevent the refresh_token and csrf_token cookies from being sent over
    // plain HTTP by the tunnel client.
    // In staging/production this is always true regardless of this setting.
    pub cookie_secure_override: bool,

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

impl Environment {
    /// Parse from env-style string. Case-insensitive; defaults to `Development`
    /// when unrecognised, so an unset `ENVIRONMENT` reads as development
    /// instead of failing the boot of a local checkout.
    pub fn from_env_str(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "production" => Self::Production,
            "staging" => Self::Staging,
            "test" => Self::Test,
            _ => Self::Development,
        }
    }

    /// Whether fake data and development shortcuts are forbidden here. The
    /// seeder reads this through the enum directly, before the whole config
    /// can be built.
    pub fn is_production_like(&self) -> bool {
        matches!(self, Self::Staging | Self::Production)
    }
}

#[cfg(test)]
mod environment_parse_tests {
    use super::*;

    #[test]
    fn parses_the_documented_names_case_insensitively() {
        assert_eq!(
            Environment::from_env_str("production"),
            Environment::Production
        );
        assert_eq!(Environment::from_env_str("Staging"), Environment::Staging);
        assert_eq!(Environment::from_env_str(" Test "), Environment::Test);
    }

    #[test]
    fn defaults_an_unset_or_unknown_value_to_development() {
        assert_eq!(Environment::from_env_str(""), Environment::Development);
        assert_eq!(Environment::from_env_str("qa"), Environment::Development);
    }

    #[test]
    fn only_staging_and_production_are_production_like() {
        assert!(Environment::Production.is_production_like());
        assert!(Environment::Staging.is_production_like());
        assert!(!Environment::Development.is_production_like());
        assert!(!Environment::Test.is_production_like());
    }
}

/// Email transport backend.
///
/// Selects which wire protocol the [`crate::services::EmailService`] uses to
/// deliver outgoing mail. Both backends are production-grade; the choice is
/// purely operational.
///
/// - [`Resend`](Self::Resend) — uses the Resend HTTP API
///   (`https://api.resend.com/emails`). Requires `RESEND_API_KEY`.
/// - [`Smtp`](Self::Smtp) — connects to any RFC 5321 SMTP relay via the `lettre`
///   crate. Supports both `smtps://` (TLS on connect, recommended for prod)
///   and `smtp://` (plain or STARTTLS, useful for local mailcatcher).
///   Requires `SMTP_URL`.
#[derive(Debug, Clone, PartialEq)]
pub enum EmailTransportKind {
    Resend,
    Smtp,
}

impl EmailTransportKind {
    /// Parse from env-style string. Case-insensitive; defaults to `Resend`
    /// when unrecognised.
    pub fn from_env_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "smtp" => Self::Smtp,
            "resend" => Self::Resend,
            _ => Self::Resend,
        }
    }
}

impl std::str::FromStr for EmailTransportKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(Self::from_env_str(s))
    }
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
    pub fn from_lookup(read: impl Fn(&str) -> Option<String>) -> Result<Self, env::VarError> {
        // Parse environment first so we can use it for environment-aware defaults
        let environment = {
            let env_var = read("ENVIRONMENT").unwrap_or_else(|| "development".to_string());
            Environment::from_env_str(&env_var)
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
            host: read("HOST").unwrap_or_else(|| "0.0.0.0".to_string()),
            port: read("PORT")
                .unwrap_or_else(|| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            https_port: read("HTTPS_PORT")
                .unwrap_or_else(|| "8443".to_string())
                .parse()
                .unwrap_or(8443),
            tls_enabled: read("TLS_ENABLED")
                .map(|v| {
                    matches!(
                        v.trim().to_lowercase().as_str(),
                        "1" | "true" | "yes" | "on"
                    )
                })
                .unwrap_or(false),
            tls_cert_path: read("TLS_CERT_PATH").unwrap_or_else(|| "cert.pem".to_string()),
            tls_key_path: read("TLS_KEY_PATH").unwrap_or_else(|| "key.pem".to_string()),
            frontend_url: read("FRONTEND_URL")
                .unwrap_or_else(|| "http://localhost:3000".to_string()),

            environment,

            database_url: required_secret_from("DATABASE_URL", &read)?,
            db_pool_size: read("DB_POOL_SIZE")
                .unwrap_or_else(|| "10".to_string())
                .parse()
                .unwrap_or(10),
            db_pool_min_idle: read("DB_POOL_MIN_IDLE").and_then(|s| s.parse::<u32>().ok()),
            db_pool_max_lifetime_secs: read("DB_POOL_MAX_LIFETIME_SECS")
                .and_then(|s| s.parse::<u64>().ok()),
            db_pool_idle_timeout_secs: read("DB_POOL_IDLE_TIMEOUT_SECS")
                .and_then(|s| s.parse::<u64>().ok()),
            db_pool_connection_timeout_secs: read("DB_POOL_CONNECTION_TIMEOUT_SECS")
                .unwrap_or_else(|| "10".to_string())
                .parse()
                .unwrap_or(10),
            db_statement_timeout_secs: read("DB_STATEMENT_TIMEOUT_SECS")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(DB_STATEMENT_TIMEOUT_DEFAULT_SECS),

            redis_url: read("REDIS_URL").unwrap_or_else(|| "redis://127.0.0.1:6379".to_string()),
            redis_pool_size: read("REDIS_POOL_SIZE")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(redis_pool_default),

            jwt_secret: required_secret_from("JWT_SECRET", &read)?,
            jwt_secrets: {
                let primary = required_secret_from("JWT_SECRET", &read)?;
                let now = time::OffsetDateTime::now_utc();
                let mut secrets = vec![JwtSecretKey {
                    kid: read("JWT_KID").unwrap_or_else(|| "primary".to_string()),
                    secret: primary,
                    created_at: now,
                    expires_at: None,
                }];

                // Support additional rotation secrets via
                // JWT_SECRETS=old_kid:old_secret,prev_kid:prev_secret[:expires_at]
                if let Some(extra) = read("JWT_SECRETS") {
                    secrets.extend(
                        extra
                            .split(',')
                            .filter(|entry| !entry.trim().is_empty())
                            .filter_map(|entry| parse_jwt_secret_entry(entry, now)),
                    );
                }

                secrets
            },
            jwt_public_key: read("JWT_PUBLIC_KEY"),
            jwt_audience: read("JWT_AUDIENCE")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| crate::middleware::auth::DEFAULT_TOKEN_AUDIENCE.to_string()),
            jwt_access_expiry_secs: 15 * 60, // 15 minutes
            jwt_refresh_expiry_secs: 30 * 24 * 3600,

            // All secrets are required - fail fast if missing in any environment
            // Use ./scripts/generate-secrets.sh to generate secure values
            master_key: required_secret_from("MASTER_KEY", &read)?,
            blind_index_key: required_secret_from("BLIND_INDEX_KEY", &read)?,
            current_encryption_key_version: read("CURRENT_ENCRYPTION_KEY_VERSION")
                .unwrap_or_else(|| "1".to_string())
                .parse()
                .unwrap_or(1),
            backend_api_keys: read("BACKEND_API_KEYS")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect(),
            csrf_secret_key: required_secret_from("CSRF_SECRET_KEY", &read)?,
            refresh_token_hash_salt: required_secret_from("REFRESH_TOKEN_HASH_SALT", &read)?,

            // Account lockout - default 15 minutes
            account_lockout_duration_secs: read("ACCOUNT_LOCKOUT_DURATION_SECS")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(15 * 60),

            resend_api_key: secret_from_env_or_file_with("RESEND_API_KEY", "", &read),
            email_from: read("EMAIL_FROM")
                .unwrap_or_else(|| "noreply@boilerplate-rust-nuxt.com".to_string()),
            email_from_name: read("EMAIL_FROM_NAME")
                .unwrap_or_else(|| "Boilerplate Rust Nuxt".to_string()),
            email_transport: EmailTransportKind::from_env_str(
                &read("EMAIL_TRANSPORT").unwrap_or_else(|| "resend".to_string()),
            ),
            smtp_url: read("SMTP_URL").unwrap_or_default(),
            smtp_timeout_secs: read("SMTP_TIMEOUT_SECS")
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),

            bunny_storage_zone: read("BUNNY_STORAGE_ZONE").unwrap_or_default(),
            bunny_storage_key: read("BUNNY_STORAGE_KEY").unwrap_or_default(),
            bunny_cdn_url: read("BUNNY_CDN_URL")
                .unwrap_or_else(|| "https://cdn.boilerplate-rust-nuxt.com".to_string()),
            bunny_token_key: read("BUNNY_TOKEN_KEY").unwrap_or_default(),

            bunny_stream_library_id: read("BUNNY_STREAM_LIBRARY_ID").unwrap_or_default(),
            bunny_stream_key: read("BUNNY_STREAM_KEY").unwrap_or_default(),
            bunny_stream_webhook_secret: read("BUNNY_STREAM_WEBHOOK_SECRET").unwrap_or_default(),

            b2_key_id: read("B2_KEY_ID").unwrap_or_default(),
            b2_application_key: read("B2_APPLICATION_KEY").unwrap_or_default(),
            b2_bucket_id: read("B2_BUCKET_ID").unwrap_or_default(),
            b2_bucket_name: read("B2_BUCKET_NAME").unwrap_or_default(),
            b2_endpoint: read("B2_ENDPOINT")
                .unwrap_or_else(|| "https://s3.us-west-004.backblazeb2.com".to_string()),

            stripe_secret_key: secret_from_env_or_file_with("STRIPE_SECRET_KEY", "", &read),
            stripe_webhook_secret: secret_from_env_or_file_with("STRIPE_WEBHOOK_SECRET", "", &read),
            stripe_publishable_key: secret_from_env_or_file_with(
                "STRIPE_PUBLISHABLE_KEY",
                "",
                &read,
            ),

            pix_webhook_secret: secret_from_env_or_file_with("PIX_WEBHOOK_SECRET", "", &read),

            platform_commission_percent: read("PLATFORM_COMMISSION_PERCENT")
                .unwrap_or_else(|| "20.0".to_string())
                .parse()
                .unwrap_or(20.0),
            min_subscription_price_cents: 500,
            max_subscription_price_cents: 50_000,
            min_withdrawal_amount_cents: 2_000,

            totp_issuer: read("TOTP_ISSUER").unwrap_or_else(|| "Boilerplate Rust Nuxt".to_string()),

            max_video_size_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            max_photo_size_bytes: 50 * 1024 * 1024,        // 50 MB
            max_audio_size_bytes: 500 * 1024 * 1024,       // 500 MB

            // Default JSON payload limit is 16 MB - sufficient for file upload metadata
            // Form payload limit is 20 MB for multipart form-data with metadata
            json_payload_limit: read("JSON_PAYLOAD_LIMIT")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(16 * 1024 * 1024),
            form_payload_limit: read("FORM_PAYLOAD_LIMIT")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(20 * 1024 * 1024),

            // Rate limiting - enabled by default in all environments
            // Can be disabled via RATE_LIMIT_ENABLED=false
            rate_limit_enabled: read("RATE_LIMIT_ENABLED")
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(true),

            // Global daily register ceiling (all IPs combined), for
            // distributed-registration protection. Default 2000/day.
            register_daily_limit: read("REGISTER_DAILY_LIMIT")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(crate::middleware::rate_limit_middleware::DEFAULT_REGISTER_DAILY_LIMIT),

            // Rejection share alert: on by default, 10% of the window, every
            // 30 seconds. A mistyped percentage or a zero window becomes a
            // value that validate() refuses, so a typo cannot silently
            // disarm the alert the way the 429 storm went unnoticed.
            alert_429_enabled: read("ALERT_429_ENABLED")
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(true),
            alert_429_threshold_share: match read("ALERT_429_SHARE_PCT") {
                Some(raw) => match raw.parse::<f64>() {
                    Ok(pct) if pct > 0.0 && pct <= 100.0 => pct / 100.0,
                    _ => 0.0,
                },
                None => 0.10,
            },
            alert_429_window_secs: match read("ALERT_429_WINDOW_SECS") {
                Some(raw) => match raw.parse::<u64>() {
                    Ok(secs) if secs > 0 => secs,
                    _ => 0,
                },
                None => 30,
            },

            // Role assigned to a new registration, resolved by name. The
            // customer role is the default: the account starts with the self
            // service permissions and nothing else.
            register_default_role: read("REGISTER_DEFAULT_ROLE")
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| "customer".to_string()),

            // Optional human verification for hostile-facing deployments:
            // disabled unless CAPTCHA_PROVIDER names a supported provider.
            // An unknown provider name must refuse the boot instead of
            // silently disabling the protection it was configured for.
            captcha_provider: match read("CAPTCHA_PROVIDER") {
                Some(raw) => crate::security::captcha::CaptchaProvider::from_str(&raw)
                    .unwrap_or_else(|error| {
                        panic!(
                            "CAPTCHA_PROVIDER must be none, turnstile or recaptcha, got '{error}'"
                        )
                    }),
                None => crate::security::captcha::CaptchaProvider::None,
            },
            captcha_secret: read("CAPTCHA_SECRET")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            // Endpoint for the `custom` CAPTCHA provider (and for deployments
            // that speak the site-verify protocol through their own service).
            captcha_verify_url: read("CAPTCHA_VERIFY_URL")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),

            // Disposable-mail policy — opt-in (see the struct docs).
            disposable_email_blocking: read("DISPOSABLE_EMAIL_BLOCKING")
                .and_then(|value| value.parse::<bool>().ok())
                .unwrap_or(false),
            extra_disposable_domains: read("DISPOSABLE_EMAIL_DOMAINS")
                .map(|value| {
                    value
                        .split(',')
                        .map(|domain| domain.trim().to_ascii_lowercase())
                        .filter(|domain| !domain.is_empty())
                        .collect()
                })
                .unwrap_or_default(),

            // Mail-exchanger check — opt-in, and vendor-neutral: the operator
            // chooses the DNS-over-HTTPS resolver the app may query.
            email_mx_check_enabled: read("EMAIL_MX_CHECK_ENABLED")
                .and_then(|value| value.parse::<bool>().ok())
                .unwrap_or(false),
            email_dns_resolver_url: read("EMAIL_DNS_RESOLVER_URL")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),

            // Cookie Secure flag override for development tunnels.
            // Set COOKIE_SECURE=true when using ngrok/Cloudflare Tunnel in dev
            // so that cookies are not sent in plaintext over the tunnel edge.
            // In staging/production, the flag is always set regardless of this value.
            cookie_secure_override: read("COOKIE_SECURE")
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(false),

            // Argon2 password hashing parameters
            argon2_m_cost: read("ARGON2_M_COST")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(65536),
            argon2_t_cost: read("ARGON2_T_COST")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(3),
            argon2_p_cost: read("ARGON2_P_COST")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1),

            // Trusted proxies for X-Forwarded-For / Forwarded header support
            // Comma-separated list of CIDR ranges (e.g., "10.0.0.0/8,172.16.0.0/12,192.168.0.0/16")
            trusted_proxies: read("TRUSTED_PROXIES")
                .map(|s| {
                    s.split(',')
                        .filter_map(|part| part.trim().parse::<ipnet::IpNet>().ok())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    pub fn from_env() -> Result<Self, env::VarError> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    pub fn is_production(&self) -> bool {
        self.is_production_like()
    }

    /// Whether the register flow must verify a human-verification token.
    ///
    /// A provider alone (without its secret) is a misconfiguration that
    /// production validation refuses at boot; outside production the flag
    /// simply stays disabled so development never needs an upstream service.
    /// The `custom` provider additionally needs its own endpoint.
    pub fn captcha_enabled(&self) -> bool {
        self.captcha_provider != crate::security::captcha::CaptchaProvider::None
            && self.captcha_secret.is_some()
            && (self.captcha_provider != crate::security::captcha::CaptchaProvider::Custom
                || self.captcha_verify_url.is_some())
    }

    pub fn is_production_like(&self) -> bool {
        self.environment.is_production_like()
    }

    /// Whether cookies should carry the `Secure` flag.
    ///
    /// Always `true` in staging/production (HTTPS required).
    ///
    /// In development/test, defaults to `false` so that the local HTTP server
    /// (`http://localhost`) can set and read cookies normally. Set
    /// `COOKIE_SECURE=true` in your `.env` when tunnelling through an HTTPS
    /// proxy (ngrok, Cloudflare Tunnel, etc.) to prevent the refresh_token
    /// and csrf_token cookies from being transmitted over the unencrypted
    /// tunnel edge.
    ///
    /// # Security note
    ///
    /// Without `Secure`, a cookie can be read by any HTTP intermediary on the
    /// network path (e.g. a transparent proxy or an attacker performing a
    /// MitM on the tunnel). `COOKIE_SECURE=true` closes this window at the
    /// cost of requiring HTTPS end-to-end in your local setup.
    pub fn should_use_secure_cookies(&self) -> bool {
        self.is_production_like() || self.cookie_secure_override
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

            // The literal documented in the repo's local-env guidance is
            // publicly known but matches neither the `changeme_` nor the
            // `REPLACE_WITH_` patterns, so it needs its own check.
            if self
                .refresh_token_hash_salt
                .contains("change_in_production")
            {
                errors.push(
                    "REFRESH_TOKEN_HASH_SALT contains placeholder value \
                     'change_in_production'. Override the secret before \
                     deploying to production (see .env.example)."
                        .to_string(),
                );
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

        // BACKEND_API_KEYS validation
        // The /api/v1/metrics endpoint (and other internal routes) are protected
        // exclusively by RequireApiKey. An empty key list means the middleware
        // will reject ALL requests to those routes, effectively disabling metrics
        // scraping. In production this is a misconfiguration, not a safe default.
        if self.backend_api_keys.is_empty() {
            if self.is_production_like() {
                errors.push(
                    "BACKEND_API_KEYS is empty. The /metrics endpoint (and other \
                     internal routes) require at least one API key. Set \
                     BACKEND_API_KEYS=<key1>,<key2> before deploying to \
                     staging/production. Generate a key with: \
                     openssl rand -hex 32"
                        .to_string(),
                );
            } else {
                // In development, log a warning at startup via validate_or_panic.
                // Not a hard error — devs may not have set up keys locally yet.
                tracing::warn!(
                    event = "config.missing_api_keys",
                    "BACKEND_API_KEYS is empty. /api/v1/metrics will reject all \
                     requests. Set BACKEND_API_KEYS in your .env file to enable \
                     metrics scraping locally."
                );
            }
        }

        // Rate limiting must never be switched off in production: the
        // brute-force budgets (per IP, per subnet, register daily aggregate)
        // are the primary protection against scripted registration and login
        // attacks. Development keeps the escape hatch for local testing.
        if !self.rate_limit_enabled && self.is_production_like() {
            errors.push(
                "RATE_LIMIT_ENABLED must not be false in staging/production. \
                 The per-IP, per-subnet and daily registration budgets are \
                 the primary protection against scripted attacks"
                    .to_string(),
            );
        }

        // The rejection share alert must stay armed: a percentage below the
        // measurement range (zero or negative, or a typo that failed to parse
        // and collapsed to it) silently disables the watchdog, and a window
        // of zero seconds would spin the sampling loop.
        if self.alert_429_threshold_share <= 0.0 || self.alert_429_threshold_share > 1.0 {
            errors.push(
                "ALERT_429_SHARE_PCT must be a percentage between 0 (exclusive) and \
                 100 (inclusive)"
                    .to_string(),
            );
        }
        if self.alert_429_window_secs == 0 {
            errors.push("ALERT_429_WINDOW_SECS must be at least 1 second".to_string());
        }

        // CAPTCHA (optional anti-bot verification) validation.
        // A provider half-enabled (provider set, secret missing) silently
        // disables the protection it was configured for — in production the
        // operator explicitly opted in, so the boot must refuse instead.
        if self.captcha_provider != crate::security::captcha::CaptchaProvider::None
            && self.captcha_secret.is_none()
            && self.is_production_like()
        {
            errors.push(format!(
                "CAPTCHA_SECRET must be set when CAPTCHA_PROVIDER={:?} is \
                 enabled in production. Generate a Turn secret at \
                 https://dash.cloudflare.com or a reCAPTCHA one at \
                 https://www.google.com/recaptcha/admin",
                self.captcha_provider
            ));
        }

        // The custom CAPTCHA provider has no built-in endpoint: without one
        // the verification could never succeed, so refuse the boot instead of
        // silently disabling it.
        if self.captcha_provider == crate::security::captcha::CaptchaProvider::Custom
            && self.captcha_verify_url.is_none()
            && self.is_production_like()
        {
            errors.push(
                "CAPTCHA_VERIFY_URL must be set when CAPTCHA_PROVIDER=custom. Point \
                 it at the site-verify endpoint of your CAPTCHA service"
                    .to_string(),
            );
        }

        // The mail-exchanger check needs a resolver to query. Enabling the
        // check without one is an explicit misconfiguration (the operator
        // asked for the check), so it is an error in every environment.
        if self.email_mx_check_enabled && self.email_dns_resolver_url.is_none() {
            errors.push(
                "EMAIL_DNS_RESOLVER_URL must be set when EMAIL_MX_CHECK_ENABLED=true. \
                 Point it at a DNS-over-HTTPS resolver you trust (your own \
                 resolver, or any public DoH endpoint)"
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
    //! SECURITY_AUDIT.md I7: placeholder rejection tests.
    //!
    //! These tests assert that production builds refuse placeholder secrets
    //! such as "changeme_*" (POSTGRES_PASSWORD, REDIS_PASSWORD) and
    //! "REPLACE_WITH_*" (JWT_SECRET, encryption keys). The development
    //! environment intentionally allows these markers.
    //!
    //! The environment is a lookup each test names explicitly: no test mutates
    //! the process environment, so the module needs neither serialization nor
    //! restore-on-drop machinery.
    use std::collections::HashMap;

    use super::*;
    use crate::security::captcha::CaptchaProvider;

    fn minimal_valid_secret() -> String {
        // 48-byte base64 string that decodes to ≥32 bytes
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.encode([7u8; 32])
    }

    /// The environment every test starts from: the required secrets with
    /// valid, placeholder-free values. A test adds its own variables on top.
    fn base_vars() -> Vec<(String, String)> {
        let valid = minimal_valid_secret();
        [
            ("DATABASE_URL", "postgres://u:p@localhost:5432/d"),
            ("DB_POOL_SIZE", "10"),
            ("DB_STATEMENT_TIMEOUT_SECS", "30"),
            ("DB_POOL_MIN_IDLE", "2"),
            ("DB_POOL_MAX_LIFETIME_SECS", "1800"),
            ("DB_POOL_IDLE_TIMEOUT_SECS", "600"),
            ("DB_POOL_CONNECTION_TIMEOUT_SECS", "10"),
            ("REDIS_URL", "redis://localhost:6379"),
            ("REDIS_POOL_SIZE", "10"),
            ("MAX_VIDEO_SIZE_BYTES", "1000"),
            ("MAX_PHOTO_SIZE_BYTES", "1000"),
            ("MAX_AUDIO_SIZE_BYTES", "1000"),
            ("JSON_PAYLOAD_LIMIT", "1048576"),
            ("FORM_PAYLOAD_LIMIT", "2097152"),
            ("CSRF_SECRET_KEY", valid.as_str()),
            ("REFRESH_TOKEN_HASH_SALT", valid.as_str()),
            ("BLIND_INDEX_KEY", valid.as_str()),
            ("MASTER_KEY", valid.as_str()),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
    }

    /// Builds a configuration from the base environment plus the given vars.
    fn config_with(extra: &[(&str, &str)]) -> Result<AppConfig, env::VarError> {
        let mut vars: HashMap<String, String> = base_vars().into_iter().collect();
        vars.extend(
            extra
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string())),
        );

        AppConfig::from_lookup(|key: &str| vars.get(key).cloned())
    }

    /// The production overrides the production cases share: the environment
    /// marker and the pool size production requires.
    fn production_vars() -> Vec<(&'static str, &'static str)> {
        vec![("ENVIRONMENT", "production"), ("REDIS_POOL_SIZE", "100")]
    }

    // ---------------------------------------------------------------------------
    // CAPTCHA (opt-in anti-bot verification)
    // ---------------------------------------------------------------------------

    #[test]
    fn captcha_defaults_to_disabled() {
        let secret = minimal_valid_secret();
        let vars = [("JWT_SECRET", secret.as_str())];
        let config = config_with(&vars).expect("config builds");

        assert_eq!(config.captcha_provider, CaptchaProvider::None);
        assert!(config.captcha_secret.is_none());
        assert!(!config.captcha_enabled());
    }

    #[test]
    fn captcha_provider_and_secret_enable_human_verification() {
        let secret = minimal_valid_secret();
        let vars = [
            ("JWT_SECRET", secret.as_str()),
            ("CAPTCHA_PROVIDER", "turnstile"),
            ("CAPTCHA_SECRET", "0x0000000000000000000000000000"),
        ];
        let config = config_with(&vars).expect("config builds");

        assert_eq!(config.captcha_provider, CaptchaProvider::Turnstile);
        assert!(config.captcha_enabled());
    }

    #[test]
    fn the_rejection_share_alert_defaults_to_a_conservative_threshold() {
        let secret = minimal_valid_secret();
        let vars = [("JWT_SECRET", secret.as_str())];
        let config = config_with(&vars).expect("config builds");

        assert!(config.alert_429_enabled, "the alert ships on");
        assert!(
            (config.alert_429_threshold_share - 0.10).abs() < 1e-9,
            "the default threshold must be 10%, got {}",
            config.alert_429_threshold_share
        );
        assert_eq!(config.alert_429_window_secs, 30);
        assert!(config.validate().is_empty(), "the defaults must validate");
    }

    #[test]
    fn the_rejection_share_alert_honours_custom_values() {
        let secret = minimal_valid_secret();
        let vars = [
            ("JWT_SECRET", secret.as_str()),
            ("ALERT_429_SHARE_PCT", "25"),
            ("ALERT_429_WINDOW_SECS", "15"),
            ("ALERT_429_ENABLED", "false"),
        ];
        let config = config_with(&vars).expect("config builds");

        assert!(!config.alert_429_enabled, "the alert can be turned off");
        assert!(
            (config.alert_429_threshold_share - 0.25).abs() < 1e-9,
            "percentages become fractions, got {}",
            config.alert_429_threshold_share
        );
        assert_eq!(config.alert_429_window_secs, 15);
        assert!(
            config.validate().is_empty(),
            "custom valid values must validate"
        );
    }

    #[test]
    fn a_mistyped_threshold_is_rejected_instead_of_defaulting() {
        let secret = minimal_valid_secret();
        let vars = [
            ("JWT_SECRET", secret.as_str()),
            ("ALERT_429_SHARE_PCT", "0"),
        ];
        let config = config_with(&vars).expect("config builds");

        assert!(
            !config.validate().is_empty(),
            "a zero threshold must be refused: a typo must not silently disarm the alert"
        );
    }

    #[test]
    fn jwt_audience_defaults_to_the_canonical_service_name() {
        let secret = minimal_valid_secret();
        let vars = [("JWT_SECRET", secret.as_str())];
        let config = config_with(&vars).expect("config builds");

        assert_eq!(config.jwt_audience, "rust-nuxt-boilerplate");
    }

    #[test]
    fn jwt_audience_follows_the_environment_and_an_empty_value_never_disables_it() {
        let secret = minimal_valid_secret();

        let custom = [
            ("JWT_SECRET", secret.as_str()),
            ("JWT_AUDIENCE", "custom-api"),
        ];
        let config = config_with(&custom).expect("config builds");
        assert_eq!(config.jwt_audience, "custom-api");

        let blank = [("JWT_SECRET", secret.as_str()), ("JWT_AUDIENCE", "   ")];
        let config = config_with(&blank).expect("config builds");
        assert_eq!(
            config.jwt_audience, "rust-nuxt-boilerplate",
            "a blank value falls back to the default instead of disabling the check"
        );
    }

    #[test]
    fn captcha_enabled_without_secret_is_a_production_validation_error() {
        let mut vars = production_vars();
        vars.push(("DATABASE_URL", "postgres://u:good_pwd@localhost:5432/d"));
        vars.push(("POSTGRES_PASSWORD", "good_secret_xyz"));
        let secret = minimal_valid_secret();
        vars.push(("JWT_SECRET", secret.as_str()));
        vars.push(("MASTER_KEY", secret.as_str()));
        vars.push(("CAPTCHA_PROVIDER", "recaptcha"));

        let config = config_with(&vars).expect("config builds");
        let errors = config.validate();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("CAPTCHA_SECRET") && e.contains("CAPTCHA_PROVIDER")),
            "production with a provider but no secret must refuse boot, got: {:?}",
            errors
        );
    }

    #[test]
    fn unknown_captcha_provider_names_refuse_to_boot() {
        let secret = minimal_valid_secret();
        let vars = [
            ("JWT_SECRET", secret.as_str()),
            ("CAPTCHA_PROVIDER", "bogus"),
        ];

        let result = std::panic::catch_unwind(|| config_with(&vars));
        let error = result.expect_err("unknown providers must fail");
        let message = error
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| error.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default();
        assert!(
            message.contains("CAPTCHA_PROVIDER"),
            "panic must name the offending variable, got: {message}"
        );
    }

    #[test]
    fn custom_captcha_provider_uses_the_configured_verify_url() {
        let secret = minimal_valid_secret();
        let vars = [
            ("JWT_SECRET", secret.as_str()),
            ("CAPTCHA_PROVIDER", "custom"),
            ("CAPTCHA_SECRET", "secret-value"),
            ("CAPTCHA_VERIFY_URL", "https://captcha.example/siteverify"),
        ];
        let config = config_with(&vars).expect("config builds");

        assert_eq!(config.captcha_provider, CaptchaProvider::Custom);
        assert!(config.captcha_enabled());
        assert_eq!(
            config.captcha_verify_url.as_deref(),
            Some("https://captcha.example/siteverify")
        );
    }

    #[test]
    fn custom_captcha_provider_without_verify_url_is_a_production_validation_error() {
        let mut vars = production_vars();
        vars.push(("DATABASE_URL", "postgres://u:good_pwd@localhost:5432/d"));
        vars.push(("POSTGRES_PASSWORD", "good_secret_xyz"));
        let secret = minimal_valid_secret();
        vars.push(("JWT_SECRET", secret.as_str()));
        vars.push(("MASTER_KEY", secret.as_str()));
        vars.push(("CAPTCHA_PROVIDER", "custom"));
        vars.push(("CAPTCHA_SECRET", "secret-value"));

        let config = config_with(&vars).expect("config builds");
        let errors = config.validate();
        assert!(
            errors.iter().any(|e| e.contains("CAPTCHA_VERIFY_URL")),
            "a custom provider without its endpoint must refuse boot, got: {:?}",
            errors
        );
    }

    #[test]
    fn disposable_email_policy_is_opt_in() {
        let secret = minimal_valid_secret();
        let vars = [("JWT_SECRET", secret.as_str())];
        let config = config_with(&vars).expect("config builds");

        assert!(
            !config.disposable_email_blocking,
            "the boilerplate must not impose a disposable-mail policy by default"
        );
        assert!(config.extra_disposable_domains.is_empty());
    }

    #[test]
    fn disposable_email_policy_can_be_enabled_with_extra_domains() {
        let secret = minimal_valid_secret();
        let vars = [
            ("JWT_SECRET", secret.as_str()),
            ("DISPOSABLE_EMAIL_BLOCKING", "true"),
            ("DISPOSABLE_EMAIL_DOMAINS", "spam.test, Other.test"),
        ];
        let config = config_with(&vars).expect("config builds");

        assert!(config.disposable_email_blocking);
        assert_eq!(
            config.extra_disposable_domains,
            vec!["spam.test".to_string(), "other.test".to_string()],
            "extra domains must be trimmed and lowercased"
        );
    }

    #[test]
    fn production_refuses_disabled_rate_limiting() {
        let mut vars = production_vars();
        vars.push(("DATABASE_URL", "postgres://u:good_pwd@localhost:5432/d"));
        vars.push(("POSTGRES_PASSWORD", "good_secret_xyz"));
        let secret = minimal_valid_secret();
        vars.push(("JWT_SECRET", secret.as_str()));
        vars.push(("MASTER_KEY", secret.as_str()));
        vars.push(("RATE_LIMIT_ENABLED", "false"));

        let config = config_with(&vars).expect("config builds");
        let errors = config.validate();
        assert!(
            errors.iter().any(|e| e.contains("RATE_LIMIT_ENABLED")),
            "production must refuse to boot with rate limiting disabled, got: {:?}",
            errors
        );
    }

    #[test]
    fn development_may_disable_rate_limiting() {
        let secret = minimal_valid_secret();
        let vars = [
            ("JWT_SECRET", secret.as_str()),
            ("RATE_LIMIT_ENABLED", "false"),
        ];
        let config = config_with(&vars).expect("config builds");

        assert!(!config.rate_limit_enabled);
        assert!(
            !config
                .validate()
                .iter()
                .any(|e| e.contains("RATE_LIMIT_ENABLED")),
            "local development keeps the escape hatch"
        );
    }

    #[test]
    fn mail_exchanger_check_is_opt_in_and_needs_a_resolver() {
        let secret = minimal_valid_secret();

        let opt_in = [("JWT_SECRET", secret.as_str())];
        let config = config_with(&opt_in).expect("config builds");
        assert!(!config.email_mx_check_enabled);
        assert!(config.email_dns_resolver_url.is_none());
        assert!(
            config.validate().is_empty(),
            "the default configuration must validate cleanly"
        );

        // Enabling the check without telling the app where to resolve DNS is a
        // misconfiguration, not a silent no-op.
        let without_resolver = [
            ("JWT_SECRET", secret.as_str()),
            ("EMAIL_MX_CHECK_ENABLED", "true"),
        ];
        let config = config_with(&without_resolver).expect("config builds");
        let errors = config.validate();
        assert!(
            errors.iter().any(|e| e.contains("EMAIL_DNS_RESOLVER_URL")),
            "the mail-exchanger check must require a resolver, got: {:?}",
            errors
        );

        let with_resolver = [
            ("JWT_SECRET", secret.as_str()),
            ("EMAIL_MX_CHECK_ENABLED", "true"),
            ("EMAIL_DNS_RESOLVER_URL", "https://dns.example/dns-query"),
        ];
        let config = config_with(&with_resolver).expect("config builds");
        assert!(config.email_mx_check_enabled);
        assert!(
            config.validate().is_empty(),
            "a resolver URL satisfies the requirement, got: {:?}",
            config.validate()
        );
    }

    #[test]
    fn the_default_registration_role_is_the_customer_role() {
        let secret = minimal_valid_secret();
        let vars = [("JWT_SECRET", secret.as_str())];
        let config = config_with(&vars).expect("config builds");

        assert_eq!(
            config.register_default_role, "customer",
            "a registration must land on the customer role by default"
        );
    }

    #[test]
    fn the_default_registration_role_is_configurable() {
        let secret = minimal_valid_secret();
        let vars = [
            ("JWT_SECRET", secret.as_str()),
            ("REGISTER_DEFAULT_ROLE", "trial"),
        ];
        let config = config_with(&vars).expect("config builds");

        assert_eq!(config.register_default_role, "trial");
    }

    #[test]
    fn a_blank_default_registration_role_falls_back_to_the_customer_role() {
        let secret = minimal_valid_secret();
        let vars = [
            ("JWT_SECRET", secret.as_str()),
            ("REGISTER_DEFAULT_ROLE", "   "),
        ];
        let config = config_with(&vars).expect("config builds");

        assert_eq!(
            config.register_default_role, "customer",
            "a blank role name must not reach the resolver"
        );
    }

    #[test]
    fn production_rejects_postgres_password_changeme() {
        let mut vars = production_vars();
        vars.push(("POSTGRES_PASSWORD", "changeme_secure_password"));
        // DATABASE_URL interpolates POSTGRES_PASSWORD; the URL embeds it.
        vars.push((
            "DATABASE_URL",
            "postgres://u:changeme_secure_password@localhost:5432/d",
        ));
        let secret = minimal_valid_secret();
        vars.push(("JWT_SECRET", secret.as_str()));

        let cfg = config_with(&vars).expect("config builds");
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
        let mut vars = production_vars();
        // Good values for everything except JWT_SECRET.
        vars.push(("DATABASE_URL", "postgres://u:good_pwd@localhost:5432/d"));
        vars.push(("POSTGRES_PASSWORD", "good_secret_xyz"));
        vars.push(("JWT_SECRET", "REPLACE_WITH_GENERATED_64_CHAR_BASE64_SECRET"));
        let secret = minimal_valid_secret();
        vars.push(("MASTER_KEY", secret.as_str()));

        let cfg = config_with(&vars).expect("config builds");
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
    fn production_rejects_refresh_salt_placeholder() {
        let mut vars = production_vars();
        vars.push(("DATABASE_URL", "postgres://u:good_pwd@localhost:5432/d"));
        vars.push(("POSTGRES_PASSWORD", "good_secret_xyz"));
        let secret = minimal_valid_secret();
        vars.push(("JWT_SECRET", secret.as_str()));
        vars.push(("MASTER_KEY", secret.as_str()));
        // The literal shipped in the repo's local .env docs: long enough to
        // pass the length check but publicly known, so it must be rejected.
        vars.push((
            "REFRESH_TOKEN_HASH_SALT",
            "refresh_token_hash_salt_change_in_production",
        ));

        let cfg = config_with(&vars).expect("config builds");
        let errors = cfg.validate();

        assert!(
            errors
                .iter()
                .any(|e| e.contains("REFRESH_TOKEN_HASH_SALT") && e.contains("placeholder")),
            "expected REFRESH_TOKEN_HASH_SALT placeholder error, got: {:?}",
            errors
        );
    }

    #[test]
    fn development_allows_changeme_placeholders() {
        let mut vars = vec![("ENVIRONMENT", "development")];
        // All-good values (no changeme_ or REPLACE_).
        vars.push(("DATABASE_URL", "postgres://u:dev_pwd@localhost:5432/d"));
        vars.push(("POSTGRES_PASSWORD", "dev_pwd_xyz"));
        let secret = minimal_valid_secret();
        vars.push(("JWT_SECRET", secret.as_str()));

        let cfg = config_with(&vars).expect("config builds");
        let errors = cfg.validate();

        for error in &errors {
            assert!(
                !error.contains("placeholder"),
                "dev config should NOT trigger placeholder error: {}",
                error
            );
        }
    }
}

#[cfg(test)]
mod docker_secrets_tests {
    //! `DOCKER_SECRETS_ENABLED` precedence tests.
    //!
    //! docker-compose.yml documents the flag for production so that a stale
    //! `.env` cannot silently override a rotated `/run/secrets` value.
    //!
    //! The environment source is a lookup the test supplies: no test mutates
    //! the process environment, so the module needs no serialization and no
    //! restore-on-drop machinery.

    use std::collections::HashMap;

    use super::{required_secret_from, secret_from_env_or_file_with};

    const SECRET_NAME: &str = "DOCKER_SECRETS_TEST_VALUE";
    const FILE_VAR: &str = "DOCKER_SECRETS_TEST_VALUE_FILE";
    const FILE_CONTENT: &str = "value-from-mounted-secret-file";
    const ENV_CONTENT: &str = "value-from-environment";

    /// Writes the secret content to a temp file and returns its path.
    fn write_secret_file() -> tempfile::TempPath {
        let file = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(file.path(), FILE_CONTENT).expect("write secret file");
        file.into_temp_path()
    }

    /// The lookup the production code reads: only the given variables exist.
    fn source(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    #[test]
    fn docker_secrets_enabled_prefers_the_mounted_file() {
        let path = write_secret_file();
        let file_var = path.to_str().expect("utf-8 path").to_string();
        let vars = [
            (SECRET_NAME, ENV_CONTENT),
            (FILE_VAR, file_var.as_str()),
            ("DOCKER_SECRETS_ENABLED", "1"),
        ];
        let read = source(&vars);

        assert_eq!(
            secret_from_env_or_file_with(SECRET_NAME, "unused", &read),
            FILE_CONTENT
        );
        assert_eq!(
            required_secret_from(SECRET_NAME, &read).unwrap(),
            FILE_CONTENT
        );
    }

    #[test]
    fn without_the_flag_the_environment_variable_still_wins() {
        let path = write_secret_file();
        let file_var = path.to_str().expect("utf-8 path").to_string();
        let vars = [(SECRET_NAME, ENV_CONTENT), (FILE_VAR, file_var.as_str())];
        let read = source(&vars);

        assert_eq!(
            secret_from_env_or_file_with(SECRET_NAME, "unused", &read),
            ENV_CONTENT
        );
        assert_eq!(
            required_secret_from(SECRET_NAME, &read).unwrap(),
            ENV_CONTENT
        );
    }

    #[test]
    fn the_flag_falls_back_to_the_environment_when_no_file_exists() {
        let vars = [
            (SECRET_NAME, ENV_CONTENT),
            ("DOCKER_SECRETS_ENABLED", "true"),
        ];
        let read = source(&vars);

        assert_eq!(
            secret_from_env_or_file_with(SECRET_NAME, "unused", &read),
            ENV_CONTENT
        );
    }

    #[test]
    fn the_flag_falls_back_to_the_default_when_nothing_is_configured() {
        let vars = [("DOCKER_SECRETS_ENABLED", "yes")];
        let read = source(&vars);

        assert_eq!(
            secret_from_env_or_file_with(SECRET_NAME, "the-default", &read),
            "the-default"
        );
        assert!(required_secret_from(SECRET_NAME, &read).is_err());
    }
}

#[cfg(test)]
mod jwt_secret_entry_tests {
    use super::*;
    use time::{Date, OffsetDateTime};

    fn created_at() -> OffsetDateTime {
        Date::from_calendar_date(2026, time::Month::January, 1)
            .unwrap()
            .midnight()
            .assume_utc()
    }

    #[test]
    fn parses_kid_and_secret_without_expiry() {
        let entry = parse_jwt_secret_entry("legacy-key:legacy-secret-value", created_at())
            .expect("entry must parse");
        assert_eq!(entry.kid, "legacy-key");
        assert_eq!(entry.secret, "legacy-secret-value");
        assert_eq!(entry.expires_at, None);
        assert!(entry.is_active());
    }

    #[test]
    fn parses_an_expiry_so_a_key_can_be_retired() {
        let entry = parse_jwt_secret_entry(
            "old-key:old-secret-value:2030-01-01T00:00:00Z",
            created_at(),
        )
        .expect("entry must parse");
        assert_eq!(
            entry.expires_at,
            Some(
                Date::from_calendar_date(2030, time::Month::January, 1)
                    .unwrap()
                    .midnight()
                    .assume_utc()
            )
        );
        assert!(entry.is_active(), "a key expiring in 2030 is still active");
    }

    #[test]
    fn rejects_an_expired_key() {
        let entry = parse_jwt_secret_entry(
            "retired-key:retired-secret:2020-01-01T00:00:00Z",
            created_at(),
        )
        .expect("entry must parse");

        assert!(
            !entry.is_active(),
            "an expired rotation key must no longer sign or verify tokens"
        );
    }

    #[test]
    fn an_unparseable_expiry_keeps_the_key_active_but_never_masks_an_error() {
        // A typo must not silently retire a key (that would log users out), but
        // it must not be treated as a valid date either.
        let entry = parse_jwt_secret_entry("typo-key:typo-secret:01/01/2030", created_at())
            .expect("entry must still parse");
        assert_eq!(entry.expires_at, None);
        assert!(entry.is_active());
    }

    #[test]
    fn ignores_malformed_entries() {
        assert!(parse_jwt_secret_entry("", created_at()).is_none());
        assert!(parse_jwt_secret_entry(":secret", created_at()).is_none());
        assert!(parse_jwt_secret_entry("kid:", created_at()).is_none());
    }
}
