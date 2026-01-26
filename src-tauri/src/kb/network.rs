//! Network security module for AssistSupport
//! Implements SSRF protection with IP blocking and allowlist support
//!
//! Security Note: This module now supports DNS pinning to prevent TOCTOU attacks.
//! Use `validate_url_for_ssrf_with_pinning()` for the most secure validation,
//! which returns both the validated URL and the pinned IP addresses.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use thiserror::Error;
use url::Url;

use super::dns::{DnsError, PinnedDnsResolver, ValidatedUrl};

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("SSRF blocked: {0}")]
    SsrfBlocked(String),
    #[error("DNS resolution failed: {0}")]
    DnsResolutionFailed(String),
    #[error("Network request failed: {0}")]
    RequestFailed(String),
    #[error("Request timeout")]
    Timeout,
    #[error("Content too large: {size} bytes (max: {max} bytes)")]
    ContentTooLarge { size: usize, max: usize },
    #[error("Offline mode: network access disabled")]
    OfflineMode,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// SSRF protection configuration
#[derive(Debug, Clone)]
pub struct SsrfConfig {
    /// Block private IP ranges (RFC 1918, etc.)
    pub block_private: bool,
    /// Block loopback addresses (127.0.0.0/8, ::1)
    pub block_loopback: bool,
    /// Block link-local addresses (169.254.0.0/16, fe80::/10)
    pub block_link_local: bool,
    /// Block multicast addresses
    pub block_multicast: bool,
    /// Allowed hosts that bypass SSRF checks (explicit user opt-in)
    pub allowlist: Vec<String>,
    /// Maximum content size in bytes
    pub max_content_size: usize,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for SsrfConfig {
    fn default() -> Self {
        Self {
            block_private: true,
            block_loopback: true,
            block_link_local: true,
            block_multicast: true,
            allowlist: Vec::new(),
            max_content_size: 10 * 1024 * 1024, // 10MB
            timeout_secs: 30,
        }
    }
}

/// Check if an IPv4 address is in a private range
fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    // RFC 1918 private ranges
    // 10.0.0.0/8
    if ip.octets()[0] == 10 {
        return true;
    }
    // 172.16.0.0/12
    if ip.octets()[0] == 172 && (ip.octets()[1] >= 16 && ip.octets()[1] <= 31) {
        return true;
    }
    // 192.168.0.0/16
    if ip.octets()[0] == 192 && ip.octets()[1] == 168 {
        return true;
    }
    // Carrier-grade NAT 100.64.0.0/10
    if ip.octets()[0] == 100 && (ip.octets()[1] >= 64 && ip.octets()[1] <= 127) {
        return true;
    }
    false
}

/// Check if an IPv6 address is in a private/internal range
fn is_private_ipv6(ip: &Ipv6Addr) -> bool {
    // Unique local addresses fc00::/7
    let segments = ip.segments();
    if (segments[0] & 0xfe00) == 0xfc00 {
        return true;
    }
    // Site-local (deprecated but still checked) fec0::/10
    if (segments[0] & 0xffc0) == 0xfec0 {
        return true;
    }
    false
}

/// Check if an IPv6 address is IPv4-mapped (::ffff:x.x.x.x)
fn get_ipv4_from_mapped(ipv6: &Ipv6Addr) -> Option<Ipv4Addr> {
    let segments = ipv6.segments();
    // IPv4-mapped format: ::ffff:x.x.x.x
    // segments[0..5] should be 0, segment[5] should be 0xffff
    if segments[0] == 0 && segments[1] == 0 && segments[2] == 0
        && segments[3] == 0 && segments[4] == 0 && segments[5] == 0xffff
    {
        let high = segments[6];
        let low = segments[7];
        return Some(Ipv4Addr::new(
            (high >> 8) as u8,
            (high & 0xff) as u8,
            (low >> 8) as u8,
            (low & 0xff) as u8,
        ));
    }

    // IPv4-compatible format (deprecated but still checked): ::x.x.x.x
    if segments[0] == 0 && segments[1] == 0 && segments[2] == 0
        && segments[3] == 0 && segments[4] == 0 && segments[5] == 0
        && (segments[6] != 0 || segments[7] != 0)
    {
        let high = segments[6];
        let low = segments[7];
        return Some(Ipv4Addr::new(
            (high >> 8) as u8,
            (high & 0xff) as u8,
            (low >> 8) as u8,
            (low & 0xff) as u8,
        ));
    }

    None
}

