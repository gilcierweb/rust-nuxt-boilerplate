#![allow(dead_code)]

use std::sync::Arc;

use chrono::Utc;
use lettre::AsyncTransport;
use lettre::message::{Mailbox, Message, MultiPart, SinglePart};
use lettre::transport::smtp::AsyncSmtpTransport;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::app_config::{AppConfig, EmailTransportKind, Environment, JwtSecretKey};
use crate::services::email_templates::{EmailTemplateError, EmailTemplates, names as tpl};
use crate::services::email_test_capture::{CapturedEmail, TestEmailCapture};
use crate::services::http_client::{HttpClient, HttpClientConfig, HttpClientError};
use crate::utils::sanitize::{sanitize_for_email, sanitize_for_html_email};

// Deterministic test key generation
fn generate_deterministic_string(length: usize, seed: u64) -> String {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn generate_deterministic_base64_key(byte_length: usize, seed: u64) -> String {
    use base64::Engine;
    use rand::{RngCore, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut bytes = vec![0u8; byte_length];
    rng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::STANDARD.encode(&bytes)
}

/// Test-only configuration for EmailService
pub fn test_config() -> AppConfig {
    AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        https_port: 8443,
        tls_enabled: false,
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
        email_transport: crate::config::app_config::EmailTransportKind::Resend,
        smtp_url: "".to_string(),
        smtp_timeout_secs: 30,
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
        cookie_secure_override: false,
        argon2_m_cost: 65536,
        argon2_t_cost: 3,
        argon2_p_cost: 1,
        trusted_proxies: Vec::new(),
    }
}

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Email service not configured: {0}")]
    NotConfigured(String),
    #[error("Failed to send email: {0}")]
    SendFailed(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] HttpClientError),
    #[error("SMTP error: {0}")]
    Smtp(String),
    #[error("Template error: {0}")]
    Template(#[from] EmailTemplateError),
}

impl From<lettre::transport::smtp::Error> for EmailError {
    fn from(err: lettre::transport::smtp::Error) -> Self {
        Self::Smtp(err.to_string())
    }
}

impl From<lettre::error::Error> for EmailError {
    fn from(err: lettre::error::Error) -> Self {
        Self::Smtp(err.to_string())
    }
}

pub type EmailResult = Result<(), EmailError>;

/// Build a `multipart/alternative` SMTP message with text + HTML bodies.
///
/// Mirrors the JSON shape sent to Resend (`from`, `to[]`, `subject`,
/// `html`, `text`) but emits RFC 5322 instead of an HTTP payload. The
/// returned `Message` is ready to hand to `AsyncSmtpTransport::send`.
fn build_smtp_message(
    from_name: &str,
    from_email: &str,
    to: &str,
    subject: &str,
    text_body: &str,
    html_body: &str,
) -> Result<Message, EmailError> {
    let from: Mailbox = format!("{} <{}>", from_name, from_email)
        .parse()
        .map_err(|e| EmailError::Smtp(format!("invalid From address: {e}")))?;
    let to: Mailbox = to
        .parse()
        .map_err(|e| EmailError::Smtp(format!("invalid To address: {e}")))?;
    Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(text_body.to_string()))
                .singlepart(SinglePart::html(html_body.to_string())),
        )
        .map_err(|e| EmailError::Smtp(format!("message build error: {e}")))
}

#[derive(Serialize)]
struct ResendEmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[derive(Deserialize)]
struct ResendEmailResponse {
    id: String,
}

/// Backend wire-format that actually delivers an email.
///
/// `Resend` keeps a long-lived `HttpClient` so the circuit breaker, retry,
/// and tracing instrumentation configured at construction time apply to every
/// `POST /emails`. `Smtp` holds a pooled `AsyncSmtpTransport` which reuses
/// connections across sends.
#[derive(Clone)]
pub enum EmailTransportImpl {
    Resend {
        client: Arc<HttpClient>,
        api_key: String,
        base_url: String,
    },
    Smtp(AsyncSmtpTransport<lettre::Tokio1Executor>),
}

impl std::fmt::Debug for EmailTransportImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resend { base_url, .. } => f
                .debug_struct("Resend")
                .field("base_url", base_url)
                .finish(),
            Self::Smtp(_) => f.debug_struct("Smtp").finish_non_exhaustive(),
        }
    }
}

