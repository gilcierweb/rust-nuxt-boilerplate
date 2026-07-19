#![allow(dead_code)]

use crate::config::app_config::{AppConfig, Environment, JwtSecretKey};
use crate::services::email_templates::{EmailTemplateError, EmailTemplates, names as tpl};
use crate::services::email_test_capture::{CapturedEmail, TestEmailCapture};
use crate::services::http_client::{HttpClient, HttpClientConfig, HttpClientError};
use crate::utils::sanitize::{sanitize_for_email, sanitize_for_html_email};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Deterministic test key generation
fn generate_deterministic_string(length: usize, seed: u64) -> String {
    use rand::Rng;
    use rand::SeedableRng;
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
    use rand::RngCore;
    use rand::SeedableRng;
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
        argon2_m_cost: 65536,
        argon2_t_cost: 3,
        argon2_p_cost: 1,
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
    #[error("Template error: {0}")]
    Template(#[from] EmailTemplateError),
}

pub type EmailResult = Result<(), EmailError>;

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

pub struct EmailService {
    client: HttpClient,
    api_key: String,
    from_email: String,
    from_name: String,
    base_url: String,
    frontend_url: String,
    templates: Option<EmailTemplates>,
    /// Optional in-memory capture used in test mode. When `Some`, outbound
    /// HTTP requests are skipped and the rendered email is recorded instead.
    capture: Option<TestEmailCapture>,
}

impl EmailService {
    pub fn new(config: &AppConfig) -> Self {
        let api_key = config.resend_api_key.clone();
        let from_email = config.email_from.clone();
        let from_name = if config.email_from_name.is_empty() {
            t!("app.name").into_owned()
        } else {
            config.email_from_name.clone()
        };
        let base_url = "https://api.resend.com".to_string();
        let frontend_url = config.frontend_url.clone();

        let http_config = HttpClientConfig {
            timeout: std::time::Duration::from_secs(10),
            max_retries: 3,
            retry_base_delay: std::time::Duration::from_millis(100),
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: std::time::Duration::from_secs(60),
        };

        let client = HttpClient::new(http_config).expect("Failed to create HTTP client");

        // Template loading is best-effort: if templates fail to compile (which
        // is statically impossible because they are bundled), we fall back to
        // the legacy inline HTML implementations.
        let templates = match EmailTemplates::new() {
            Ok(t) => Some(t),
            Err(err) => {
                tracing::error!(error = %err, "failed to load email templates; falling back to inline HTML");
                None
            },
        };

        Self {
            client,
            api_key,
            from_email,
            from_name,
            base_url,
            frontend_url,
            templates,
            capture: None,
        }
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
        !self.api_key.is_empty()
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
                "Email service not configured (missing RESEND_API_KEY), skipping email to {}",
                to
            );
            return Err(EmailError::NotConfigured(
                t!("email.service_not_configured").into_owned(),
            ));
        }

        let safe_to = sanitize_for_email(to);
        let safe_subject = sanitize_for_email(subject);
        let safe_body = sanitize_for_html_email(body);

        // Clone for logging since they'll be moved into request
        let log_to = safe_to.clone();
        let log_subject = safe_subject.clone();

        let html = html_body
            .map(sanitize_for_html_email)
            .unwrap_or_else(|| self.wrap_html(&safe_subject, &safe_body));

        let request = ResendEmailRequest {
            from: format!("{} <{}>", self.from_name, self.from_email),
            to: vec![safe_to],
            subject: safe_subject,
            html,
            text: Some(safe_body),
        };

        let url = format!("{}/emails", self.base_url);

        let response = self
            .client
            .post(&url)
            .await
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(EmailError::HttpError)?;

        let status = response.status();
        let response_text = response.text().await.map_err(HttpClientError::HttpError)?;

        if status.is_success() {
            let _resend_response: ResendEmailResponse = serde_json::from_str(&response_text)
                .map_err(|e| EmailError::SendFailed(format!("Invalid response: {}", e)))?;
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
    }

    fn wrap_html(&self, subject: &str, body: &str) -> String {
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
            subject,
            subject,
            body,
            t!("email.footer", app = self.from_name).into_owned()
        )
    }

    /// Send account confirmation email
    pub async fn send_confirmation_email(&self, to: &str, confirm_url: &str) -> EmailResult {
        let subject = t!("email.confirmation.subject").into_owned();
        let resolved_url = self.resolve_url(confirm_url);

        let ctx = serde_json::json!({
            "user_name": "",
            "confirm_url": resolved_url,
            "to_email": to,
        });

        let (html, text) = self
            .render_pair(tpl::USER_CONFIRMATION_HTML, tpl::USER_CONFIRMATION_TEXT, &ctx)
            .unwrap_or_else(|err| {
                tracing::warn!(error = %err, "confirmation template render failed; using inline fallback");
                let body = t!("email.confirmation.body_text", url = resolved_url).into_owned();
                let html = self.confirmation_html_fallback(&resolved_url);
                (html, body)
            });

        self.dispatch(
            to,
            &subject,
            &text,
            Some(&html),
            tpl::USER_CONFIRMATION_HTML,
        )
        .await
    }

    /// Send password reset email
    pub async fn send_password_reset_email(&self, to: &str, reset_url: &str) -> EmailResult {
        let subject = t!("email.password_reset.subject").into_owned();
        let resolved_url = self.resolve_url(reset_url);

        let ctx = serde_json::json!({
            "user_name": "",
            "reset_url": resolved_url,
            "to_email": to,
        });

        let (html, text) = self
            .render_pair(tpl::USER_PASSWORD_RESET_HTML, tpl::USER_PASSWORD_RESET_TEXT, &ctx)
            .unwrap_or_else(|err| {
                tracing::warn!(error = %err, "password reset template render failed; using inline fallback");
                let body = t!("email.password_reset.body_text", url = resolved_url).into_owned();
                let html = self.password_reset_html_fallback(&resolved_url);
                (html, body)
            });

        self.dispatch(
            to,
            &subject,
            &text,
            Some(&html),
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
        let subject = t!("email.two_factor_setup.subject").into_owned();
        let backup_codes_text = backup_codes.join(", ");

        let ctx = serde_json::json!({
            "user_name": "",
            "secret": secret,
            "qr_code_url": qr_code_url,
            "backup_codes_text": backup_codes_text,
            "to_email": to,
        });

        let (html, text) = self
            .render_pair(tpl::USER_TWO_FACTOR_SETUP_HTML, tpl::USER_TWO_FACTOR_SETUP_TEXT, &ctx)
            .unwrap_or_else(|err| {
                tracing::warn!(error = %err, "2fa setup template render failed; using inline fallback");
                let body = t!(
                    "email.two_factor_setup.body_text",
                    secret = secret,
                    qr = qr_code_url,
                    codes = backup_codes_text
                )
                .into_owned();
                let html = self.two_factor_setup_html_fallback(secret, qr_code_url, &backup_codes_text);
                (html, body)
            });

        self.dispatch(
            to,
            &subject,
            &text,
            Some(&html),
            tpl::USER_TWO_FACTOR_SETUP_HTML,
        )
        .await
    }

    /// Send password changed notification
    pub async fn send_password_changed_notification(&self, to: &str) -> EmailResult {
        let subject = t!("email.password_changed.subject").into_owned();

        let ctx = serde_json::json!({
            "user_name": "",
            "to_email": to,
        });

        let (html, text) = self
            .render_pair(tpl::USER_PASSWORD_CHANGED_HTML, tpl::USER_PASSWORD_CHANGED_TEXT, &ctx)
            .unwrap_or_else(|err| {
                tracing::warn!(error = %err, "password changed template render failed; using inline fallback");
                let body = t!("email.password_changed.body_text").into_owned();
                let html = self.password_changed_html_fallback();
                (html, body)
            });

        self.dispatch(
            to,
            &subject,
            &text,
            Some(&html),
            tpl::USER_PASSWORD_CHANGED_HTML,
        )
        .await
    }

    /// Alias for backward compatibility
    pub async fn send_password_reset(&self, to: &str, token: &str) -> EmailResult {
        let reset_url = format!("/auth/reset?token={}", token);
        self.send_password_reset_email(to, &reset_url).await
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

    // ---- Inline HTML fallbacks (used when templates are unavailable) ----

    fn confirmation_html_fallback(&self, confirm_url: &str) -> String {
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
        <p>{}</p>
        <p style="text-align: center; margin: 32px 0;">
            <a href="{}" style="background: #1a1a2e; color: white; padding: 14px 28px; border-radius: 6px; text-decoration: none; font-weight: 600; display: inline-block;">
                {}
            </a>
        </p>
        <p style="color: #6c757d; font-size: 14px;">{} <a href="{}">{}</a></p>
        <p style="color: #6c757d; font-size: 14px;">{}</p>
    </div>
    <p style="color: #6c757d; font-size: 12px; text-align: center; margin-top: 24px;">
        {}
    </p>
</body>
</html>"#,
            t!("email.confirmation.html_title").into_owned(),
            t!("email.confirmation.html_heading").into_owned(),
            t!("email.confirmation.html_body").into_owned(),
            confirm_url,
            t!("email.confirmation.html_button").into_owned(),
            t!("email.confirmation.fallback_instruction").into_owned(),
            confirm_url,
            confirm_url,
            t!("email.confirmation.expiry_notice").into_owned(),
            t!("email.footer", app = self.from_name).into_owned()
        )
    }

    fn password_reset_html_fallback(&self, reset_url: &str) -> String {
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
        <p>{}</p>
        <p style="text-align: center; margin: 32px 0;">
            <a href="{}" style="background: #dc2626; color: white; padding: 14px 28px; border-radius: 6px; text-decoration: none; font-weight: 600; display: inline-block;">
                {}
            </a>
        </p>
        <p style="color: #6c757d; font-size: 14px;">{} <a href="{}">{}</a></p>
        <p style="color: #6c757d; font-size: 14px;">{}</p>
        <p style="color: #6c757d; font-size: 14px;">{}</p>
    </div>
    <p style="color: #6c757d; font-size: 12px; text-align: center; margin-top: 24px;">
        {}
    </p>
</body>
</html>"#,
            t!("email.password_reset.html_title").into_owned(),
            t!("email.password_reset.html_heading").into_owned(),
            t!("email.password_reset.html_body").into_owned(),
            reset_url,
            t!("email.password_reset.html_button").into_owned(),
            t!("email.password_reset.fallback_instruction").into_owned(),
            reset_url,
            reset_url,
            t!("email.password_reset.expiry_notice").into_owned(),
            t!("email.password_reset.html_warning").into_owned(),
            t!("email.footer", app = self.from_name).into_owned()
        )
    }

    fn two_factor_setup_html_fallback(
        &self,
        secret: &str,
        qr_code_url: &str,
        backup_codes_text: &str,
    ) -> String {
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
        <p>{}</p>
        <div style="background: white; border: 1px solid #dee2e6; border-radius: 6px; padding: 16px; margin: 16px 0; font-family: monospace; word-break: break-all;">
            <strong>{}</strong> {}
        </div>
        <p><strong>{}</strong> <a href="{}">{}</a></p>
        <h2>{}</h2>
        <div style="background: #fff3cd; border: 1px solid #ffc107; border-radius: 6px; padding: 16px;">
            <p style="font-family: monospace; word-break: break-all;">{}</p>
        </div>
    </div>
    <p style="color: #6c757d; font-size: 12px; text-align: center; margin-top: 24px;">
        {}
    </p>
</body>
</html>"#,
            t!("email.two_factor_setup.html_title").into_owned(),
            t!("email.two_factor_setup.html_heading").into_owned(),
            t!("email.two_factor_setup.html_body").into_owned(),
            t!("email.two_factor_setup.secret_label").into_owned(),
            secret,
            t!("email.two_factor_setup.qr_heading").into_owned(),
            qr_code_url,
            qr_code_url,
            t!("email.two_factor_setup.backup_heading").into_owned(),
            backup_codes_text,
            t!("email.footer", app = self.from_name).into_owned()
        )
    }

    fn password_changed_html_fallback(&self) -> String {
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
        <h1 style="color: #dc2626; margin-top: 0;">{}</h1>
        <p>{}</p>
        <p style="color: #dc2626;"><strong>{}</strong></p>
    </div>
    <p style="color: #6c757d; font-size: 12px; text-align: center; margin-top: 24px;">
        {}
    </p>
</body>
</html>"#,
            t!("email.password_changed.html_title").into_owned(),
            t!("email.password_changed.html_heading").into_owned(),
            t!("email.password_changed.html_body").into_owned(),
            t!("email.password_changed.html_warning").into_owned(),
            t!("email.footer", app = self.from_name).into_owned()
        )
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
}