/// Check if an IP address should be blocked by SSRF protection
/// This includes checking IPv6-mapped IPv4 addresses (::ffff:x.x.x.x)
pub fn is_ip_blocked(ip: &IpAddr, config: &SsrfConfig) -> Option<String> {
    match ip {
        IpAddr::V4(ipv4) => check_ipv4_blocked(ipv4, config),
        IpAddr::V6(ipv6) => {
            // First, check if this is an IPv4-mapped or IPv4-compatible IPv6 address
            // and apply IPv4 rules if so (prevents bypass via ::ffff:127.0.0.1)
            if let Some(mapped_ipv4) = get_ipv4_from_mapped(ipv6) {
                if let Some(reason) = check_ipv4_blocked(&mapped_ipv4, config) {
                    return Some(format!("{} (IPv4-mapped)", reason));
                }
            }

            // Apply IPv6-specific checks
            if config.block_loopback && ipv6.is_loopback() {
                return Some("loopback address blocked".into());
            }
            if config.block_private && is_private_ipv6(ipv6) {
                return Some("private IP range blocked".into());
            }
            if config.block_link_local && (ipv6.segments()[0] & 0xffc0) == 0xfe80 {
                return Some("link-local address blocked".into());
            }
            if config.block_multicast && ipv6.is_multicast() {
                return Some("multicast address blocked".into());
            }
            // Block unspecified address ::
            if ipv6.is_unspecified() {
                return Some("unspecified address blocked".into());
            }
            // Block AWS IMDSv2 IPv6 metadata endpoint (fd00:ec2::254)
            let segments = ipv6.segments();
            if segments[0] == 0xfd00 && segments[1] == 0x0ec2
                && segments[2..7] == [0, 0, 0, 0, 0]
                && segments[7] == 0x0254
            {
                return Some("cloud metadata endpoint blocked (IPv6)".into());
            }
            None
        }
    }
}

/// Check if an IPv4 address should be blocked
fn check_ipv4_blocked(ipv4: &Ipv4Addr, config: &SsrfConfig) -> Option<String> {
    if config.block_loopback && ipv4.is_loopback() {
        return Some("loopback address blocked".into());
    }
    if config.block_private && is_private_ipv4(ipv4) {
        return Some("private IP range blocked".into());
    }
    if config.block_link_local && ipv4.is_link_local() {
        return Some("link-local address blocked".into());
    }
    if config.block_multicast && ipv4.is_multicast() {
        return Some("multicast address blocked".into());
    }
    // Block broadcast
    if ipv4.is_broadcast() {
        return Some("broadcast address blocked".into());
    }
    // Block documentation ranges (192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24)
    if (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 2)
        || (ipv4.octets()[0] == 198 && ipv4.octets()[1] == 51 && ipv4.octets()[2] == 100)
        || (ipv4.octets()[0] == 203 && ipv4.octets()[1] == 0 && ipv4.octets()[2] == 113)
    {
        return Some("documentation IP range blocked".into());
    }
    // Block 0.0.0.0/8 (this network)
    if ipv4.octets()[0] == 0 {
        return Some("this-network address blocked".into());
    }
    // Block AWS/cloud metadata endpoints (169.254.169.254)
    if ipv4.octets()[0] == 169 && ipv4.octets()[1] == 254
        && ipv4.octets()[2] == 169 && ipv4.octets()[3] == 254
    {
        return Some("cloud metadata endpoint blocked".into());
    }
    None
}

/// Check if a host matches an allowlist pattern
fn is_host_in_allowlist(host: &str, allowlist: &[String]) -> bool {
    for pattern in allowlist {
        // Exact match
        if pattern == host {
            return true;
        }
        // Wildcard match (*.example.com)
        if pattern.starts_with("*.") {
            let suffix = &pattern[1..]; // .example.com
            if host.ends_with(suffix) || host == &pattern[2..] {
                return true;
            }
        }
    }
    false
}