pub struct EmailService {
    transport: EmailTransportImpl,
    from_email: String,
    from_name: String,
    frontend_url: String,
    templates: Option<EmailTemplates>,
    /// Optional in-memory capture used in test mode. When `Some`, outbound
    /// HTTP requests are skipped and the rendered email is recorded instead.
    capture: Option<TestEmailCapture>,
    /// Which backend is configured. Mirrors `transport` for ergonomic access
    /// (logs, instrumentation, `is_configured`).
    kind: EmailTransportKind,
}

impl EmailService {
    pub fn new(config: &AppConfig) -> Self {
        let from_email = config.email_from.clone();
        let from_name = if config.email_from_name.is_empty() {
            Self::translate_for_request_static("app.name")
        } else {
            config.email_from_name.clone()
        };
        let frontend_url = config.frontend_url.clone();

        let templates = match EmailTemplates::new() {
            Ok(t) => Some(t),
            Err(err) => {
                tracing::error!(error = %err, "failed to load email templates; falling back to inline HTML");
                None
            },
        };

        let (transport, kind) = match config.email_transport {
            EmailTransportKind::Resend => {
                let http_config = HttpClientConfig {
                    timeout: std::time::Duration::from_secs(10),
                    max_retries: 3,
                    retry_base_delay: std::time::Duration::from_millis(100),
                    circuit_breaker_threshold: 5,
                    circuit_breaker_timeout: std::time::Duration::from_secs(60),
                };
                let client =
                    Arc::new(HttpClient::new(http_config).expect("Failed to create HTTP client"));
                let api_key = config.resend_api_key.clone();
                let base_url = "https://api.resend.com".to_string();
                let impl_ = EmailTransportImpl::Resend {
                    client,
                    api_key,
                    base_url,
                };
                (impl_, EmailTransportKind::Resend)
            },
            EmailTransportKind::Smtp => {
                let url = config.smtp_url.clone();
                if url.is_empty() {
                    tracing::warn!(
                        "EMAIL_TRANSPORT=smtp but SMTP_URL is empty; the service will be unconfigured"
                    );
                }
                let timeout = std::time::Duration::from_secs(config.smtp_timeout_secs.max(1));
                // Build the async transport; if the URL is malformed we fall back
                // to a no-op transport that will surface as a Smtp error on send.
                let smtp = match AsyncSmtpTransport::<lettre::Tokio1Executor>::from_url(&url) {
                    Ok(builder) => builder.timeout(Some(timeout)).build(),
                    Err(err) => {
                        tracing::error!(error = %err, "invalid SMTP_URL; email delivery will fail until corrected");
                        // Build with a localhost placeholder so the service still
                        // constructs; the first send will surface the real error.
                        AsyncSmtpTransport::<lettre::Tokio1Executor>::from_url(
                            "smtp://127.0.0.1:25",
                        )
                        .expect("fallback SMTP transport must construct")
                        .timeout(Some(timeout))
                        .build()
                    },
                };
                (EmailTransportImpl::Smtp(smtp), EmailTransportKind::Smtp)
            },
        };

        Self {
            transport,
            from_email,
            from_name,
            frontend_url,
            templates,
            capture: None,
            kind,
        }
    }

    /// Translate a key using the per-request locale when available,
    /// falling back to the global rust_i18n locale.
    fn translate_for_request(
        key: &str,
        args: Option<&std::collections::HashMap<String, String>>,
    ) -> String {
        if let Some(rl) = crate::middleware::locale::current_request_locale() {
            return rl.t_blocking(key, args);
        }
        // Fallback: global rust_i18n with explicit pt-BR default locale.
        let raw = t!(key, locale = "pt-BR").into_owned();
        match args {
            Some(a) if !a.is_empty() => {
                let mut patterns: Vec<&str> = a.keys().map(String::as_str).collect();
                patterns.sort();
                let values: Vec<String> = patterns.iter().map(|p| a[*p].clone()).collect();
                rust_i18n::replace_patterns(&raw, &patterns, &values)
            },
            _ => raw,
        }
    }

    /// Same as [`translate_for_request`] but with no interpolation args.
    /// Used during struct construction (no request context available).
    fn translate_for_request_static(key: &str) -> String {
        Self::translate_for_request(key, None)
    }

    /// Resolve the per-request locale identifier, falling back to the
    /// translation-service default (pt-BR) when none is available.
    fn locale_for_request() -> String {
        crate::middleware::locale::current_request_locale()
            .map(|rl| rl.resolution.locale.clone())
            .unwrap_or_else(|| crate::services::translation_service::DEFAULT_LOCALE.to_string())
    }

