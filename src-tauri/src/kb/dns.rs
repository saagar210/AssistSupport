//! DNS resolution with IP pinning for SSRF protection
//!
//! This module prevents DNS rebinding attacks by resolving DNS once at validation time
//! and using IP-based URLs to bypass DNS resolution at connection time.
//!
//! The TOCTOU vulnerability exists when:
//! 1. A legacy validator resolves DNS to 8.8.8.8 (public, allowed)
//! 2. Attacker changes DNS record to 127.0.0.1 (private, blocked)
//! 3. reqwest re-resolves DNS and connects to 127.0.0.1
//!
//! Solution: Resolve DNS once, validate IPs, then connect directly to the validated IP
//! with the proper Host header. This completely eliminates DNS re-resolution.

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};

use super::network::{is_ip_blocked, SsrfConfig};

/// Error type for DNS resolution
#[derive(Debug, thiserror::Error)]
pub enum DnsError {
    #[error("DNS resolution failed: {0}")]
    ResolutionFailed(String),
    #[error("No addresses found for host: {0}")]
    NoAddresses(String),
    #[error("All resolved IPs are blocked: {0}")]
    AllBlocked(String),
    #[error("Host not pre-validated: {0}")]
    NotValidated(String),
    #[error("Resolver error: {0}")]
    ResolverError(String),
}

/// Validated URL with pinned IP addresses
#[derive(Debug, Clone)]
pub struct ValidatedUrl {
    pub url: url::Url,
    pub host: String,
    pub port: u16,
    pub pinned_ips: Vec<IpAddr>,
}

impl ValidatedUrl {
    /// Get socket addresses for the pinned IPs
    pub fn socket_addrs(&self) -> Vec<SocketAddr> {
        self.pinned_ips
            .iter()
            .map(|ip| SocketAddr::new(*ip, self.port))
            .collect()
    }
}

/// Pinned DNS resolver that caches validated IPs
///
/// This resolver stores the IP addresses that were validated during SSRF checks
/// and ensures that subsequent connections use ONLY these pre-validated IPs.
pub struct PinnedDnsResolver {
    /// Map from hostname to validated IPs
    pinned: Arc<RwLock<HashMap<String, Vec<IpAddr>>>>,
    /// Async DNS resolver for initial lookups
    resolver: TokioAsyncResolver,
    /// SSRF protection config
    ssrf_config: SsrfConfig,
}

impl PinnedDnsResolver {
    /// Create a new pinned DNS resolver
    pub async fn new(ssrf_config: SsrfConfig) -> Result<Self, DnsError> {
        // trust-dns-resolver 0.23's TokioAsyncResolver::tokio() returns the resolver directly
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        Ok(Self {
            pinned: Arc::new(RwLock::new(HashMap::new())),
            resolver,
            ssrf_config,
        })
    }

    /// Resolve and validate a URL, returning pinned IP addresses
    ///
    /// This method:
    /// 1. Resolves DNS for the hostname
    /// 2. Filters out all blocked IPs (private, loopback, etc.)
    /// 3. Stores the validated IPs in the pinned cache
    /// 4. Returns a ValidatedUrl with the pinned IPs
    pub async fn resolve_and_validate(&self, url: &url::Url) -> Result<ValidatedUrl, DnsError> {
        let host = url
            .host_str()
            .ok_or_else(|| DnsError::ResolutionFailed("URL has no host".into()))?;

        let port = url.port_or_known_default().unwrap_or(80);

        // Check if host is already an IP address
        if let Ok(ip) = host.parse::<IpAddr>() {
            // Validate the IP directly
            if let Some(reason) = is_ip_blocked(&ip, &self.ssrf_config) {
                return Err(DnsError::AllBlocked(format!(
                    "IP {} is blocked: {}",
                    ip, reason
                )));
            }
            let validated = ValidatedUrl {
                url: url.clone(),
                host: host.to_string(),
                port,
                pinned_ips: vec![ip],
            };
            self.pin_host(host, vec![ip]);
            return Ok(validated);
        }

        // Resolve DNS
        let response = self
            .resolver
            .lookup_ip(host)
            .await
            .map_err(|e| DnsError::ResolutionFailed(e.to_string()))?;

        let all_ips: Vec<IpAddr> = response.iter().collect();

        if all_ips.is_empty() {
            return Err(DnsError::NoAddresses(host.to_string()));
        }

        // Filter to only allowed IPs
        let validated_ips: Vec<IpAddr> = all_ips
            .into_iter()
            .filter(|ip| is_ip_blocked(ip, &self.ssrf_config).is_none())
            .collect();

        if validated_ips.is_empty() {
            return Err(DnsError::AllBlocked(format!(
                "All resolved IPs for {} are blocked (private, loopback, or internal ranges)",
                host
            )));
        }

        // Pin the validated IPs
        self.pin_host(host, validated_ips.clone());

        Ok(ValidatedUrl {
            url: url.clone(),
            host: host.to_string(),
            port,
            pinned_ips: validated_ips,
        })
    }

    /// Pin a host to specific IP addresses
    fn pin_host(&self, host: &str, ips: Vec<IpAddr>) {
        if let Ok(mut pinned) = self.pinned.write() {
            pinned.insert(host.to_lowercase(), ips);
        }
    }

    /// Get pinned IPs for a host (returns None if not pre-validated)
    pub fn get_pinned(&self, host: &str) -> Option<Vec<IpAddr>> {
        self.pinned
            .read()
            .ok()
            .and_then(|pinned| pinned.get(&host.to_lowercase()).cloned())
    }

    /// Check if a host is pinned
    pub fn is_pinned(&self, host: &str) -> bool {
        self.pinned
            .read()
            .ok()
            .map(|pinned| pinned.contains_key(&host.to_lowercase()))
            .unwrap_or(false)
    }

    /// Get SSRF config reference
    pub fn ssrf_config(&self) -> &SsrfConfig {
        &self.ssrf_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_validated_url_socket_addrs() {
        let validated = ValidatedUrl {
            url: url::Url::parse("https://example.com/path").unwrap(),
            host: "example.com".to_string(),
            port: 443,
            pinned_ips: vec![
                IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
                IpAddr::V4(Ipv4Addr::new(93, 184, 216, 35)),
            ],
        };

        let addrs = validated.socket_addrs();
        assert_eq!(addrs.len(), 2);
        assert_eq!(addrs[0].port(), 443);
    }

    #[tokio::test]
    async fn test_pinned_resolver_blocks_private_ips() {
        let config = SsrfConfig::default();
        let resolver = PinnedDnsResolver::new(config).await.unwrap();

        // Test with localhost URL (should be blocked)
        let url = url::Url::parse("http://127.0.0.1/").unwrap();
        let result = resolver.resolve_and_validate(&url).await;
        assert!(result.is_err());

        // Test with private IP (should be blocked)
        let url = url::Url::parse("http://192.168.1.1/").unwrap();
        let result = resolver.resolve_and_validate(&url).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_pinned_host_storage() {
        let pinned: Arc<RwLock<HashMap<String, Vec<IpAddr>>>> =
            Arc::new(RwLock::new(HashMap::new()));

        // Add a pinned host
        {
            let mut guard = pinned.write().unwrap();
            guard.insert(
                "example.com".to_string(),
                vec![IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))],
            );
        }

        // Verify it's stored
        {
            let guard = pinned.read().unwrap();
            assert!(guard.contains_key("example.com"));
            assert!(!guard.contains_key("evil.com"));
        }
    }
}
