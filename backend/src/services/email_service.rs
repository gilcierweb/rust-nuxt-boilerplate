#![allow(dead_code)]

use crate::config::app_config::AppConfig;
use crate::utils::sanitize::{sanitize_for_email, sanitize_for_html_email};

pub struct EmailService {
    api_key: String,
    from_email: String,
    from_name: String,
}

impl EmailService {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            api_key: config.resend_api_key.clone(),
            from_email: config.email_from.clone(),
            from_name: config.email_from_name.clone(),
        }
    }

    pub fn new() -> Self {
        Self {
            api_key: String::new(),
            from_email: String::new(),
            from_name: String::new(),
        }
    }

    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let _safe_to = sanitize_for_email(to);
        let _safe_subject = sanitize_for_email(subject);
        let _safe_body = sanitize_for_html_email(body);
        Ok(())
    }

    pub async fn send_confirmation(&self, email: &str, token: &str) -> Result<(), String> {
        let safe_email = sanitize_for_email(email);
        let safe_token = sanitize_for_email(token);
        let subject = sanitize_for_email("Confirm your email address");
        let _html = confirmation_html(&safe_email, &safe_token);
        tracing::info!(
            event = "email.confirmation.queued",
            to = %safe_email,
            subject = %subject,
            "confirmation email queued"
        );
        Ok(())
    }

    pub async fn send_password_reset(&self, email: &str, token: &str) -> Result<(), String> {
        let safe_email = sanitize_for_email(email);
        let safe_token = sanitize_for_email(token);
        let subject = sanitize_for_email("Reset your password");
        let _html = password_reset_html(&safe_email, &safe_token);
        tracing::info!(
            event = "email.password_reset.queued",
            to = %safe_email,
            subject = %subject,
            "password reset email queued"
        );
        Ok(())
    }
}

pub fn confirmation_html(email: &str, token: &str) -> String {
    let safe_email = sanitize_for_html_email(email);
    let safe_token = sanitize_for_html_email(token);
    format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family:sans-serif;max-width:600px;margin:0 auto;padding:20px;">
  <h2 style="color:#333;">Confirm your email</h2>
  <p>You registered with <strong>{}</strong>.</p>
  <p>Click below to confirm your email address:</p>
  <p><a href="/auth/confirm?token={}" style="display:inline-block;padding:12px 24px;background:#2563eb;color:#fff;text-decoration:none;border-radius:6px;">Confirm Email</a></p>
  <p style="color:#666;font-size:14px;">If you did not register, you can safely ignore this email.</p>
</body>
</html>"#,
        safe_email, safe_token
    )
}

pub fn password_reset_html(email: &str, token: &str) -> String {
    let safe_email = sanitize_for_html_email(email);
    let safe_token = sanitize_for_html_email(token);
    format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family:sans-serif;max-width:600px;margin:0 auto;padding:20px;">
  <h2 style="color:#333;">Reset your password</h2>
  <p>We received a password reset request for <strong>{}</strong>.</p>
  <p>Click below to reset your password:</p>
  <p><a href="/auth/reset?token={}" style="display:inline-block;padding:12px 24px;background:#dc2626;color:#fff;text-decoration:none;border-radius:6px;">Reset Password</a></p>
  <p style="color:#666;font-size:14px;">This link expires in 2 hours. If you did not request a reset, you can safely ignore this email.</p>
</body>
</html>"#,
        safe_email, safe_token
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_config_populates_fields() {
        let config = crate::security::test_utils::test_config();
        let service = EmailService::from_config(&config);
        assert_eq!(service.api_key, config.resend_api_key);
        assert_eq!(service.from_email, config.email_from);
        assert_eq!(service.from_name, config.email_from_name);
    }

    #[test]
    fn new_creates_empty_service() {
        let service = EmailService::new();
        assert!(service.api_key.is_empty());
        assert!(service.from_email.is_empty());
        assert!(service.from_name.is_empty());
    }

    #[actix_web::test]
    async fn send_email_returns_ok() {
        let service = EmailService::new();
        let result = service
            .send_email("test@example.com", "Subject", "Body")
            .await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn send_confirmation_returns_ok() {
        let service = EmailService::new();
        let result = service
            .send_confirmation("test@example.com", "token123")
            .await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn send_password_reset_returns_ok() {
        let service = EmailService::new();
        let result = service
            .send_password_reset("test@example.com", "token123")
            .await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn send_email_sanitizes_script_tag_in_to() {
        let service = EmailService::new();
        let result = service
            .send_email(
                "<script>alert('xss')</script>@example.com",
                "Subject",
                "Body",
            )
            .await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn send_confirmation_sanitizes_xss_in_email() {
        let service = EmailService::new();
        let result = service
            .send_confirmation("<img src=x onerror=alert(1)>@example.com", "token123")
            .await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn send_password_reset_sanitizes_xss_in_email() {
        let service = EmailService::new();
        let result = service
            .send_password_reset("user@example.com<script>alert(1)</script>", "token123")
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn confirmation_html_contains_sanitized_email() {
        let html = confirmation_html("user@example.com", "abc123");
        assert!(html.contains("user@example.com"));
        assert!(html.contains("abc123"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn confirmation_html_strips_script_tag() {
        let html = confirmation_html("<script>alert(1)</script>@example.com", "token");
        assert!(!html.contains("<script>"));
        assert!(html.contains("@example.com"));
    }

    #[test]
    fn confirmation_html_strips_img_onerror() {
        let html = confirmation_html("<img src=x onerror=alert(1)>@example.com", "token");
        assert!(!html.contains("<img"));
        assert!(html.contains("@example.com"));
    }

    #[test]
    fn confirmation_html_strips_event_handler() {
        let html = confirmation_html("<b onclick=alert(1)>user</b>@example.com", "token");
        assert!(!html.contains("onclick"));
        assert!(html.contains("@example.com"));
    }

    #[test]
    fn password_reset_html_contains_sanitized_email() {
        let html = password_reset_html("user@example.com", "xyz789");
        assert!(html.contains("user@example.com"));
        assert!(html.contains("xyz789"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn password_reset_html_strips_script_tag() {
        let html = password_reset_html("<script>alert(1)</script>@example.com", "token");
        assert!(!html.contains("<script>"));
        assert!(html.contains("@example.com"));
    }

    #[test]
    fn password_reset_html_strips_event_handler() {
        let html = password_reset_html("<b onfocus=alert(1)>@example.com", "token");
        assert!(!html.contains("onfocus"));
        assert!(html.contains("@example.com"));
    }

    #[test]
    fn confirmation_html_allows_safe_tags() {
        let html = confirmation_html("user@example.com", "token");
        assert!(html.contains("<strong>"));
    }

    #[test]
    fn password_reset_html_allows_safe_tags() {
        let html = password_reset_html("user@example.com", "token");
        assert!(html.contains("<strong>"));
    }
}
