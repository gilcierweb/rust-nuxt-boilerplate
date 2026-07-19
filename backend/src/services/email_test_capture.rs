//! In-memory email capture for tests.
//!
//! When `EmailService` is constructed in test mode (`EmailService::for_test`),
//! every outbound email is recorded in a shared [`TestEmailCapture`] backend
//! instead of being sent via Resend. This mirrors Rails'
//! `ActionMailer::Base.deliveries` and lets tests assert on captured
//! recipients, subjects, and rendered bodies without going through HTTP.
//!
//! Thread-safe (`parking_lot::Mutex`). Clone-safe (uses [`Arc`] internally).
//!
//! Optionally usable from outside the email crate: import via
//! `crate::services::email_test_capture::TestEmailCapture`.

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use std::sync::Arc;

/// A captured email payload. Mirrors the public-facing fields of an outbound
/// email without exposing raw HTTP request bodies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedEmail {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub text_body: String,
    pub html_body: String,
    pub context: serde_json::Value,
    pub sent_at: DateTime<Utc>,
}

/// In-memory email capture backed by a `Mutex<Vec<…>>`.
///
/// Cloning yields a new handle pointing to the same backing buffer.
#[derive(Clone, Default)]
pub struct TestEmailCapture {
    inner: Arc<Mutex<Vec<CapturedEmail>>>,
}

impl TestEmailCapture {
    /// Construct a fresh, empty capture.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a captured email. Called internally by `EmailService`
    /// when running with `capture_outbound_emails = true`.
    pub fn capture(&self, email: CapturedEmail) {
        self.inner.lock().push(email);
    }

    /// Snapshot of all captured emails.
    pub fn get_all(&self) -> Vec<CapturedEmail> {
        self.inner.lock().clone()
    }

    /// Clear every captured email from the buffer.
    pub fn clear(&self) {
        self.inner.lock().clear();
    }

    /// Count of captured emails.
    pub fn count(&self) -> usize {
        self.inner.lock().len()
    }

    /// The most recently captured email, if any.
    pub fn last(&self) -> Option<CapturedEmail> {
        self.inner.lock().last().cloned()
    }

    /// All emails sent to the given recipient.
    pub fn find_by_to(&self, to: &str) -> Vec<CapturedEmail> {
        self.inner
            .lock()
            .iter()
            .filter(|email| email.to == to)
            .cloned()
            .collect()
    }

    /// All emails that were dispatched using the supplied template name.
    pub fn find_by_template(&self, template: &str) -> Vec<CapturedEmail> {
        self.inner
            .lock()
            .iter()
            .filter(|email| email.template == template)
            .cloned()
            .collect()
    }
}

impl std::fmt::Debug for TestEmailCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestEmailCapture")
            .field("count", &self.count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(to: &str) -> CapturedEmail {
        CapturedEmail {
            to: to.to_string(),
            subject: "Test".to_string(),
            template: "user_mailer/welcome.html.tera".to_string(),
            text_body: "hello".to_string(),
            html_body: "<p>hello</p>".to_string(),
            context: serde_json::json!({"user_name": "Ada"}),
            sent_at: Utc::now(),
        }
    }

    #[test]
    fn captures_and_returns_emails() {
        let capture = TestEmailCapture::new();
        assert!(capture.last().is_none());
        capture.capture(sample("a@example.com"));
        capture.capture(sample("b@example.com"));
        assert_eq!(capture.count(), 2);
        assert_eq!(capture.last().unwrap().to, "b@example.com");
    }

    #[test]
    fn clear_clears_buffer() {
        let capture = TestEmailCapture::new();
        capture.capture(sample("a@example.com"));
        capture.clear();
        assert_eq!(capture.count(), 0);
    }

    #[test]
    fn find_by_to_filters() {
        let capture = TestEmailCapture::new();
        capture.capture(sample("a@example.com"));
        capture.capture(sample("b@example.com"));
        capture.capture(sample("a@example.com"));
        assert_eq!(capture.find_by_to("a@example.com").len(), 2);
        assert_eq!(capture.find_by_to("missing").len(), 0);
    }

    #[test]
    fn find_by_template_filters() {
        let capture = TestEmailCapture::new();
        let mut other = sample("a@example.com");
        other.template = "user_mailer/password_reset.html.tera".to_string();
        capture.capture(sample("a@example.com"));
        capture.capture(other);
        assert_eq!(
            capture
                .find_by_template("user_mailer/welcome.html.tera")
                .len(),
            1
        );
    }

    #[test]
    fn clone_shares_storage() {
        let a = TestEmailCapture::new();
        let b = a.clone();
        a.capture(sample("a@example.com"));
        assert_eq!(b.count(), 1);
    }
}