/// Validate a URL for SSRF protection (legacy, without DNS pinning)
///
/// WARNING: This function is vulnerable to DNS rebinding attacks.
/// Use `validate_url_for_ssrf_with_pinning()` for new code.
///
/// This function:
/// 1. Parses and validates the URL
/// 2. Checks if the host is in the allowlist (bypass other checks if so)
/// 3. Resolves DNS and checks all resulting IPs against blocked ranges
///
/// Returns Ok(()) if the URL is safe to access, Err otherwise.
#[deprecated(since = "0.3.1", note = "Use validate_url_for_ssrf_with_pinning to prevent DNS rebinding")]
pub fn validate_url_for_ssrf(url_str: &str, config: &SsrfConfig) -> Result<Url, NetworkError> {
    // Parse URL
    let url = Url::parse(url_str)
        .map_err(|e| NetworkError::InvalidUrl(e.to_string()))?;

    // Only allow http and https schemes
    match url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(NetworkError::InvalidUrl(format!(
                "Invalid scheme '{}'. Only http and https are allowed.",
                scheme
            )));
        }
    }

    // Get host
    let host = url.host_str()
        .ok_or_else(|| NetworkError::InvalidUrl("URL has no host".into()))?;

    // Check allowlist first (explicit user opt-in bypasses other checks)
    if is_host_in_allowlist(host, &config.allowlist) {
        return Ok(url);
    }

    // Get port (default to 80/443)
    let port = url.port_or_known_default()
        .ok_or_else(|| NetworkError::InvalidUrl("Cannot determine port".into()))?;

    // Resolve DNS and check all resulting IPs
    let socket_addr = format!("{}:{}", host, port);
    let addrs: Vec<_> = socket_addr
        .to_socket_addrs()
        .map_err(|e| NetworkError::DnsResolutionFailed(e.to_string()))?
        .collect();

    if addrs.is_empty() {
        return Err(NetworkError::DnsResolutionFailed(
            "DNS resolution returned no addresses".into()
        ));
    }

    // Check ALL resolved IPs (DNS rebinding protection)
    for addr in &addrs {
        if let Some(reason) = is_ip_blocked(&addr.ip(), config) {
            return Err(NetworkError::SsrfBlocked(format!(
                "Host '{}' resolves to blocked IP {}: {}",
                host, addr.ip(), reason
            )));
        }
    }

    Ok(url)
}

/// Validate a URL for SSRF protection with DNS pinning
///
/// This function prevents DNS rebinding attacks by:
/// 1. Resolving DNS once at validation time
/// 2. Validating ALL resolved IPs against SSRF rules
/// 3. Returning the validated IPs for use in subsequent connections
///
/// The caller MUST use the returned pinned IPs for the actual connection,
/// bypassing any further DNS resolution.
///
/// # Security
/// This is the recommended function for SSRF protection. It eliminates
/// the TOCTOU vulnerability where DNS could change between validation
/// and connection time.
pub async fn validate_url_for_ssrf_with_pinning(
    url_str: &str,
    resolver: &PinnedDnsResolver,
) -> Result<ValidatedUrl, NetworkError> {
    // Parse URL
    let url = Url::parse(url_str)
        .map_err(|e| NetworkError::InvalidUrl(e.to_string()))?;

    // Only allow http and https schemes
    match url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(NetworkError::InvalidUrl(format!(
                "Invalid scheme '{}'. Only http and https are allowed.",
                scheme
            )));
        }
    }

    // Get host for allowlist check
    let host = url.host_str()
        .ok_or_else(|| NetworkError::InvalidUrl("URL has no host".into()))?;

    // Check allowlist first (explicit user opt-in bypasses DNS validation)
    if is_host_in_allowlist(host, &resolver.ssrf_config().allowlist) {
        // For allowlisted hosts, still resolve but don't validate IPs
        let port = url.port_or_known_default().unwrap_or(80);
        return Ok(ValidatedUrl {
            url: url.clone(),
            host: host.to_string(),
            port,
            pinned_ips: vec![], // Empty = use normal DNS
        });
    }

    // Use pinned resolver to validate and get IPs
    resolver.resolve_and_validate(&url).await.map_err(|e| match e {
        DnsError::ResolutionFailed(msg) => NetworkError::DnsResolutionFailed(msg),
        DnsError::NoAddresses(host) => NetworkError::DnsResolutionFailed(
            format!("DNS resolution returned no addresses for {}", host)
        ),
        DnsError::AllBlocked(msg) => NetworkError::SsrfBlocked(msg),
        DnsError::NotValidated(host) => NetworkError::SsrfBlocked(
            format!("Host '{}' was not pre-validated", host)
        ),
        DnsError::ResolverError(msg) => NetworkError::DnsResolutionFailed(msg),
    })
}