    pub fn from_config(config: &AppConfig) -> Self {
        Self::new(config)
    }

    /// Construct a service for testing. Returns `(service, capture)` where the
    /// capture records every outgoing email — no HTTP requests are issued.
    pub fn for_test(config: &AppConfig) -> (Self, TestEmailCapture) {
        let mut s = Self::new(config);
        let capture = TestEmailCapture::new();
        s.capture = Some(capture.clone());
        (s, capture)
    }

    /// Whether test-capture mode is enabled (skipping HTTP delivery).
    pub fn is_capturing(&self) -> bool {
        self.capture.is_some()
    }

    /// Borrow the capture handle if enabled. Returns `None` in production mode.
    pub fn capture(&self) -> Option<&TestEmailCapture> {
        self.capture.as_ref()
    }

    pub fn is_configured(&self) -> bool {
        match &self.transport {
            EmailTransportImpl::Resend { api_key, .. } => !api_key.is_empty(),
            EmailTransportImpl::Smtp(_) => true,
        }
    }

    /// Which backend this service was configured to use.
    pub fn transport_kind(&self) -> EmailTransportKind {
        self.kind.clone()
    }

    /// Return the configured frontend base URL (used to build action URLs).
    pub fn frontend_url(&self) -> &str {
        &self.frontend_url
    }

    /// Return the templates instance if available (used for previews).
    pub fn templates(&self) -> Option<&EmailTemplates> {
        self.templates.as_ref()
    }

    /// Resolve a token into a full URL by joining with the configured frontend URL.
    ///
    /// If `path_or_url` already looks like an absolute URL, return it verbatim.
    fn resolve_url(&self, path_or_url: &str) -> String {
        if path_or_url.starts_with("http://")
            || path_or_url.starts_with("https://")
            || path_or_url.starts_with("//")
        {
            return path_or_url.to_string();
        }
        let base = self.frontend_url.trim_end_matches('/');
        let path = path_or_url.trim_start_matches('/');
        format!("{}/{}", base, path)
    }

    /// Send a plain text email
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> EmailResult {
        self.send_email_with_html(to, subject, body, None).await
    }

    /// Send an email with HTML body
    #[tracing::instrument(skip_all, fields(to = %to, subject = %subject, service = "resend"))]
    pub async fn send_email_with_html(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        html_body: Option<&str>,
    ) -> EmailResult {
        self.dispatch(to, subject, body, html_body, "").await
    }

