// Network stack implementation
// Building a TCP/IP stack from scratch!

use alloc::vec;

pub mod ethernet;
pub mod arp;
pub mod ipv4;
pub mod icmp;
pub mod udp;
pub mod tcp;
pub mod socket;
pub mod dns;

use alloc::vec::Vec;
use spin::Mutex;

/// Network packet buffer
#[derive(Debug, Clone)]
pub struct PacketBuffer {
    pub data: Vec<u8>,
    pub len: usize,
}

impl PacketBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0u8; capacity],
            len: 0,
        }
    }

    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            len: data.len(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[..self.len]
    }
}

/// Global network statistics
pub static NET_STATS: Mutex<NetworkStats> = Mutex::new(NetworkStats::new());

#[derive(Debug)]
pub struct NetworkStats {
    pub packets_received: u64,
    pub packets_sent: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub errors: u64,
}

impl NetworkStats {
    pub const fn new() -> Self {
        Self {
            packets_received: 0,
            packets_sent: 0,
            bytes_received: 0,
            bytes_sent: 0,
            errors: 0,
        }
    }
}

/// Initialize the network stack
pub fn init() {
    crate::println!("Initializing network stack...");
    // Will initialize E1000 driver here
}
