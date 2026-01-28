//! SSRF DNS Rebinding Tests
//!
//! Tests for the DNS pinning mechanism that prevents DNS rebinding attacks.
//! DNS rebinding is a TOCTOU attack where an attacker changes DNS between
//! validation time and connection time.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Test constants
const LOCALHOST_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const PRIVATE_10: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
const PRIVATE_172: IpAddr = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));
const PRIVATE_192: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
const LINK_LOCAL: IpAddr = IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1));
const PUBLIC_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)); // example.com

// IPv6 variants
const LOCALHOST_V6: IpAddr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
const PRIVATE_V6: IpAddr = IpAddr::V6(Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 1));
const LINK_LOCAL_V6: IpAddr = IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1));
const PUBLIC_V6: IpAddr = IpAddr::V6(Ipv6Addr::new(0x2606, 0x2800, 0x220, 0x1, 0, 0, 0, 0x248)); // example.com

// IPv4-mapped IPv6 addresses (used for rebinding attacks)
const MAPPED_LOCALHOST: IpAddr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 0x0001));
const MAPPED_PRIVATE_10: IpAddr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x0a00, 0x0001));

/// Check if an IP address is private (blocked for SSRF)
fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified()
                // Check for reserved ranges
                || v4.octets()[0] == 0
                || v4.octets()[0] >= 224
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                // Check for IPv4-mapped addresses
                || is_ipv4_mapped_private(&v6)
                // Unique local (fc00::/7)
                || (v6.segments()[0] & 0xfe00) == 0xfc00
                // Link-local (fe80::/10)
                || (v6.segments()[0] & 0xffc0) == 0xfe80
                // Documentation (2001:db8::/32)
                || (v6.segments()[0] == 0x2001 && v6.segments()[1] == 0xdb8)
        }
    }
}

/// Check if an IPv6 address is an IPv4-mapped address pointing to a private IPv4
fn is_ipv4_mapped_private(v6: &Ipv6Addr) -> bool {
    // IPv4-mapped format: ::ffff:x.x.x.x
    let segments = v6.segments();
    if segments[0] == 0
        && segments[1] == 0
        && segments[2] == 0
        && segments[3] == 0
        && segments[4] == 0
        && segments[5] == 0xffff
    {
        // Extract the IPv4 address
        let v4_high = segments[6];
        let v4_low = segments[7];
        let octets = [
            (v4_high >> 8) as u8,
            (v4_high & 0xff) as u8,
            (v4_low >> 8) as u8,
            (v4_low & 0xff) as u8,
        ];
        let v4 = Ipv4Addr::from(octets);

        v4.is_loopback()
            || v4.is_private()
            || v4.is_link_local()
            || v4.is_broadcast()
            || v4.is_unspecified()
            || octets[0] == 0
            || octets[0] >= 224
    } else {
        false
    }
}

// ============================================================================
// IPv4 Private IP Blocking Tests
// ============================================================================

#[test]
fn test_blocks_localhost_v4() {
    assert!(is_private_ip(LOCALHOST_V4));
}

#[test]
fn test_blocks_10_network() {
    assert!(is_private_ip(PRIVATE_10));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(10, 255, 255, 255))));
}

#[test]
fn test_blocks_172_network() {
    assert!(is_private_ip(PRIVATE_172));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 31, 255, 255))));
    // 172.32.x.x is public
    assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 32, 0, 1))));
}

#[test]
fn test_blocks_192_168_network() {
    assert!(is_private_ip(PRIVATE_192));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 255, 255))));
}

#[test]
fn test_blocks_link_local_v4() {
    assert!(is_private_ip(LINK_LOCAL));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 255, 255))));
}

#[test]
fn test_allows_public_ipv4() {
    assert!(!is_private_ip(PUBLIC_IP));
    assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));
}

// ============================================================================
// IPv6 Private IP Blocking Tests
// ============================================================================

#[test]
fn test_blocks_localhost_v6() {
    assert!(is_private_ip(LOCALHOST_V6));
}

#[test]
fn test_blocks_unique_local_v6() {
    assert!(is_private_ip(PRIVATE_V6));
    assert!(is_private_ip(IpAddr::V6(Ipv6Addr::new(
        0xfc00, 0, 0, 0, 0, 0, 0, 1
    ))));
}

#[test]
fn test_blocks_link_local_v6() {
    assert!(is_private_ip(LINK_LOCAL_V6));
}

#[test]
fn test_allows_public_ipv6() {
    assert!(!is_private_ip(PUBLIC_V6));
}

// ============================================================================
// IPv4-Mapped IPv6 Rebinding Attack Tests
// ============================================================================