/// Validate a redirect target URL with DNS pinning
///
/// This should be called for each redirect to prevent SSRF via redirect chains.
pub async fn validate_redirect_with_pinning(
    url_str: &str,
    resolver: &PinnedDnsResolver,
    redirect_count: usize,
    max_redirects: usize,
) -> Result<ValidatedUrl, NetworkError> {
    if redirect_count >= max_redirects {
        return Err(NetworkError::RequestFailed(format!(
            "Too many redirects (max: {})",
            max_redirects
        )));
    }

    validate_url_for_ssrf_with_pinning(url_str, resolver).await
}

/// Canonicalize a URL (normalize for deduplication)
pub fn canonicalize_url(url_str: &str) -> Result<String, NetworkError> {
    let mut url = Url::parse(url_str)
        .map_err(|e| NetworkError::InvalidUrl(e.to_string()))?;

    // Lowercase scheme and host
    // URL crate already does this, but be explicit

    // Remove default ports
    if (url.scheme() == "http" && url.port() == Some(80))
        || (url.scheme() == "https" && url.port() == Some(443))
    {
        url.set_port(None).ok();
    }

    // Remove trailing slash from path if it's just "/"
    if url.path() == "/" && url.query().is_none() && url.fragment().is_none() {
        // Keep as-is, "/" is canonical for root
    }

    // Remove fragment (not sent to server)
    url.set_fragment(None);

    // Sort query parameters for consistency (optional, depends on requirements)
    // For now, we keep query as-is since order might matter

    Ok(url.to_string())
}

/// Check if a URL looks like a login/SSO page
pub fn is_login_page(url: &Url, content: Option<&str>) -> bool {
    let url_str = url.as_str().to_lowercase();
    let path = url.path().to_lowercase();

    // Check URL patterns
    let login_patterns = [
        "/login", "/signin", "/sign-in", "/sso", "/oauth",
        "/auth", "/authenticate", "/saml", "/adfs",
        "/accounts/login", "/user/login", "/session/new",
        "login.microsoftonline.com", "accounts.google.com",
        "login.salesforce.com", "sso.", "auth.",
    ];

    for pattern in &login_patterns {
        if url_str.contains(pattern) || path.contains(pattern) {
            return true;
        }
    }

    // Check content for login form indicators
    if let Some(content) = content {
        let content_lower = content.to_lowercase();
        let content_patterns = [
            "type=\"password\"",
            "name=\"password\"",
            "id=\"password\"",
            "sign in to your account",
            "log in to continue",
            "enter your credentials",
            "authentication required",
        ];

        for pattern in &content_patterns {
            if content_lower.contains(pattern) {
                return true;
            }
        }
    }

    false
}

