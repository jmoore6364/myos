// UDP - User Datagram Protocol (Layer 4)
// Connectionless, unreliable transport protocol

use super::ipv4::{IpProtocol, Ipv4Addr, Ipv4Packet};
use super::PacketBuffer;

/// UDP header (8 bytes)
///
/// Format:
/// - Source Port (2 bytes)
/// - Destination Port (2 bytes)
/// - Length (2 bytes) - includes header + data
/// - Checksum (2 bytes)
#[derive(Debug, Clone)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dest_port: u16,
    pub length: u16,
    pub checksum: u16,
}

impl UdpHeader {
    pub const SIZE: usize = 8;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }

        Some(UdpHeader {
            src_port: u16::from_be_bytes([data[0], data[1]]),
            dest_port: u16::from_be_bytes([data[2], data[3]]),
            length: u16::from_be_bytes([data[4], data[5]]),
            checksum: u16::from_be_bytes([data[6], data[7]]),
        })
    }

    pub fn write_to(&self, buffer: &mut [u8]) -> Option<usize> {
        if buffer.len() < Self::SIZE {
            return None;
        }

        buffer[0..2].copy_from_slice(&self.src_port.to_be_bytes());
        buffer[2..4].copy_from_slice(&self.dest_port.to_be_bytes());
        buffer[4..6].copy_from_slice(&self.length.to_be_bytes());
        buffer[6..8].copy_from_slice(&self.checksum.to_be_bytes());

        Some(Self::SIZE)
    }
}

/// UDP datagram
#[derive(Debug)]
pub struct UdpDatagram {
    pub header: UdpHeader,
    pub data: PacketBuffer,
}

impl UdpDatagram {
    pub fn new(src_port: u16, dest_port: u16, data: PacketBuffer) -> Self {
        let length = (UdpHeader::SIZE + data.len) as u16;
        UdpDatagram {
            header: UdpHeader {
                src_port,
                dest_port,
                length,
                checksum: 0, // Optional in IPv4
            },
            data,
        }
    }

    pub fn to_bytes(&self) -> PacketBuffer {
        let total_len = UdpHeader::SIZE + self.data.len;
        let mut buffer = PacketBuffer::new(total_len);

        self.header.write_to(&mut buffer.data[0..UdpHeader::SIZE]);
        buffer.data[UdpHeader::SIZE..total_len].copy_from_slice(self.data.as_slice());
        buffer.len = total_len;

        buffer
    }
}

/// Handle incoming UDP datagram
pub fn handle_udp(_ip_packet: &Ipv4Packet) {
    // TODO: Implement UDP socket handling
    crate::println!("UDP: Packet received (handler not yet implemented)");
}