    /// Internal dispatcher that records the originating template when
    /// capture-mode is enabled. Kept private to avoid leaking the
    /// `template` argument into the public API.
    async fn dispatch(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        html_body: Option<&str>,
        template: &str,
    ) -> EmailResult {
        // ---- Test capture short-circuit ----
        // When capture-mode is enabled we record the email and return without
        // hitting the network. This keeps test suites hermetic and provides
        // assertions on to/subject/body without mocking the HTTP client.
        if let Some(capture) = &self.capture {
            let safe_to = sanitize_for_email(to);
            let safe_subject = sanitize_for_email(subject);
            let safe_body = sanitize_for_html_email(body);
            let html = html_body
                .map(sanitize_for_html_email)
                .unwrap_or_else(|| self.wrap_html(&safe_subject, &safe_body));
            capture.capture(CapturedEmail {
                to: safe_to.clone(),
                subject: safe_subject.clone(),
                template: template.to_string(),
                text_body: safe_body.clone(),
                html_body: html,
                context: serde_json::json!({}),
                sent_at: Utc::now(),
            });
            tracing::debug!(to = %safe_to, subject = %safe_subject, "Email captured (test mode)");
            return Ok(());
        }

        if !self.is_configured() {
            tracing::warn!(
                "Email service not configured (transport={:?}); skipping email to {}",
                self.kind,
                to
            );
            return Err(EmailError::NotConfigured(Self::translate_for_request(
                "email.service_not_configured",
                None,
            )));
        }

        let safe_to = sanitize_for_email(to);
        let safe_subject = sanitize_for_email(subject);
        let safe_body = sanitize_for_html_email(body);
        let log_to = safe_to.clone();
        let log_subject = safe_subject.clone();

        let html = html_body
            .map(sanitize_for_html_email)
            .unwrap_or_else(|| self.wrap_html(&safe_subject, &safe_body));

        match &self.transport {
            EmailTransportImpl::Resend {
                client,
                api_key,
                base_url,
            } => {
                let request = ResendEmailRequest {
                    from: format!("{} <{}>", self.from_name, self.from_email),
                    to: vec![safe_to],
                    subject: safe_subject,
                    html,
                    text: Some(safe_body),
                };
                let url = format!("{}/emails", base_url);
                let response = client
                    .post(&url)
                    .await
                    .header("Authorization", &format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&request)
                    .send()
                    .await
                    .map_err(EmailError::HttpError)?;
                let status = response.status();
                let response_text = response.text().await.map_err(HttpClientError::HttpError)?;
                if status.is_success() {
                    let _resend_response: ResendEmailResponse =
                        serde_json::from_str(&response_text).map_err(|e| {
                            EmailError::SendFailed(format!("Invalid response: {}", e))
                        })?;
                    tracing::info!(to = %log_to, subject = %log_subject, "Email sent successfully");
                    Ok(())
                } else {
                    tracing::error!(
                        to = %log_to,
                        status = %status,
                        response = %response_text,
                        "Failed to send email via Resend"
                    );
                    Err(EmailError::SendFailed(format!(
                        "Resend API error ({}): {}",
                        status, response_text
                    )))
                }
            },
            EmailTransportImpl::Smtp(transport) => {
                let message = build_smtp_message(
                    &self.from_name,
                    &self.from_email,
                    &safe_to,
                    &safe_subject,
                    &safe_body,
                    &html,
                )?;
                let response = transport.send(message).await.map_err(EmailError::from)?;
                let code: u16 = response.code().into();
                if (200..300).contains(&code) {
                    tracing::info!(to = %log_to, subject = %log_subject, code = %code, "Email sent successfully via SMTP");
                    Ok(())
                } else {
                    let server_msg = response.message().collect::<Vec<_>>().join("; ");
                    tracing::error!(
                        to = %log_to,
                        code = %code,
                        response = %server_msg,
                        "Failed to send email via SMTP"
                    );
                    Err(EmailError::SendFailed(format!(
                        "SMTP error ({}): {}",
                        code, server_msg
                    )))
                }
            },
        }
    }