/// Extract same-origin links from HTML content
pub fn extract_same_origin_links(base_url: &Url, html: &str) -> Vec<String> {
    let mut links = Vec::new();
    let base_host = base_url.host_str().unwrap_or("");

    // Simple regex-like extraction (avoid full HTML parser for performance)
    // This is intentionally conservative to avoid XSS vectors
    for cap in regex_lite::Regex::new(r#"href\s*=\s*["']([^"']+)["']"#)
        .unwrap()
        .captures_iter(html)
    {
        if let Some(href) = cap.get(1) {
            let href_str = href.as_str();

            // Skip javascript:, mailto:, tel:, etc.
            if href_str.starts_with("javascript:")
                || href_str.starts_with("mailto:")
                || href_str.starts_with("tel:")
                || href_str.starts_with("#")
            {
                continue;
            }

            // Resolve relative URLs
            if let Ok(resolved) = base_url.join(href_str) {
                // Only include same-origin links
                if resolved.host_str() == Some(base_host) {
                    links.push(resolved.to_string());
                }
            }
        }
    }

    // Deduplicate
    links.sort();
    links.dedup();

    links
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_private_ipv4() {
        assert!(is_private_ipv4(&Ipv4Addr::new(10, 0, 0, 1)));
        assert!(is_private_ipv4(&Ipv4Addr::new(10, 255, 255, 255)));
        assert!(is_private_ipv4(&Ipv4Addr::new(172, 16, 0, 1)));
        assert!(is_private_ipv4(&Ipv4Addr::new(172, 31, 255, 255)));
        assert!(is_private_ipv4(&Ipv4Addr::new(192, 168, 0, 1)));
        assert!(is_private_ipv4(&Ipv4Addr::new(192, 168, 255, 255)));
        assert!(is_private_ipv4(&Ipv4Addr::new(100, 64, 0, 1))); // CGNAT

        assert!(!is_private_ipv4(&Ipv4Addr::new(8, 8, 8, 8)));
        assert!(!is_private_ipv4(&Ipv4Addr::new(1, 1, 1, 1)));
        assert!(!is_private_ipv4(&Ipv4Addr::new(172, 15, 0, 1))); // Just outside 172.16.0.0/12
        assert!(!is_private_ipv4(&Ipv4Addr::new(172, 32, 0, 1)));
    }

    #[test]
    fn test_is_ip_blocked() {
        let config = SsrfConfig::default();

        // Loopback
        assert!(is_ip_blocked(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), &config).is_some());
        assert!(is_ip_blocked(&IpAddr::V6(Ipv6Addr::LOCALHOST), &config).is_some());

        // Private
        assert!(is_ip_blocked(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), &config).is_some());
        assert!(is_ip_blocked(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), &config).is_some());

        // Public (should not be blocked)
        assert!(is_ip_blocked(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), &config).is_none());
        assert!(is_ip_blocked(&IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), &config).is_none());

        // Cloud metadata endpoint
        assert!(is_ip_blocked(&IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254)), &config).is_some());
    }

    #[test]
    fn test_ipv6_mapped_ipv4_blocked() {
        let config = SsrfConfig::default();

        // IPv6-mapped loopback (::ffff:127.0.0.1)
        let mapped_loopback = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 0x0001);
        let result = is_ip_blocked(&IpAddr::V6(mapped_loopback), &config);
        assert!(result.is_some(), "IPv6-mapped loopback should be blocked");
        assert!(result.unwrap().contains("IPv4-mapped"));

        // IPv6-mapped private range (::ffff:192.168.1.1)
        let mapped_private = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc0a8, 0x0101);
        let result = is_ip_blocked(&IpAddr::V6(mapped_private), &config);
        assert!(result.is_some(), "IPv6-mapped private should be blocked");

        // IPv6-mapped 10.x.x.x (::ffff:10.0.0.1)
        let mapped_10 = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x0a00, 0x0001);
        let result = is_ip_blocked(&IpAddr::V6(mapped_10), &config);
        assert!(result.is_some(), "IPv6-mapped 10.0.0.1 should be blocked");

        // IPv6-mapped cloud metadata (::ffff:169.254.169.254)
        let mapped_metadata = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xa9fe, 0xa9fe);
        let result = is_ip_blocked(&IpAddr::V6(mapped_metadata), &config);
        assert!(result.is_some(), "IPv6-mapped metadata endpoint should be blocked");

        // IPv6-mapped public (::ffff:8.8.8.8) should NOT be blocked
        let mapped_public = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x0808, 0x0808);
        let result = is_ip_blocked(&IpAddr::V6(mapped_public), &config);
        assert!(result.is_none(), "IPv6-mapped public IP should not be blocked");
    }

    #[test]
    fn test_ipv4_compatible_ipv6_blocked() {
        let config = SsrfConfig::default();

        // IPv4-compatible loopback (::127.0.0.1, deprecated but should still be blocked)
        let compat_loopback = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0x7f00, 0x0001);
        let result = is_ip_blocked(&IpAddr::V6(compat_loopback), &config);
        assert!(result.is_some(), "IPv4-compatible loopback should be blocked");
    }

    #[test]
    fn test_allowlist() {
        assert!(is_host_in_allowlist("example.com", &["example.com".into()]));
        assert!(is_host_in_allowlist("sub.example.com", &["*.example.com".into()]));
        assert!(is_host_in_allowlist("example.com", &["*.example.com".into()]));
        assert!(!is_host_in_allowlist("evil.com", &["example.com".into()]));
        assert!(!is_host_in_allowlist("notexample.com", &["*.example.com".into()]));
    }

    #[test]
    #[allow(deprecated)]
    fn test_validate_url_blocked() {
        let config = SsrfConfig::default();

        // These should be blocked
        assert!(validate_url_for_ssrf("http://localhost/", &config).is_err());
        assert!(validate_url_for_ssrf("http://127.0.0.1/", &config).is_err());
        assert!(validate_url_for_ssrf("http://[::1]/", &config).is_err());
        assert!(validate_url_for_ssrf("http://192.168.1.1/", &config).is_err());
        assert!(validate_url_for_ssrf("http://10.0.0.1/", &config).is_err());

        // Invalid schemes
        assert!(validate_url_for_ssrf("ftp://example.com/", &config).is_err());
        assert!(validate_url_for_ssrf("file:///etc/passwd", &config).is_err());
    }

    #[test]
    #[allow(deprecated)]
    fn test_validate_url_allowed() {
        let mut config = SsrfConfig::default();

        // Public IPs should be allowed
        // Note: This test requires DNS resolution, so it may fail in isolated environments
        // assert!(validate_url_for_ssrf("https://example.com/", &config).is_ok());

        // Allowlist should bypass checks
        config.allowlist.push("localhost".into());
        assert!(validate_url_for_ssrf("http://localhost/", &config).is_ok());
    }

    #[test]
    fn test_canonicalize_url() {
        assert_eq!(
            canonicalize_url("HTTP://EXAMPLE.COM/Path").unwrap(),
            "http://example.com/Path"
        );
        assert_eq!(
            canonicalize_url("https://example.com:443/path").unwrap(),
            "https://example.com/path"
        );
        assert_eq!(
            canonicalize_url("http://example.com:80/path").unwrap(),
            "http://example.com/path"
        );
        assert_eq!(
            canonicalize_url("https://example.com/path#fragment").unwrap(),
            "https://example.com/path"
        );
    }

    #[test]
    fn test_is_login_page() {
        let url = Url::parse("https://login.example.com/").unwrap();
        assert!(is_login_page(&url, None));

        let url = Url::parse("https://example.com/signin").unwrap();
        assert!(is_login_page(&url, None));

        let url = Url::parse("https://example.com/about").unwrap();
        assert!(!is_login_page(&url, None));

        let url = Url::parse("https://example.com/").unwrap();
        assert!(is_login_page(&url, Some("<input type=\"password\">")));
    }

    #[test]
    fn test_extract_same_origin_links() {
        let base = Url::parse("https://example.com/page").unwrap();
        let html = r#"
            <a href="/other">Other</a>
            <a href="https://example.com/about">About</a>
            <a href="https://evil.com/bad">Evil</a>
            <a href="javascript:alert(1)">XSS</a>
            <a href="relative.html">Relative</a>
        "#;

        let links = extract_same_origin_links(&base, html);
        assert!(links.contains(&"https://example.com/other".to_string()));
        assert!(links.contains(&"https://example.com/about".to_string()));
        assert!(links.contains(&"https://example.com/relative.html".to_string()));
        assert!(!links.iter().any(|l| l.contains("evil.com")));
        assert!(!links.iter().any(|l| l.contains("javascript:")));
    }
}
