//! Shared network security guards for curl-invoking modules.
//!
//! Provides URL validation, header sanitisation, and RFC 6890 private-range
//! blocking used by `data_exchange`, `webhooks`, `citation_discovery`, and
//! `ollama_models`.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Validate that `value` is a well-formed http:// or https:// URL with no
/// control characters. Returns the trimmed URL on success.
pub(crate) fn validate_http_url(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if !is_http_url(trimmed) {
        return Err(format!("{label} must be an http:// or https:// URL."));
    }
    if trimmed.chars().any(|c| c == '\0' || c == '\n' || c == '\r') {
        return Err(format!("{label} cannot contain control characters."));
    }
    Ok(trimmed.to_string())
}

/// Returns `true` if the string starts with `http://` or `https://` (case-insensitive).
pub(crate) fn is_http_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

/// Returns `true` when the URL's host is NOT in an RFC 6890 / IANA special-purpose
/// private, loopback, or link-local range.
///
/// DNS-resolution TOCTOU is inherent in any pre-connect check; this function
/// screens literal IP addresses and well-known loopback hostnames only.
pub(crate) fn is_public_destination(url_str: &str) -> bool {
    let after_scheme = match url_str.find("://") {
        Some(pos) => &url_str[pos + 3..],
        None => return false,
    };
    // Strip userinfo
    let authority = match after_scheme.find('@') {
        Some(pos) => &after_scheme[pos + 1..],
        None => after_scheme,
    };
    // IPv6 literals: [::1] or [::1]:8080
    let host = if authority.starts_with('[') {
        match authority.find(']') {
            Some(pos) => &authority[1..pos],
            None => return false,
        }
    } else {
        let end = authority
            .find(|c| c == ':' || c == '/' || c == '?' || c == '#')
            .unwrap_or(authority.len());
        &authority[..end]
    };

    let lower = host.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "localhost" | "ip6-localhost" | "ip6-loopback"
    ) {
        return false;
    }
    if matches!(lower.as_str(), "0.0.0.0" | "::") {
        return false;
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        return !is_private_ip(ip);
    }
    // Non-IP hostname — allow (DNS resolution TOCTOU is inherent).
    true
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_private_v4(v4),
        IpAddr::V6(v6) => is_private_v6(v6),
    }
}

fn is_private_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    o[0] == 127                                        // 127.0.0.0/8  loopback
        || o[0] == 10                                  // 10.0.0.0/8   private
        || (o[0] == 172 && (16..=31).contains(&o[1])) // 172.16.0.0/12
        || (o[0] == 192 && o[1] == 168)                // 192.168.0.0/16
        || (o[0] == 169 && o[1] == 254)                // 169.254.0.0/16 link-local
        || (o[0] == 100 && (64..=127).contains(&o[1])) // 100.64.0.0/10 CGNAT
        || (o[0] == 192 && o[1] == 0 && o[2] == 2)    // 192.0.2.0/24 doc
        || (o[0] == 198 && o[1] == 51 && o[2] == 100) // 198.51.100.0/24 doc
        || (o[0] == 203 && o[1] == 0 && o[2] == 113)  // 203.0.113.0/24 doc
        || o[0] == 0                                   // 0.0.0.0/8 "this" network
        || o[0] >= 240 // 240.0.0.0/4 reserved
}

fn is_private_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    ip == Ipv6Addr::LOCALHOST                    // ::1
        || ip == Ipv6Addr::UNSPECIFIED           // ::
        || (s[0] & 0xfe00) == 0xfc00             // fc00::/7 ULA
        || (s[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
}

/// Returns `true` when the header name is a valid HTTP token: ASCII
/// printable, no colon, no whitespace, no control characters.
pub(crate) fn is_safe_header_name(name: &str) -> bool {
    !name.is_empty()
        && name.is_ascii()
        && name
            .chars()
            .all(|c| c > ' ' && c != ':' && c != '\x7f' && !c.is_control())
}

/// Returns `true` when the header value contains no CRLF, LF, or NUL.
pub(crate) fn is_safe_header_value(value: &str) -> bool {
    value.chars().all(|c| c != '\r' && c != '\n' && c != '\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_http_url_accepts_http_and_https() {
        assert!(validate_http_url("http://example.com", "URL").is_ok());
        assert!(validate_http_url("https://example.com/path?q=1", "URL").is_ok());
        assert_eq!(
            validate_http_url("  https://example.com  ", "URL").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn validate_http_url_rejects_file_and_other_schemes() {
        assert!(validate_http_url("file:///etc/passwd", "URL").is_err());
        assert!(validate_http_url("ftp://example.com", "URL").is_err());
        assert!(validate_http_url("gopher://example.com", "URL").is_err());
        // Argument-injection: dash-prefixed looks like a curl flag
        assert!(validate_http_url("-Kfile", "URL").is_err());
        assert!(validate_http_url("", "URL").is_err());
    }

    #[test]
    fn validate_http_url_rejects_control_characters() {
        assert!(validate_http_url("https://example.com/\r\nX-Evil: yes", "URL").is_err());
        assert!(validate_http_url("https://example.com/\0", "URL").is_err());
        assert!(validate_http_url("https://example.com/\npath", "URL").is_err());
    }

    #[test]
    fn is_public_destination_blocks_private_ipv4() {
        assert!(!is_public_destination("http://10.0.0.1/"));
        assert!(!is_public_destination("http://172.16.0.1/"));
        assert!(!is_public_destination("http://172.31.255.255/"));
        assert!(!is_public_destination("http://192.168.1.1/"));
        assert!(!is_public_destination(
            "http://169.254.169.254/latest/meta-data/"
        ));
        assert!(!is_public_destination("http://127.0.0.1:8080/admin"));
        assert!(!is_public_destination("http://100.64.0.1/"));
    }

    #[test]
    fn is_public_destination_blocks_ipv6_private() {
        assert!(!is_public_destination("http://[::1]/"));
        assert!(!is_public_destination("http://[fc00::1]/"));
        assert!(!is_public_destination("http://[fe80::1]/"));
    }

    #[test]
    fn is_public_destination_blocks_localhost_names() {
        assert!(!is_public_destination("http://localhost/admin"));
        assert!(!is_public_destination("http://localhost:8080/"));
        assert!(!is_public_destination("http://ip6-localhost/"));
    }

    #[test]
    fn is_public_destination_allows_public_addresses() {
        assert!(is_public_destination("https://api.example.com/v1"));
        assert!(is_public_destination("https://8.8.8.8/dns"));
        assert!(is_public_destination("https://1.1.1.1/"));
        assert!(is_public_destination("https://api.tavily.com/search"));
    }

    #[test]
    fn is_safe_header_name_rejects_bad_chars() {
        assert!(!is_safe_header_name(""));
        assert!(!is_safe_header_name("X-Evil\r\n"));
        assert!(!is_safe_header_name("X:Colon"));
        assert!(!is_safe_header_name("X Header"));
        assert!(!is_safe_header_name("X\0Nul"));
        assert!(is_safe_header_name("X-Api-Key"));
        assert!(is_safe_header_name("Authorization"));
        assert!(is_safe_header_name("Content-Type"));
    }

    #[test]
    fn is_safe_header_value_rejects_crlf() {
        assert!(!is_safe_header_value("value\r\nX-Evil: injected"));
        assert!(!is_safe_header_value("value\nX-Evil: injected"));
        assert!(!is_safe_header_value("val\0ue"));
        assert!(is_safe_header_value("Bearer abc123"));
        assert!(is_safe_header_value("application/json"));
    }
}
