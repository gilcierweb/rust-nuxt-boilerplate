//! Registration-time email sanity checks.
//!
//! Mass-registration campaigns need one fresh mailbox per account. Two cheap
//! signals expose most of them:
//!
//! 1. **Disposable providers** — curated denylist of temporary-mail services,
//!    extendable per deployment through `DISPOSABLE_EMAIL_DOMAINS`.
//! 2. **No mail exchanger** — a domain without MX records cannot receive the
//!    confirmation email, so the account could never be activated. Optional
//!    (`EMAIL_MX_CHECK_ENABLED`) because it needs a DNS lookup, which is why it
//!    runs over DNS-over-HTTPS with the HTTP client the project already owns.
//!
//! Both checks are advisory inputs to the registration policy: they run before
//! the Argon2id hash so a bot pays nothing for a rejected address.

use std::sync::OnceLock;

/// Well-known temporary/disposable mailbox providers.
///
/// Curated on purpose: a broad, auto-scraped list produces false positives
/// against legitimate privacy-focused mailboxes, while this set covers the
/// providers bots actually reach for. Deployments can extend it with
/// `DISPOSABLE_EMAIL_DOMAINS`.
const DISPOSABLE_DOMAINS: &[&str] = &[
    "10minutemail.com",
    "10minutemail.net",
    "20minutemail.com",
    "33mail.com",
    "dispostable.com",
    "discard.email",
    "emailfake.com",
    "emailondeck.com",
    "emailtemporanea.com",
    "fakeinbox.com",
    "getairmail.com",
    "getnada.com",
    "grr.la",
    "guerrillamail.biz",
    "guerrillamail.com",
    "guerrillamail.de",
    "guerrillamail.info",
    "guerrillamail.net",
    "guerrillamail.org",
    "guerrillamailblock.com",
    "harakirimail.com",
    "inboxbear.com",
    "mail-temp.com",
    "mailcatch.com",
    "maildrop.cc",
    "mailinator.com",
    "mailinator.net",
    "mailnesia.com",
    "mailnull.com",
    "mailsac.com",
    "mailtemp.info",
    "mintemail.com",
    "moakt.com",
    "mohmal.com",
    "mytemp.email",
    "mytrashmail.com",
    "sharklasers.com",
    "spam4.me",
    "spamgourmet.com",
    "temp-mail.io",
    "temp-mail.org",
    "tempail.com",
    "tempemail.net",
    "tempinbox.com",
    "tempmail.com",
    "tempmail.de",
    "tempmail.net",
    "tempmailo.com",
    "tempr.email",
    "throwawaymail.com",
    "trashmail.com",
    "trashmail.de",
    "trashmail.me",
    "trashmail.net",
    "tmails.net",
    "tmpmail.net",
    "tmpmail.org",
    "yopmail.com",
    "yopmail.fr",
];

/// DNS record type for MX (`15`).
const DNS_TYPE_MX: u32 = 15;

static SHARED_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn shared_client() -> &'static reqwest::Client {
    SHARED_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("reqwest client with default TLS must always build")
    })
}

/// The domain part of an email address, lowercased and trimmed.
///
/// Returns `None` for input without an `@`. The **last** `@` wins, matching
/// the address grammar: `a@b@c.test` belongs to `c.test`.
pub fn email_domain(email: &str) -> Option<String> {
    let (_, domain) = email.rsplit_once('@')?;
    let domain = domain.trim().to_ascii_lowercase();
    if domain.is_empty() {
        return None;
    }
    Some(domain)
}

/// Whether the domain belongs to a known disposable-mail provider.
///
/// Matching is case-insensitive and covers subdomains (`team.mailinator.com`),
/// but never bleeds into unrelated names: `notmailinator.com` is not
/// `mailinator.com`.
pub fn is_disposable_domain(domain: &str, extra_domains: &[String]) -> bool {
    let domain = domain.trim().to_ascii_lowercase();

    let matches = |candidate: &str| -> bool {
        let candidate = candidate.trim().to_ascii_lowercase();
        !candidate.is_empty()
            && (domain == candidate
                || domain
                    .strip_suffix(candidate.as_str())
                    .is_some_and(|prefix| prefix.ends_with('.')))
    };

    DISPOSABLE_DOMAINS.iter().any(|known| matches(known))
        || extra_domains.iter().any(|known| matches(known))
}

