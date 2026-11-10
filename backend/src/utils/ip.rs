use std::net::IpAddr;

use actix_web::HttpRequest;
use ipnet::IpNet;

/// Extract the real client IP from the request, considering trusted proxies.
///
/// Checks headers in order of preference:
/// 1. `Forwarded` header (RFC 7239)
/// 2. `X-Forwarded-For` header
/// 3. `X-Real-IP` header (single-value proxies)
/// 4. Direct peer address (fallback)
///
/// Only uses forwarding headers if the immediate peer (`peer_addr`) is in the
/// trusted proxies list — this prevents header spoofing from untrusted clients.
///
/// The proxy chains are walked **right to left** (from the proxy that
/// transmitted the request to the origin client), skipping every hop that is
/// itself a trusted proxy. The first non-trusted entry is the real client.
///
/// This right-to-left direction is load-bearing: appending proxies (e.g. nginx
/// `$proxy_add_x_forwarded_for`) place the transmitting client at the right end
/// and blindly relay whatever the previous hop sent, so an attacker can freely
/// inject leftmost entries. Trusting the **leftmost** value lets a spoofed
/// `X-Forwarded-For: <random-ip>` mint a fresh rate-limit bucket per request;
/// trusting the **rightmost non-trusted** entry takes that power away.
pub fn extract_client_ip(req: &HttpRequest, trusted_proxies: &[IpNet]) -> Option<IpAddr> {
    let peer_ip = req.peer_addr()?.ip();

    // Only trust proxy headers if the immediate peer is a trusted proxy
    if !trusted_proxies.iter().any(|net| net.contains(&peer_ip)) {
        return Some(peer_ip);
    }

    // 1. Walk the proxy chain from the transmitting hop backwards to the
    //    origin client (see the function documentation for why right-to-left).
    if let Some(chain) = forwarding_chain(req) {
        for ip in chain.iter().rev() {
            if !trusted_proxies.iter().any(|net| net.contains(ip)) {
                return Some(*ip);
            }
        }
        // Every hop is itself a trusted proxy: the leftmost entry is the origin.
        return chain.into_iter().next();
    }

    // 2. Single-value fallback for proxies that only set `X-Real-IP` (typically
    //    the edge proxy overwriting it with `$remote_addr`). It carries no hop
    //    chain to validate, so it has the lowest precedence.
    if let Some(xri) = req.headers().get("x-real-ip")
        && let Ok(xri_str) = xri.to_str()
        && let Ok(ip) = xri_str.trim().parse::<IpAddr>()
    {
        return Some(ip);
    }

    // Fallback to peer address
    Some(peer_ip)
}

/// All client addresses contributed by the proxy forwarding headers, in the
/// order the proxies appended them (leftmost = origin, rightmost = transmitting
/// hop). Returns `None` when no header carries a parseable entry.
fn forwarding_chain(req: &HttpRequest) -> Option<Vec<IpAddr>> {
    if let Some(forwarded) = req.headers().get("forwarded")
        && let Ok(forwarded_str) = forwarded.to_str()
    {
        let chain = parse_forwarded_chain(forwarded_str);
        if !chain.is_empty() {
            return Some(chain);
        }
    }

    if let Some(xff) = req.headers().get("x-forwarded-for")
        && let Ok(xff_str) = xff.to_str()
    {
        let chain: Vec<IpAddr> = xff_str
            .split(',')
            .filter_map(|part| parse_chain_entry(part.trim()))
            .collect();
        if !chain.is_empty() {
            return Some(chain);
        }
    }

    None
}

/// Parse an `RFC 7239` `Forwarded` list into its `for=` addresses.
/// Example: `for=192.0.2.60;proto=https, for="[198.51.100.7]:8443"`
fn parse_forwarded_chain(forwarded: &str) -> Vec<IpAddr> {
    forwarded
        .split(',')
        .filter_map(|hop| {
            hop.split(';').find_map(|parameter| {
                let parameter = parameter.trim();
                let value = parameter.strip_prefix("for=")?;
                parse_chain_entry(value.trim().trim_matches('"'))
            })
        })
        .collect()
}

