use actix_web::HttpRequest;
use ipnet::IpNet;
use std::net::IpAddr;

/// Extract the real client IP from the request, considering trusted proxies.
///
/// Checks headers in order of preference:
/// 1. `Forwarded` header (RFC 7239)
/// 2. `X-Forwarded-For` header
/// 3. `X-Real-IP` header
/// 4. Direct peer address (fallback)
///
/// Only uses proxy headers if the immediate peer (peer_addr) is in the trusted_proxies list.
/// This prevents header spoofing from untrusted clients.
pub fn extract_client_ip(req: &HttpRequest, trusted_proxies: &[IpNet]) -> Option<IpAddr> {
    let peer_ip = req.peer_addr()?.ip();

    // Only trust proxy headers if the immediate peer is a trusted proxy
    let is_trusted = trusted_proxies.iter().any(|net| net.contains(&peer_ip));

    if !is_trusted {
        return Some(peer_ip);
    }

    // 1. Check Forwarded header (RFC 7239)
    if let Some(forwarded) = req.headers().get("forwarded") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip) = parse_forwarded_for(forwarded_str) {
                return Some(ip);
            }
        }
    }

    // 2. Check X-Forwarded-For header
    if let Some(xff) = req.headers().get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // X-Forwarded-For can contain multiple IPs: "client, proxy1, proxy2"
            // The leftmost is the original client
            if let Some(ip_str) = xff_str.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    // 3. Check X-Real-IP header
    if let Some(xri) = req.headers().get("x-real-ip") {
        if let Ok(xri_str) = xri.to_str() {
            if let Ok(ip) = xri_str.trim().parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // Fallback to peer address
    Some(peer_ip)
}

/// Parse the `for` parameter from a Forwarded header (RFC 7239).
/// Example: `for="192.0.2.60";proto=https;by=203.0.113.43`
fn parse_forwarded_for(forwarded: &str) -> Option<IpAddr> {
    for part in forwarded.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("for=") {
            // Remove quotes if present
            let value = value.trim().trim_matches('"');
            // Handle optional port: "192.0.2.60" or "[2001:db8::1]" or "192.0.2.60:8080"
            if let Some(ip) = value.strip_prefix('[') {
                // IPv6: "[2001:db8::1]"
                if let Some(ip) = ip.strip_suffix(']') {
                    return ip.parse::<IpAddr>().ok();
                }
            } else if value.contains(':') && !value.starts_with('[') {
                // Could be IPv4 with port: "192.0.2.60:8080"
                if let Some(ip) = value.split(':').next() {
                    return ip.parse::<IpAddr>().ok();
                }
            } else {
                // IPv4 or IPv6 without brackets
                return value.parse::<IpAddr>().ok();
            }
        }
    }
    None
}