    fn wrap_html(&self, subject: &str, body: &str) -> String {
        let footer = {
            let mut args = std::collections::HashMap::new();
            args.insert("app".to_string(), self.from_name.clone());
            Self::translate_for_request("email.footer", Some(&args))
        };
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: #f8f9fa; border-radius: 8px; padding: 32px;">
        <h1 style="color: #1a1a2e; margin-top: 0;">{}</h1>
        <div style="white-space: pre-wrap;">{}</div>
    </div>
    <p style="color: #6c757d; font-size: 12px; text-align: center; margin-top: 24px;">
        {}
    </p>
</body>
</html>"#,
            subject, subject, body, footer
        )
    }

    /// Send account confirmation email
    pub async fn send_confirmation_email(&self, to: &str, confirm_url: &str) -> EmailResult {
        let subject = Self::translate_for_request("email.confirmation.subject", None);
        let resolved_url = self.resolve_url(confirm_url);

        let ctx = serde_json::json!({
            "user_name": "",
            "confirm_url": resolved_url,
            "to_email": to,
            "locale": Self::locale_for_request(),
        });

        let (html, text) = match self.render_pair(
            tpl::USER_CONFIRMATION_HTML,
            tpl::USER_CONFIRMATION_TEXT,
            &ctx,
        ) {
            Ok((html, text)) => (Some(html), text),
            Err(err) => {
                tracing::warn!(error = %err, "confirmation template render failed; sending text-only");
                let mut args = std::collections::HashMap::new();
                args.insert("url".to_string(), resolved_url);
                (
                    None,
                    Self::translate_for_request("email.confirmation.body_text", Some(&args)),
                )
            },
        };

        self.dispatch(
            to,
            &subject,
            &text,
            html.as_deref(),
            tpl::USER_CONFIRMATION_HTML,
        )
        .await
    }

    /// Send password reset email
    pub async fn send_password_reset_email(&self, to: &str, reset_url: &str) -> EmailResult {
        let subject = Self::translate_for_request("email.password_reset.subject", None);
        let resolved_url = self.resolve_url(reset_url);

        let ctx = serde_json::json!({
            "user_name": "",
            "reset_url": resolved_url,
            "to_email": to,
            "locale": Self::locale_for_request(),
        });

        let (html, text) = match self.render_pair(
            tpl::USER_PASSWORD_RESET_HTML,
            tpl::USER_PASSWORD_RESET_TEXT,
            &ctx,
        ) {
            Ok((html, text)) => (Some(html), text),
            Err(err) => {
                tracing::warn!(error = %err, "password reset template render failed; sending text-only");
                let mut args = std::collections::HashMap::new();
                args.insert("url".to_string(), resolved_url);
                (
                    None,
                    Self::translate_for_request("email.password_reset.body_text", Some(&args)),
                )
            },
        };

        self.dispatch(
            to,
            &subject,
            &text,
            html.as_deref(),
            tpl::USER_PASSWORD_RESET_HTML,
        )
        .await
    }

    /// Send 2FA setup email
    pub async fn send_2fa_setup_email(
        &self,
        to: &str,
        secret: &str,
        qr_code_url: &str,
        backup_codes: &[String],
    ) -> EmailResult {
        let subject = Self::translate_for_request("email.two_factor_setup.subject", None);
        let backup_codes_text = backup_codes.join(", ");

        let ctx = serde_json::json!({
            "user_name": "",
            "secret": secret,
            "qr_code_url": qr_code_url,
            "backup_codes_text": backup_codes_text,
            "to_email": to,
            "locale": Self::locale_for_request(),
        });

        let (html, text) = match self.render_pair(
            tpl::USER_TWO_FACTOR_SETUP_HTML,
            tpl::USER_TWO_FACTOR_SETUP_TEXT,
            &ctx,
        ) {
            Ok((html, text)) => (Some(html), text),
            Err(err) => {
                tracing::warn!(error = %err, "2fa setup template render failed; sending text-only");
                let mut args = std::collections::HashMap::new();
                args.insert("secret".to_string(), secret.to_string());
                args.insert("qr".to_string(), qr_code_url.to_string());
                args.insert("codes".to_string(), backup_codes_text);
                let body =
                    Self::translate_for_request("email.two_factor_setup.body_text", Some(&args));
                (None, body)
            },
        };

        self.dispatch(
            to,
            &subject,
            &text,
            html.as_deref(),
            tpl::USER_TWO_FACTOR_SETUP_HTML,
        )
        .await
    }

    /// Send password changed notification
    pub async fn send_password_changed_notification(&self, to: &str) -> EmailResult {
        let subject = Self::translate_for_request("email.password_changed.subject", None);

        let ctx = serde_json::json!({
            "user_name": "",
            "to_email": to,
            "locale": Self::locale_for_request(),
        });

        let (html, text) = match self.render_pair(
            tpl::USER_PASSWORD_CHANGED_HTML,
            tpl::USER_PASSWORD_CHANGED_TEXT,
            &ctx,
        ) {
            Ok((html, text)) => (Some(html), text),
            Err(err) => {
                tracing::warn!(error = %err, "password changed template render failed; sending text-only");
                (
                    None,
                    Self::translate_for_request("email.password_changed.body_text", None),
                )
            },
        };

        self.dispatch(
            to,
            &subject,
            &text,
            html.as_deref(),
            tpl::USER_PASSWORD_CHANGED_HTML,
        )
        .await
    }

    /// Alias for backward compatibility
    pub async fn send_password_reset(&self, to: &str, token: &str) -> EmailResult {
        let reset_url = format!("/auth/reset?token={}", token);
        self.send_password_reset_email(to, &reset_url).await
    }

    /// Send a magic link email.
    ///
    /// The link is short-lived (15 min) and single-use, so it renders its own
    /// dedicated template instead of reusing the password reset one.
    pub async fn send_magic_link(&self, to: &str, token: &str) -> EmailResult {
        let magic_url = format!("/auth/magic-link/verify?token={}", token);
        self.send_magic_link_email(to, &magic_url).await
    }

    /// Send a magic link email with a pre-built URL.
    pub async fn send_magic_link_email(&self, to: &str, magic_url: &str) -> EmailResult {
        let subject = Self::translate_for_request("email.magic_link.subject", None);
        let resolved_url = self.resolve_url(magic_url);

        let ctx = serde_json::json!({
            "user_name": "",
            "magic_url": resolved_url,
            "to_email": to,
            "locale": Self::locale_for_request(),
        });

        let (html, text) = match self.render_pair(
            tpl::USER_MAGIC_LINK_HTML,
            tpl::USER_MAGIC_LINK_TEXT,
            &ctx,
        ) {
            Ok((html, text)) => (Some(html), text),
            Err(err) => {
                tracing::warn!(error = %err, "magic link template render failed; sending text-only");
                let mut args = std::collections::HashMap::new();
                args.insert("url".to_string(), resolved_url);
                (
                    None,
                    Self::translate_for_request("email.magic_link.body_text", Some(&args)),
                )
            },
        };

        self.dispatch(
            to,
            &subject,
            &text,
            html.as_deref(),
            tpl::USER_MAGIC_LINK_HTML,
        )
        .await
    }

    /// Alias for backward compatibility
    pub async fn send_confirmation(&self, to: &str, token: &str) -> EmailResult {
        let confirm_url = format!("/auth/confirm?token={}", token);
        self.send_confirmation_email(to, &confirm_url).await
    }

    /// Render an HTML+text pair from templates using the mailer layout.
    fn render_pair(
        &self,
        html_template: &str,
        text_template: &str,
        ctx: &serde_json::Value,
    ) -> Result<(String, String), EmailError> {
        let templates =
            self.templates
                .as_ref()
                .ok_or(EmailError::Template(EmailTemplateError::NotFound(
                    "templates not loaded".to_string(),
                )))?;
        let html = templates.render_html_with_layout(html_template, ctx)?;
        let text = templates.render(text_template, ctx)?;
        Ok((html, text))
    }
}

