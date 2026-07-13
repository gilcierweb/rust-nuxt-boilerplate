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
