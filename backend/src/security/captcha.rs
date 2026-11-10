//! Optional CAPTCHA verification for brute-force-sensitive endpoints.
//!
//! The boilerplate ships with CAPTCHA **disabled** (`CAPTCHA_PROVIDER=none`).
//! Frontends exposed on hostile networks (staging, production, public deploys)
//! can opt into human verification without any code change:
//!
//! ```text
//! CAPTCHA_PROVIDER=turnstile   # or recaptcha
//! CAPTCHA_SECRET=0x........
//! ```
//!
//! The supported providers are verified server-side against their official
//! site-verify HTTP endpoints. When enabled, the register handler refuses the
//! request with `400` for missing/absent tokens and `422` for invalid ones,
//! and returns `503` when the verification service is unreachable — a dead
//! upstream must fail closed, never become a bypass.
//!
//! Both providers share the same response shape (`{"success": bool, ...}`), so
//! a single parser covers Turnstile and reCAPTCHA.

use std::str::FromStr;
use std::sync::OnceLock;

use serde::Deserialize;

/// Shared site-verify HTTP client (connection pooling + request timeout).
static SHARED_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

const SITEVERIFY_TIMEOUT_SECS: u64 = 5;

const TURNSTILE_SITEVERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";
const RECAPTCHA_SITEVERIFY_URL: &str = "https://www.google.com/recaptcha/api/siteverify";

/// CAPTCHA provider selected via `CAPTCHA_PROVIDER`.
///
/// The boilerplate never selects a vendor: `none` is the default and the
/// operator chooses between the two widely used site-verify protocols
/// (`turnstile`, `recaptcha`) or points `CAPTCHA_VERIFY_URL` at any other
/// service speaking the same protocol (`custom`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaProvider {
    /// No human verification (boilerplate default).
    #[default]
    None,
    /// Cloudflare Turnstile.
    Turnstile,
    /// Google reCAPTCHA (v2 checkbox / invisible).
    Recaptcha,
    /// Any other site-verify-compatible service, configured through
    /// `CAPTCHA_VERIFY_URL`.
    Custom,
}

impl FromStr for CaptchaProvider {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "none" | "off" => Ok(Self::None),
            "turnstile" => Ok(Self::Turnstile),
            "recaptcha" => Ok(Self::Recaptcha),
            "custom" => Ok(Self::Custom),
            other => Err(other.to_string()),
        }
    }
}

impl CaptchaProvider {
    /// Official server-side verification endpoint, `None` when disabled.
    pub fn siteverify_url(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Turnstile => Some(TURNSTILE_SITEVERIFY_URL),
            Self::Recaptcha => Some(RECAPTCHA_SITEVERIFY_URL),
            // The endpoint is deployment-specific; it is passed in explicitly.
            Self::Custom => None,
        }
    }
}

/// Outcome of a successful site-verify round trip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaOutcome {
    /// The provider confirmed the token was solved for this site.
    Valid,
    /// The provider rejected the token (missing, replayed, wrong site, expired).
    Invalid,
}

/// The verification service could not be consulted (transport/format failure).
/// Callers must fail closed.
#[derive(Debug)]
pub struct CaptchaUnavailable;

#[derive(Deserialize)]
struct SiteVerifyResponse {
    success: Option<bool>,
}

/// HTTP client shared by every site-verify call.
fn shared_client() -> &'static reqwest::Client {
    SHARED_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(SITEVERIFY_TIMEOUT_SECS))
            .build()
            .expect("reqwest client with default TLS must always build")
    })
}