/// Whether the domain publishes MX records, i.e. can receive mail.
///
/// The resolver is supplied by the deployment (`EMAIL_DNS_RESOLVER_URL`), so no
/// DNS vendor is imposed on the project — it may be the operator's own
/// resolver or any DNS-over-HTTPS endpoint speaking the JSON API.
///
/// `None` means "unknown" (the lookup failed) — callers must decide whether
/// to fail open or closed; `Some(false)` means the domain definitively has no
/// mail exchanger.
pub async fn has_mail_exchanger_with_url(domain: &str, doh_endpoint: &str) -> Option<bool> {
    // The query string is built by hand: `RequestBuilder::query` needs a reqwest
    // feature this project does not enable.
    let url = format!(
        "{}?name={}&type={DNS_TYPE_MX}",
        doh_endpoint,
        urlencoding::encode(domain)
    );

    let response = shared_client()
        .get(url)
        .header(reqwest::header::ACCEPT, "application/dns-json")
        .send()
        .await
        .ok()?;

    let payload: DohResponse = response.json().await.ok()?;

    // Status 0 = NOERROR. Any other status (NXDOMAIN, SERVFAIL, ...) means
    // the lookup did not produce a usable answer.
    if payload.status != 0 {
        return None;
    }

    Some(
        payload
            .answer
            .unwrap_or_default()
            .iter()
            .any(|answer| answer.record_type == DNS_TYPE_MX && !answer.data.is_empty()),
    )
}

#[derive(serde::Deserialize)]
struct DohResponse {
    // DNS-over-HTTPS JSON APIs (Cloudflare, Google, Quad9) answer with the
    // classic DNS field names: "Status" and "Answer".
    #[serde(rename = "Status")]
    status: i32,
    #[serde(rename = "Answer", default)]
    answer: Option<Vec<DohAnswer>>,
}

#[derive(serde::Deserialize)]
struct DohAnswer {
    #[serde(rename = "type")]
    record_type: u32,
    #[serde(default)]
    data: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    /// Serve one canned JSON body over HTTP and return its URL.
    fn spawn_doh_responder(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0u8; 4096];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/dns-json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body,
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        format!("http://{addr}/dns-query")
    }

    #[test]
    fn domain_extraction_lowercases_and_trims() {
        assert_eq!(
            email_domain("  User@Example.COM "),
            Some("example.com".into())
        );
    }

    #[test]
    fn domain_extraction_uses_the_last_at_sign() {
        assert_eq!(email_domain("a@b@c.test"), Some("c.test".into()));
    }

    #[test]
    fn domain_extraction_rejects_input_without_an_at_sign() {
        assert_eq!(email_domain("not-an-email"), None);
        assert_eq!(email_domain("user@"), None);
    }

    #[test]
    fn well_known_disposable_providers_are_blocked() {
        for domain in [
            "mailinator.com",
            "guerrillamail.com",
            "yopmail.com",
            "trashmail.com",
        ] {
            assert!(
                is_disposable_domain(domain, &[]),
                "{domain} must be treated as disposable"
            );
        }
    }

    #[test]
    fn disposable_matching_ignores_case() {
        assert!(is_disposable_domain("MAILINATOR.COM", &[]));
    }

    #[test]
    fn disposable_matching_covers_subdomains() {
        assert!(is_disposable_domain("team.mailinator.com", &[]));
    }

    #[test]
    fn disposable_matching_respects_label_boundaries() {
        assert!(
            !is_disposable_domain("notmailinator.com", &[]),
            "a different domain merely ending with a known name must pass"
        );
    }

    #[test]
    fn regular_providers_are_not_disposable() {
        for domain in ["gmail.com", "outlook.com", "example.com", "company.io"] {
            assert!(
                !is_disposable_domain(domain, &[]),
                "{domain} must not be treated as disposable"
            );
        }
    }

    #[test]
    fn deployment_supplied_domains_extend_the_denylist() {
        let extra = vec!["spam-farm.test".to_string()];
        assert!(is_disposable_domain("spam-farm.test", &extra));
        assert!(is_disposable_domain("mail.spaM-farm.test", &extra));
        assert!(!is_disposable_domain("gmail.com", &extra));
    }

    #[tokio::test]
    async fn domain_with_mx_records_can_receive_mail() {
        let url = spawn_doh_responder(
            r#"{"Status":0,"Answer":[{"name":"example.com","type":15,"TTL":3600,"data":"10 mail.example.com"}]}"#,
        );
        let result = has_mail_exchanger_with_url("example.com", &url).await;
        assert_eq!(result, Some(true));
    }

    #[tokio::test]
    async fn domain_without_mx_records_cannot_receive_mail() {
        let url = spawn_doh_responder(r#"{"Status":0}"#);
        let result = has_mail_exchanger_with_url("example.com", &url).await;
        assert_eq!(result, Some(false));
    }

    #[tokio::test]
    async fn non_address_records_do_not_count_as_mail_exchangers() {
        let url = spawn_doh_responder(
            r#"{"Status":0,"Answer":[{"name":"example.com","type":1,"TTL":3600,"data":"203.0.113.10"}]}"#,
        );
        let result = has_mail_exchanger_with_url("example.com", &url).await;
        assert_eq!(result, Some(false));
    }

    #[tokio::test]
    async fn failed_lookup_is_unknown_not_a_verdict() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let result =
            has_mail_exchanger_with_url("example.com", &format!("http://{addr}/dns-query")).await;
        assert_eq!(result, None, "an unreachable resolver yields 'unknown'");
    }
}