impl Default for EmailService {
    fn default() -> Self {
        Self::new(&test_config())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_html() {
        let config = test_config();
        let service = EmailService::new(&config);
        let html = service.wrap_html("Test Subject", "Test body");
        assert!(html.contains("Test Subject"));
        assert!(html.contains("Test body"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_is_configured_empty_key() {
        let mut config = test_config();
        config.resend_api_key = String::new();
        let service = EmailService::new(&config);
        assert!(!service.is_configured());
    }

    #[test]
    fn test_is_configured_with_key() {
        let mut config = test_config();
        config.resend_api_key = "test_key".to_string();
        let service = EmailService::new(&config);
        assert!(service.is_configured());
    }

    #[test]
    fn templates_load_successfully() {
        let config = test_config();
        let service = EmailService::new(&config);
        assert!(service.templates().is_some(), "templates should load");
    }

    #[test]
    fn resolve_url_joins_relative_paths() {
        let config = test_config();
        let service = EmailService::new(&config);
        let url = service.resolve_url("/auth/confirm?token=abc");
        assert!(url.starts_with("http://localhost:3000"));
        assert!(url.contains("/auth/confirm?token=abc"));
    }

    #[test]
    fn resolve_url_passes_through_absolute_urls() {
        let config = test_config();
        let service = EmailService::new(&config);
        let url = service.resolve_url("https://example.com/x");
        assert_eq!(url, "https://example.com/x");
    }

    #[test]
    fn smtp_backend_is_always_configured() {
        // When transport is Smtp, `is_configured` returns true regardless of
        // RESEND_API_KEY, because SMTP delivery doesn't need an API key
        // (mailcatcher and unauthenticated relays are valid).
        let mut config = test_config();
        config.email_transport = EmailTransportKind::Smtp;
        config.smtp_url = "smtp://localhost:1025/?tls=opportunistic".to_string();
        config.resend_api_key = String::new();
        let service = EmailService::new(&config);
        assert!(service.is_configured());
        assert_eq!(service.transport_kind(), EmailTransportKind::Smtp);
    }

    #[test]
    fn smtp_backend_builds_even_with_invalid_url() {
        // A malformed SMTP_URL should still construct the service (so the app
        // can boot) but the first send will surface the underlying error.
        let mut config = test_config();
        config.email_transport = EmailTransportKind::Smtp;
        config.smtp_url = "this-is-not-a-valid-smtp-url".to_string();
        let service = EmailService::new(&config);
        assert!(service.is_configured());
    }

    #[test]
    fn resend_backend_unconfigured_without_api_key() {
        let mut config = test_config();
        config.email_transport = EmailTransportKind::Resend;
        config.resend_api_key = String::new();
        let service = EmailService::new(&config);
        assert!(!service.is_configured());
        assert_eq!(service.transport_kind(), EmailTransportKind::Resend);
    }
}