/// Verify a CAPTCHA token against one specific site-verify URL.
///
/// Split from [`verify_captcha_token`] so tests can point at a local
/// responder instead of the real provider.
pub async fn verify_with_url(
    client: &reqwest::Client,
    siteverify_url: &str,
    secret: &str,
    token: &str,
) -> Result<CaptchaOutcome, CaptchaUnavailable> {
    // An absent token is an invalid submission, not an outage: no round trip.
    if token.trim().is_empty() {
        return Ok(CaptchaOutcome::Invalid);
    }

    // siteverify only accepts form-encoded bodies; `urlencoding` avoids
    // pulling reqwest's extra form feature.
    let payload = format!(
        "secret={}&response={}",
        urlencoding::encode(secret),
        urlencoding::encode(token)
    );

    let response = client
        .post(siteverify_url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(payload)
        .send()
        .await
        .map_err(|_| CaptchaUnavailable)?;

    let body: SiteVerifyResponse = response.json().await.map_err(|_| CaptchaUnavailable)?;

    if body.success == Some(true) {
        Ok(CaptchaOutcome::Valid)
    } else {
        Ok(CaptchaOutcome::Invalid)
    }
}

/// Resolve the site-verify endpoint for a provider.
///
/// Presets carry their own endpoint; the `custom` provider (and any explicit
/// override) uses the deployment's `CAPTCHA_VERIFY_URL`.
pub fn resolve_siteverify_url(
    provider: CaptchaProvider,
    custom_url: Option<&str>,
) -> Option<String> {
    match provider {
        CaptchaProvider::Custom => custom_url
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .map(str::to_string),
        other => other.siteverify_url().map(str::to_string),
    }
}

/// Verify a token against the provider's official endpoint.
pub async fn verify_captcha_token(
    provider: CaptchaProvider,
    siteverify_url: Option<&str>,
    secret: &str,
    token: &str,
) -> Result<CaptchaOutcome, CaptchaUnavailable> {
    let url = resolve_siteverify_url(provider, siteverify_url).ok_or(CaptchaUnavailable)?;
    verify_with_url(shared_client(), &url, secret, token).await
}

/// Enforce the CAPTCHA policy for one submitted token.
///
/// The caller gates on `captcha_enabled()`; this turns provider state plus the
/// submitted token into a decision the HTTP layer maps onto errors:
///
/// - [`CaptchaDecision::Required`] — no token submitted;
/// - [`CaptchaDecision::Rejected`] — token rejected (missing/expired/wrong
///   site/replayed) — also used for empty/whitespace tokens without a round
///   trip;
/// - [`CaptchaDecision::ServiceUnavailable`] — half-enabled provider or dead
///   upstream; must fail **closed** so an outage never becomes a bypass;
/// - [`CaptchaDecision::Verified`] — human verification passed, or the
///   provider is disabled (the boilerplate's default).
pub async fn decide_captcha(
    provider: CaptchaProvider,
    siteverify_url: Option<&str>,
    secret: Option<&str>,
    token: Option<&str>,
) -> CaptchaDecision {
    if provider == CaptchaProvider::None {
        return CaptchaDecision::Verified;
    }

    let secret = match secret {
        Some(secret) if !secret.trim().is_empty() => secret,
        _ => return CaptchaDecision::ServiceUnavailable,
    };

    if resolve_siteverify_url(provider, siteverify_url).is_none() {
        return CaptchaDecision::ServiceUnavailable;
    }

    match token {
        None => CaptchaDecision::Required,
        Some(token) => match verify_captcha_token(provider, siteverify_url, secret, token).await {
            Ok(CaptchaOutcome::Valid) => CaptchaDecision::Verified,
            Ok(CaptchaOutcome::Invalid) => CaptchaDecision::Rejected,
            Err(CaptchaUnavailable) => CaptchaDecision::ServiceUnavailable,
        },
    }
}

/// Outcome of one CAPTCHA enforcement, see [`decide_captcha`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaDecision {
    Verified,
    Required,
    Rejected,
    ServiceUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    /// Bind a throwaway HTTP responder that answers every request with
    /// `body`, and return its `http://...` URL.
    fn spawn_siteverify(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                // Drain the request line + headers; the body is irrelevant to
                // site-verify simulation.
                let mut buffer = [0u8; 4096];
                let _ = stream.read(&mut buffer);

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body,
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        format!("http://{addr}/siteverify")
    }

    #[tokio::test]
    async fn provider_confirming_the_token_is_valid() {
        let url = spawn_siteverify(r#"{"success": true,"challenge_ts": "2026-01-01"}"#);
        let outcome = verify_with_url(shared_client(), &url, "secret", "token-abc")
            .await
            .expect("reachable provider");
        assert_eq!(outcome, CaptchaOutcome::Valid);
    }

    #[tokio::test]
    async fn provider_rejecting_the_token_is_invalid() {
        let url = spawn_siteverify(r#"{"success": false,"error-codes": ["timeout-or-duplicate"]}"#);
        let outcome = verify_with_url(shared_client(), &url, "secret", "replayed-token")
            .await
            .expect("reachable provider");
        assert_eq!(outcome, CaptchaOutcome::Invalid);
    }

    #[tokio::test]
    async fn response_without_a_success_field_is_invalid_not_valid() {
        let url = spawn_siteverify(r#"{"error-codes": ["missing-input-response"]}"#);
        let outcome = verify_with_url(shared_client(), &url, "secret", "token")
            .await
            .expect("reachable provider");
        assert_eq!(outcome, CaptchaOutcome::Invalid);
    }

    #[tokio::test]
    async fn malformed_response_fails_closed() {
        let url = spawn_siteverify("<html>not json</html>");
        let outcome = verify_with_url(shared_client(), &url, "secret", "token").await;
        assert!(
            outcome.is_err(),
            "an unreadable provider answer must be treated as an outage"
        );
    }

    #[tokio::test]
    async fn unreachable_siteverify_is_an_outage() {
        // Reserve then immediately free the port: connecting must be refused.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let outcome = verify_with_url(
            shared_client(),
            &format!("http://{addr}/siteverify"),
            "s",
            "t",
        )
        .await;
        assert!(outcome.is_err(), "refused TCP connections are outages");
    }

    #[tokio::test]
    async fn empty_tokens_fail_fast_without_a_round_trip() {
        // No responder: any HTTP attempt would be an outage error; the empty
        // token must instead return Invalid without touching the network.
        let outcome = verify_with_url(shared_client(), "http://127.0.0.1:2/siteverify", "s", "   ")
            .await
            .expect("no round trip happened");
        assert_eq!(outcome, CaptchaOutcome::Invalid);
    }

    #[test]
    fn provider_strings_parse_case_insensitively() {
        assert_eq!("turnstile".parse(), Ok(CaptchaProvider::Turnstile));
        assert_eq!("TURNSTILE".parse(), Ok(CaptchaProvider::Turnstile));
        assert_eq!("Recaptcha".parse(), Ok(CaptchaProvider::Recaptcha));
        assert_eq!("none".parse(), Ok(CaptchaProvider::None));
        assert_eq!("".parse(), Ok(CaptchaProvider::None));
    }

    #[test]
    fn unknown_provider_names_are_rejected() {
        assert!("hcaptcha".parse::<CaptchaProvider>().is_err());
        assert!("bogus".parse::<CaptchaProvider>().is_err());
    }

    #[test]
    fn disabled_provider_has_no_siteverify_endpoint() {
        assert_eq!(CaptchaProvider::None.siteverify_url(), None);
        assert!(CaptchaProvider::Turnstile.siteverify_url().is_some());
        assert!(CaptchaProvider::Recaptcha.siteverify_url().is_some());
        assert_eq!(CaptchaProvider::Custom.siteverify_url(), None);
    }

    #[test]
    fn custom_provider_resolves_the_deployment_endpoint() {
        assert_eq!(
            resolve_siteverify_url(CaptchaProvider::Custom, Some("https://captcha.test/verify")),
            Some("https://captcha.test/verify".to_string()),
        );
        assert_eq!(
            resolve_siteverify_url(CaptchaProvider::Custom, None),
            None,
            "a custom provider without an endpoint cannot be verified"
        );
    }

    #[test]
    fn presets_ignore_the_custom_endpoint() {
        assert_eq!(
            resolve_siteverify_url(
                CaptchaProvider::Turnstile,
                Some("https://captcha.test/verify")
            ),
            CaptchaProvider::Turnstile
                .siteverify_url()
                .map(str::to_string),
        );
    }

    // -------------------------------------------------------------------------
    // Decision helper (network-free paths)
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn disabled_provider_verifies_trivially() {
        // Documented contract: callers gate on `captcha_enabled()`; treat a
        // disabled provider as verified so call-order regressions cannot lock
        // register behind invisible requirements.
        let decision = decide_captcha(CaptchaProvider::None, None, None, None).await;
        assert_eq!(decision, CaptchaDecision::Verified);
    }

    #[tokio::test]
    async fn missing_token_is_required_not_rejected() {
        let decision = decide_captcha(CaptchaProvider::Turnstile, None, Some("secret"), None).await;
        assert_eq!(decision, CaptchaDecision::Required);
    }

    #[tokio::test]
    async fn whitespace_token_is_rejected_without_a_round_trip() {
        // No responder exists; a network attempt would surface as
        // ServiceUnavailable while the expected outcome is an explicit reject.
        let decision = decide_captcha(
            CaptchaProvider::Turnstile,
            None,
            Some("secret"),
            Some("   "),
        )
        .await;
        assert_eq!(decision, CaptchaDecision::Rejected);
    }

    #[tokio::test]
    async fn half_enabled_provider_is_an_outage() {
        // Provider set but secret forgotten: fail closed (production refuses
        // this configuration at boot; outside production it stays closed).
        let decision = decide_captcha(CaptchaProvider::Turnstile, None, None, Some("token")).await;
        assert_eq!(decision, CaptchaDecision::ServiceUnavailable);
    }

    #[tokio::test]
    async fn custom_provider_without_endpoint_is_an_outage() {
        let decision =
            decide_captcha(CaptchaProvider::Custom, None, Some("secret"), Some("token")).await;
        assert_eq!(decision, CaptchaDecision::ServiceUnavailable);
    }
}
