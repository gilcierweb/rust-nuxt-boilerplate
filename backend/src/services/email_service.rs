#![allow(dead_code)]

use crate::config::app_config::AppConfig;

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

    pub async fn send_email(&self, _to: &str, _subject: &str, _body: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn send_confirmation(&self, _email: &str, _token: &str) -> Result<(), String> {
        Ok(())
    }

    pub async fn send_password_reset(&self, _email: &str, _token: &str) -> Result<(), String> {
        Ok(())
    }
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
        let result = service.send_email("test@example.com", "Subject", "Body").await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn send_confirmation_returns_ok() {
        let service = EmailService::new();
        let result = service.send_confirmation("test@example.com", "token123").await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn send_password_reset_returns_ok() {
        let service = EmailService::new();
        let result = service.send_password_reset("test@example.com", "token123").await;
        assert!(result.is_ok());
    }
}
