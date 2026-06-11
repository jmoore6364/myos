// DNS - Domain Name System
// Resolves hostnames to IP addresses

use super::ipv4::Ipv4Addr;

/// DNS query type
#[derive(Debug, Clone, Copy)]
pub enum DnsQueryType {
    A = 1,     // IPv4 address
    AAAA = 28, // IPv6 address
    MX = 15,   // Mail exchange
    CNAME = 5, // Canonical name
}

/// Simple DNS query structure
pub struct DnsQuery {
    pub hostname: alloc::string::String,
    pub query_type: DnsQueryType,
}

impl DnsQuery {
    pub fn new(hostname: &str) -> Self {
        DnsQuery {
            hostname: alloc::string::String::from(hostname),
            query_type: DnsQueryType::A,
        }
    }
}

/// Resolve hostname to IP address
pub fn resolve(hostname: &str) -> Result<Ipv4Addr, &'static str> {
    crate::println!("DNS: Resolving '{}'...", hostname);

    // TODO: Implement actual DNS resolution:
    // 1. Build DNS query packet
    // 2. Send UDP packet to DNS server (typically 8.8.8.8:53 or configured server)
    // 3. Parse DNS response
    // 4. Extract IP address from answer section

    crate::println!("DNS: Not yet implemented");
    crate::println!("DNS: Would send UDP query to DNS server (8.8.8.8:53)");
    crate::println!("DNS: Would parse response and return IP address");

    Err("DNS resolution not yet implemented")
}

/// Static DNS cache for testing
pub fn resolve_static(hostname: &str) -> Option<Ipv4Addr> {
    // Hardcoded entries for testing
    match hostname {
        "localhost" => Some(Ipv4Addr::LOCALHOST),
        "router" => Some(Ipv4Addr::new(192, 168, 1, 1)),
        "google.com" => Some(Ipv4Addr::new(142, 250, 185, 46)),
        "example.com" => Some(Ipv4Addr::new(93, 184, 216, 34)),
        _ => None,
    }
}