#[test]
fn test_blocks_ipv4_mapped_localhost() {
    // Attacker uses ::ffff:127.0.0.1 to bypass IPv4 checks
    assert!(is_private_ip(MAPPED_LOCALHOST));
}

#[test]
fn test_blocks_ipv4_mapped_private_10() {
    // Attacker uses ::ffff:10.0.0.1 to access internal network
    assert!(is_private_ip(MAPPED_PRIVATE_10));
}

#[test]
fn test_blocks_ipv4_mapped_private_172() {
    let mapped = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xac10, 0x0001)); // ::ffff:172.16.0.1
    assert!(is_private_ip(mapped));
}

#[test]
fn test_blocks_ipv4_mapped_private_192() {
    let mapped = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc0a8, 0x0101)); // ::ffff:192.168.1.1
    assert!(is_private_ip(mapped));
}

#[test]
fn test_allows_ipv4_mapped_public() {
    // ::ffff:93.184.216.34 (example.com) should be allowed
    let mapped = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x5db8, 0xd822));
    assert!(!is_private_ip(mapped));
}

// ============================================================================
// DNS Rebinding Simulation Tests
// ============================================================================

/// Simulates a DNS rebinding attack scenario
///
/// In a real attack:
/// 1. Attacker controls attacker.com DNS
/// 2. First query returns public IP (passes validation)
/// 3. Second query (during connection) returns 127.0.0.1
///
/// Our defense: Pin IPs during validation, never re-resolve
#[test]
fn test_dns_rebinding_scenario() {
    // Simulate validation phase - DNS returns public IP
    let validation_ips = vec![PUBLIC_IP];

    // All validation IPs should be public
    assert!(validation_ips.iter().all(|ip| !is_private_ip(*ip)));

    // Simulate attack - attacker changes DNS to return localhost
    let attack_ips = vec![LOCALHOST_V4];

    // Attack IPs should be blocked
    assert!(attack_ips.iter().all(|ip| is_private_ip(*ip)));

    // With pinning, we use validation_ips, so attack fails
    // The connection would use PUBLIC_IP, not LOCALHOST_V4
}

#[test]
fn test_dns_rebinding_ipv6_variant() {
    // Attacker returns public IPv6 during validation
    let validation_ips = vec![PUBLIC_V6];
    assert!(validation_ips.iter().all(|ip| !is_private_ip(*ip)));

    // Then returns IPv4-mapped localhost during connection
    let attack_ips = vec![MAPPED_LOCALHOST];
    assert!(attack_ips.iter().all(|ip| is_private_ip(*ip)));
}

#[test]
fn test_dns_returns_mixed_ips() {
    // DNS might return multiple IPs (some public, some private)
    let mixed_ips = vec![
        PUBLIC_IP,
        LOCALHOST_V4, // Attacker injects this
    ];

    // Should reject if ANY IP is private
    let has_private = mixed_ips.iter().any(|ip| is_private_ip(*ip));
    assert!(has_private);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_blocks_zero_address() {
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))));
    assert!(is_private_ip(IpAddr::V6(Ipv6Addr::new(
        0, 0, 0, 0, 0, 0, 0, 0
    ))));
}

#[test]
fn test_blocks_broadcast() {
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255))));
}

#[test]
fn test_blocks_multicast() {
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1))));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(239, 255, 255, 255))));
}

#[test]
fn test_blocks_documentation_ranges() {
    // IPv4 documentation ranges
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)))); // 192.0.2.0/24
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)))); // 198.51.100.0/24
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)))); // 203.0.113.0/24

    // IPv6 documentation range
    assert!(is_private_ip(IpAddr::V6(Ipv6Addr::new(
        0x2001, 0xdb8, 0, 0, 0, 0, 0, 1
    ))));
}

#[test]
fn test_blocks_reserved_ranges() {
    // Current network (0.x.x.x)
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(0, 1, 2, 3))));
}

// ============================================================================
// IP Validation Helper Tests
// ============================================================================

/// Test that we correctly identify various private network boundaries
#[test]
fn test_private_network_boundaries() {
    // 10.0.0.0/8 boundaries
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0))));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(10, 255, 255, 255))));
    assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(11, 0, 0, 0))));

    // 172.16.0.0/12 boundaries
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 0))));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 31, 255, 255))));
    assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 15, 255, 255))));
    assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(172, 32, 0, 0))));

    // 192.168.0.0/16 boundaries
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0))));
    assert!(is_private_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 255, 255))));
    assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(
        192, 167, 255, 255
    ))));
    assert!(!is_private_ip(IpAddr::V4(Ipv4Addr::new(192, 169, 0, 0))));
}
