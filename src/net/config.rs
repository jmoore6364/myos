// Network Configuration
// Stores network interface settings

use super::ethernet::MacAddr;
use super::ipv4::Ipv4Addr;
use spin::Mutex;

/// Network interface configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub ip_addr: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub dns_server: Ipv4Addr,
    pub mac_addr: MacAddr,
}

impl NetworkConfig {
    pub const fn new() -> Self {
        NetworkConfig {
            ip_addr: Ipv4Addr::ZERO,
            netmask: Ipv4Addr::ZERO,
            gateway: Ipv4Addr::ZERO,
            dns_server: Ipv4Addr::ZERO,
            mac_addr: MacAddr::ZERO,
        }
    }

    /// Check if we're on the same subnet
    pub fn is_local(&self, ip: Ipv4Addr) -> bool {
        let our_network = self.ip_addr.as_u32() & self.netmask.as_u32();
        let their_network = ip.as_u32() & self.netmask.as_u32();
        our_network == their_network
    }

    /// Get next hop for destination IP (either direct or via gateway)
    pub fn next_hop(&self, dest: Ipv4Addr) -> Ipv4Addr {
        if self.is_local(dest) {
            dest // Direct delivery
        } else {
            self.gateway // Via gateway
        }
    }
}

/// Global network configuration
pub static NET_CONFIG: Mutex<NetworkConfig> = Mutex::new(NetworkConfig::new());

/// Configure network interface
pub fn configure(ip: Ipv4Addr, netmask: Ipv4Addr, gateway: Ipv4Addr, mac: MacAddr) {
    let mut config = NET_CONFIG.lock();
    config.ip_addr = ip;
    config.netmask = netmask;
    config.gateway = gateway;
    config.dns_server = Ipv4Addr::new(8, 8, 8, 8); // Google DNS
    config.mac_addr = mac;

    crate::println!("Network configured:");
    crate::println!("  IP:      {}", ip);
    crate::println!("  Netmask: {}", netmask);
    crate::println!("  Gateway: {}", gateway);
    crate::println!("  MAC:     {}", mac);
}

/// Get current configuration
pub fn get_config() -> NetworkConfig {
    NET_CONFIG.lock().clone()
}

/// Get our IP address
pub fn get_ip() -> Ipv4Addr {
    NET_CONFIG.lock().ip_addr
}

/// Get our MAC address
pub fn get_mac() -> MacAddr {
    NET_CONFIG.lock().mac_addr
}
