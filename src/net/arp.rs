// ARP - Address Resolution Protocol
// Maps IP addresses to MAC addresses

use super::ethernet::{EtherType, EthernetFrame, MacAddr};
use super::ipv4::Ipv4Addr;
use super::PacketBuffer;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// ARP cache - maps IP to MAC addresses
pub static ARP_CACHE: Mutex<BTreeMap<Ipv4Addr, MacAddr>> = Mutex::new(BTreeMap::new());

/// ARP operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ArpOp {
    Request = 1,
    Reply = 2,
}

impl From<u16> for ArpOp {
    fn from(value: u16) -> Self {
        match value {
            1 => ArpOp::Request,
            2 => ArpOp::Reply,
            _ => ArpOp::Request, // Default
        }
    }
}

/// ARP packet structure
///
/// Format (28 bytes for IPv4 over Ethernet):
/// - Hardware type (2 bytes) - 1 for Ethernet
/// - Protocol type (2 bytes) - 0x0800 for IPv4
/// - Hardware address length (1 byte) - 6 for MAC
/// - Protocol address length (1 byte) - 4 for IPv4
/// - Operation (2 bytes) - 1=request, 2=reply
/// - Sender hardware address (6 bytes) - sender MAC
/// - Sender protocol address (4 bytes) - sender IP
/// - Target hardware address (6 bytes) - target MAC
/// - Target protocol address (4 bytes) - target IP
#[derive(Debug, Clone)]
pub struct ArpPacket {
    pub hardware_type: u16,
    pub protocol_type: u16,
    pub hardware_len: u8,
    pub protocol_len: u8,
    pub operation: ArpOp,
    pub sender_mac: MacAddr,
    pub sender_ip: Ipv4Addr,
    pub target_mac: MacAddr,
    pub target_ip: Ipv4Addr,
}

impl ArpPacket {
    pub const SIZE: usize = 28;

    /// Parse ARP packet from bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }

        let hardware_type = u16::from_be_bytes([data[0], data[1]]);
        let protocol_type = u16::from_be_bytes([data[2], data[3]]);
        let hardware_len = data[4];
        let protocol_len = data[5];
        let operation = u16::from_be_bytes([data[6], data[7]]);

        let sender_mac = MacAddr::from_bytes(&data[8..14])?;
        let sender_ip = Ipv4Addr::from_bytes(&data[14..18])?;
        let target_mac = MacAddr::from_bytes(&data[18..24])?;
        let target_ip = Ipv4Addr::from_bytes(&data[24..28])?;

        Some(ArpPacket {
            hardware_type,
            protocol_type,
            hardware_len,
            protocol_len,
            operation: ArpOp::from(operation),
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        })
    }

    /// Serialize ARP packet to bytes
    pub fn to_bytes(&self) -> PacketBuffer {
        let mut buffer = PacketBuffer::new(Self::SIZE);

        buffer.data[0..2].copy_from_slice(&self.hardware_type.to_be_bytes());
        buffer.data[2..4].copy_from_slice(&self.protocol_type.to_be_bytes());
        buffer.data[4] = self.hardware_len;
        buffer.data[5] = self.protocol_len;
        buffer.data[6..8].copy_from_slice(&(self.operation as u16).to_be_bytes());
        buffer.data[8..14].copy_from_slice(&self.sender_mac.0);
        buffer.data[14..18].copy_from_slice(&self.sender_ip.0);
        buffer.data[18..24].copy_from_slice(&self.target_mac.0);
        buffer.data[24..28].copy_from_slice(&self.target_ip.0);

        buffer.len = Self::SIZE;
        buffer
    }

    /// Create an ARP request
    pub fn request(sender_mac: MacAddr, sender_ip: Ipv4Addr, target_ip: Ipv4Addr) -> Self {
        ArpPacket {
            hardware_type: 1, // Ethernet
            protocol_type: 0x0800, // IPv4
            hardware_len: 6,
            protocol_len: 4,
            operation: ArpOp::Request,
            sender_mac,
            sender_ip,
            target_mac: MacAddr::ZERO,
            target_ip,
        }
    }

    /// Create an ARP reply
    pub fn reply(
        sender_mac: MacAddr,
        sender_ip: Ipv4Addr,
        target_mac: MacAddr,
        target_ip: Ipv4Addr,
    ) -> Self {
        ArpPacket {
            hardware_type: 1,
            protocol_type: 0x0800,
            hardware_len: 6,
            protocol_len: 4,
            operation: ArpOp::Reply,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        }
    }
}

/// Handle incoming ARP packet
pub fn handle_arp(frame: &EthernetFrame, our_mac: MacAddr, our_ip: Ipv4Addr) -> Option<EthernetFrame> {
    let arp = ArpPacket::parse(frame.payload.as_slice())?;

    // Update ARP cache with sender's info
    {
        let mut cache = ARP_CACHE.lock();
        cache.insert(arp.sender_ip, arp.sender_mac);
    }

    match arp.operation {
        ArpOp::Request => {
            // Is this request for us?
            if arp.target_ip == our_ip {
                // Send ARP reply
                let reply = ArpPacket::reply(our_mac, our_ip, arp.sender_mac, arp.sender_ip);
                let reply_frame = EthernetFrame::new(
                    arp.sender_mac,
                    our_mac,
                    EtherType::Arp,
                    reply.to_bytes(),
                );
                return Some(reply_frame);
            }
        }
        ArpOp::Reply => {
            // Already updated cache above
            crate::println!("ARP: Learned {} -> {}", arp.sender_ip, arp.sender_mac);
        }
    }

    None
}

/// Look up MAC address for IP (returns cached value)
pub fn lookup(ip: Ipv4Addr) -> Option<MacAddr> {
    ARP_CACHE.lock().get(&ip).copied()
}

/// Send ARP request for an IP address
pub fn send_request(sender_mac: MacAddr, sender_ip: Ipv4Addr, target_ip: Ipv4Addr) -> EthernetFrame {
    let arp_request = ArpPacket::request(sender_mac, sender_ip, target_ip);
    EthernetFrame::new(
        MacAddr::BROADCAST,
        sender_mac,
        EtherType::Arp,
        arp_request.to_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arp_request() {
        let sender_mac = MacAddr::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55);
        let sender_ip = Ipv4Addr::new(192, 168, 1, 100);
        let target_ip = Ipv4Addr::new(192, 168, 1, 1);

        let arp = ArpPacket::request(sender_mac, sender_ip, target_ip);
        assert_eq!(arp.operation, ArpOp::Request);
        assert_eq!(arp.sender_mac, sender_mac);
        assert_eq!(arp.sender_ip, sender_ip);
        assert_eq!(arp.target_ip, target_ip);

        let bytes = arp.to_bytes();
        let parsed = ArpPacket::parse(bytes.as_slice()).unwrap();
        assert_eq!(parsed.operation, ArpOp::Request);
    }
}
