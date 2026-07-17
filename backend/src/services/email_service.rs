#![allow(dead_code)]

use crate::config::app_config::AppConfig;
use crate::config::app_config::Environment;
use crate::config::app_config::JwtSecretKey;
use crate::utils::sanitize::{sanitize_for_email, sanitize_for_html_email};
use reqwest::Client;
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
        db_statement_timeout_secs: Some(30),
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

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Email service not configured: {0}")]
    NotConfigured(String),
    #[error("Failed to send email: {0}")]
    SendFailed(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
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
    client: Client,
    api_key: String,
    from_email: String,
    from_name: String,
    base_url: String,
}

impl EmailService {
    pub fn new(config: &AppConfig) -> Self {
        let api_key = config.resend_api_key.clone();
        let from_email = config.email_from.clone();
        let from_name = "Boilerplate App".to_string();
        let base_url = "https://api.resend.com".to_string();

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            from_email,
            from_name,
            base_url,
        }
    }

    pub fn from_config(config: &AppConfig) -> Self {
        Self::new(config)
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    /// Send a plain text email
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> EmailResult {
        self.send_email_with_html(to, subject, body, None).await
    }

    /// Send an email with HTML body
    pub async fn send_email_with_html(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        html_body: Option<&str>,
    ) -> EmailResult {
        if !self.is_configured() {
            tracing::warn!(
                "Email service not configured (missing RESEND_API_KEY), skipping email to {}",
                to
            );
            return Err(EmailError::NotConfigured(
                "RESEND_API_KEY not set".to_string(),
            ));
        }

        let safe_to = sanitize_for_email(to);
        let safe_subject = sanitize_for_email(subject);
        let safe_body = sanitize_for_html_email(body);

        // Clone for logging since they'll be moved into request
        let log_to = safe_to.clone();
        let log_subject = safe_subject.clone();

        let html = html_body
            .map(|h| sanitize_for_html_email(h))
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

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
        This email was sent by Boilerplate App
    </p>
</body>
</html>"#,
            subject, subject, body
        )
    }

    /// Send account confirmation email
    pub async fn send_confirmation_email(&self, to: &str, confirm_url: &str) -> EmailResult {
        let subject = "Confirm your email address";
        let body = format!(
            "Please click the link below to confirm your email address:\n\n{}\n\nThis link will expire in 24 hours.",
            confirm_url
        );
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Confirm your email</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: #f8f9fa; border-radius: 8px; padding: 32px;">
        <h1 style="color: #1a1a2e; margin-top: 0;">Confirm your email address</h1>
        <p>Thank you for registering! Please click the button below to confirm your email address:</p>
        <p style="text-align: center; margin: 32px 0;">
            <a href="{}" style="background: #1a1a2e; color: white; padding: 14px 28px; border-radius: 6px; text-decoration: none; font-weight: 600; display: inline-block;">
                Confirm Email
            </a>
        </p>
        <p style="color: #6c757d; font-size: 14px;">Or copy this link: <a href="{}">{}</a></p>
        <p style="color: #6c757d; font-size: 14px;">This link will expire in 24 hours.</p>
    </div>
    <p style="color: #6c757d; font-size: 12px; text-align: center; margin-top: 24px;">
        This email was sent by Boilerplate App
    </p>
</body>
</html>"#,
            confirm_url, confirm_url, confirm_url
        );

        self.send_email_with_html(to, subject, &body, Some(&html))
            .await
    }

    /// Send password reset email
    pub async fn send_password_reset_email(&self, to: &str, reset_url: &str) -> EmailResult {
        let subject = "Reset your password";
        let body = format!(
            "Click the link below to reset your password:\n\n{}\n\nThis link will expire in 1 hour.",
            reset_url
        );
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reset your password</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: #f8f9fa; border-radius: 8px; padding: 32px;">
        <h1 style="color: #1a1a2e; margin-top: 0;">Reset your password</h1>
        <p>You requested a password reset. Click the button below to create a new password:</p>
        <p style="text-align: center; margin: 32px 0;">
            <a href="{}" style="background: #dc2626; color: white; padding: 14px 28px; border-radius: 6px; text-decoration: none; font-weight: 600; display: inline-block;">
                Reset Password
            </a>
        </p>
        <p style="color: #6c757d; font-size: 14px;">Or copy this link: <a href="{}">{}</a></p>
        <p style="color: #6c757d; font-size: 14px;">This link will expire in 1 hour.</p>
        <p style="color: #6c757d; font-size: 14px;">If you didn't request this, please ignore this email.</p>
    </div>
    <p style="color: #6c757d; font-size: 12px; text-align: center; margin-top: 24px;">
        This email was sent by Boilerplate App
    </p>
</body>
</html>"#,
            reset_url, reset_url, reset_url
        );

        self.send_email_with_html(to, subject, &body, Some(&html))
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
        let subject = "Your 2FA setup codes";
        let backup_codes_text = backup_codes.join(", ");
        let body = format!(
            "Your 2FA secret: {}\n\nQR Code: {}\n\nBackup codes (save these!):\n{}",
            secret, qr_code_url, backup_codes_text
        );
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>2FA Setup</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: #f8f9fa; border-radius: 8px; padding: 32px;">
        <h1 style="color: #1a1a2e; margin-top: 0;">Two-Factor Authentication Setup</h1>
        <p>Your 2FA has been enabled. Here are your setup details:</p>
        <div style="background: white; border: 1px solid #dee2e6; border-radius: 6px; padding: 16px; margin: 16px 0; font-family: monospace; word-break: break-all;">
            <strong>Secret:</strong> {}
        </div>
        <p><strong>QR Code:</strong> <a href="{}">{}</a></p>
        <h2>Backup Codes (save these!)</h2>
        <div style="background: #fff3cd; border: 1px solid #ffc107; border-radius: 6px; padding: 16px;">
            <p style="font-family: monospace; word-break: break-all;">{}</p>
        </div>
    </div>
    <p style="color: #6c757d; font-size: 12px; text-align: center; margin-top: 24px;">
        This email was sent by Boilerplate App
    </p>
</body>
</html>"#,
            secret, qr_code_url, qr_code_url, backup_codes_text
        );

        self.send_email_with_html(to, subject, &body, Some(&html))
            .await
    }

    /// Send password changed notification
    pub async fn send_password_changed_notification(&self, to: &str) -> EmailResult {
        let subject = "Your password has been changed";
        let body = "Your password was successfully changed. If you didn't make this change, please contact support immediately.";
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Password Changed</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: #f8f9fa; border-radius: 8px; padding: 32px;">
        <h1 style="color: #dc2626; margin-top: 0;">Password Changed</h1>
        <p>Your password was successfully changed.</p>
        <p style="color: #dc2626;"><strong>If you didn't make this change, please contact support immediately.</strong></p>
    </div>
    <p style="color: #6c757d; font-size: 12px; text-align: center; margin-top: 24px;">
        This email was sent by Boilerplate App
    </p>
</body>
</html>"#
        );

        self.send_email_with_html(to, subject, body, Some(&html))
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
}