/// Parse one chain entry, tolerating bracketed IPv6 and port suffixes:
/// `192.0.2.60`, `192.0.2.60:8080`, `[2001:db8::1]`, `[2001:db8::1]:443`,
/// `2001:db8::1`.
fn parse_chain_entry(entry: &str) -> Option<IpAddr> {
    if let Some(rest) = entry.strip_prefix('[') {
        // Bracketed IPv6, optionally followed by ":port" after the closing
        // bracket — the port belongs to the socket, not to the address.
        let address = rest.split(']').next()?;
        return address.parse::<IpAddr>().ok();
    }

    // IPv4 optionally carrying a port, or a bare multi-colon IPv6 literal.
    if entry.matches(':').count() == 1 {
        let address = entry.split(':').next()?;
        return address.parse::<IpAddr>().ok();
    }

    entry.parse::<IpAddr>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    /// Trusted proxies used across these tests: the 10.0.0.0/8 private range.
    fn trusted_proxies() -> Vec<IpNet> {
        vec!["10.0.0.0/8".parse().unwrap()]
    }

    const EMPTY_TRUSTED: Vec<IpNet> = Vec::new();

    fn request(
        peer: &str,
        xff: Option<&str>,
        forwarded: Option<&str>,
        x_real_ip: Option<&str>,
    ) -> actix_web::HttpRequest {
        let mut request = TestRequest::default().peer_addr(peer.parse().unwrap());
        if let Some(header) = xff {
            request = request.insert_header(("x-forwarded-for", header));
        }
        if let Some(header) = forwarded {
            request = request.insert_header(("forwarded", header));
        }
        if let Some(header) = x_real_ip {
            request = request.insert_header(("x-real-ip", header));
        }
        request.to_http_request()
    }

    #[test]
    fn untrusted_peer_ignores_every_forwarding_header() {
        // A direct client may send all the spoofed headers it wants: without a
        // trusted proxy in front, the socket address is the only truth.
        let req = request(
            "192.0.2.77:4432",
            Some("1.2.3.4, 5.6.7.8"),
            Some("for=9.9.9.9"),
            Some("8.8.8.8"),
        );
        assert_eq!(
            extract_client_ip(&req, &EMPTY_TRUSTED),
            Some("192.0.2.77".parse().unwrap()),
        );
    }

    #[test]
    fn spoofed_leftmost_xff_entry_is_discarded_behind_trusted_proxy() {
        // REGRESSION: the old implementation took the LEFTMOST XFF entry, so a
        // bot sending `X-Forwarded-For: <random-ip>` minted a fresh rate-limit
        // bucket on every request. Appending proxies put the real client at the
        // right end, which is the only entry the walk may trust.
        let req = request(
            "10.1.2.3:5000",
            Some("203.0.113.66, 198.51.100.44"),
            None,
            None,
        );
        assert_eq!(
            extract_client_ip(&req, &trusted_proxies()),
            Some("198.51.100.44".parse().unwrap()),
            "the rightmost non-trusted XFF entry must win over attacker-chosen values",
        );
    }

    #[test]
    fn fully_appended_multi_proxy_chain_resolves_the_origin_client() {
        // client → trusted1 → trusted2 → app. trusted1 appended the client,
        // trusted2 appended trusted1, the app's peer is trusted2.
        let req = request("10.1.2.3:5000", Some("198.51.100.44, 10.0.0.9"), None, None);
        assert_eq!(
            extract_client_ip(&req, &trusted_proxies()),
            Some("198.51.100.44".parse().unwrap()),
        );
    }

    #[test]
    fn fully_trusted_chain_falls_back_to_the_leftmost_entry() {
        // An internal mesh where every hop is trusted: the leftmost entry is
        // the origin, even though it is itself a trusted address.
        let req = request("10.1.2.3:5000", Some("10.0.0.1, 10.0.0.2"), None, None);
        assert_eq!(
            extract_client_ip(&req, &trusted_proxies()),
            Some("10.0.0.1".parse().unwrap()),
        );
    }

    #[test]
    fn spoofed_forwarded_entry_is_discarded_behind_trusted_proxy() {
        // RFC 7239 lists hops left to right as well; the same walk applies.
        let req = request(
            "10.1.2.3:5000",
            None,
            Some(r#"for=203.0.113.66, for="198.51.100.44:8443""#),
            None,
        );
        assert_eq!(
            extract_client_ip(&req, &trusted_proxies()),
            Some("198.51.100.44".parse().unwrap()),
            "the quoted IPv4-with-port Forwarded entry must parse without the port",
        );
    }

    #[test]
    fn ipv6_chain_entries_are_parsed_without_brackets_or_port() {
        // Bracketed IPv6 (with port) leftmost, a trusted hop appended on the
        // right: the walk skips the trusted hop and returns the bracketed
        // origin — which only works if `[...]:443` parsed as an address.
        let req = request(
            "10.1.2.3:5000",
            Some("[2001:db8::1]:443, 10.0.0.9"),
            None,
            None,
        );
        assert_eq!(
            extract_client_ip(&req, &trusted_proxies()),
            Some("2001:db8::1".parse().unwrap()),
        );
    }

    #[test]
    fn x_real_ip_is_only_a_fallback_without_a_chain_header() {
        // Proxies that only set X-Real-IP still contribute the client address;
        // when a chain header exists the chain decision wins and X-Real-IP
        // cannot override it.
        let req = request("10.1.2.3:5000", None, None, Some("198.51.100.77"));
        assert_eq!(
            extract_client_ip(&req, &trusted_proxies()),
            Some("198.51.100.77".parse().unwrap()),
        );

        let req = request(
            "10.1.2.3:5000",
            Some("198.51.100.44"),
            None,
            Some("203.0.113.1"),
        );
        assert_eq!(
            extract_client_ip(&req, &trusted_proxies()),
            Some("198.51.100.44".parse().unwrap()),
            "X-Real-IP must not override the XFF walk",
        );
    }

    #[test]
    fn unparseable_headers_behind_a_trusted_proxy_fall_back_to_the_peer() {
        let req = request(
            "10.1.2.3:5000",
            Some("not-an-ip, also-not-an-ip"),
            None,
            None,
        );
        assert_eq!(
            extract_client_ip(&req, &trusted_proxies()),
            Some("10.1.2.3".parse().unwrap()),
        );
    }

    #[test]
    fn requests_without_a_socket_address_have_no_client_ip() {
        let req = TestRequest::default().to_http_request();
        assert_eq!(extract_client_ip(&req, &trusted_proxies()), None);
    }
}
