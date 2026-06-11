// ICMP - Internet Control Message Protocol
// Used for ping, traceroute, error messages, etc.

use super::ipv4::{IpProtocol, Ipv4Addr, Ipv4Packet};
use super::PacketBuffer;

/// ICMP message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IcmpType {
    EchoReply = 0,
    DestinationUnreachable = 3,
    SourceQuench = 4,
    Redirect = 5,
    EchoRequest = 8,
    TimeExceeded = 11,
    ParameterProblem = 12,
    TimestampRequest = 13,
    TimestampReply = 14,
    Unknown = 255,
}

impl From<u8> for IcmpType {
    fn from(value: u8) -> Self {
        match value {
            0 => IcmpType::EchoReply,
            3 => IcmpType::DestinationUnreachable,
            4 => IcmpType::SourceQuench,
            5 => IcmpType::Redirect,
            8 => IcmpType::EchoRequest,
            11 => IcmpType::TimeExceeded,
            12 => IcmpType::ParameterProblem,
            13 => IcmpType::TimestampRequest,
            14 => IcmpType::TimestampReply,
            _ => IcmpType::Unknown,
        }
    }
}

/// ICMP header
///
/// Format (8 bytes minimum):
/// - Type (1 byte)
/// - Code (1 byte)
/// - Checksum (2 bytes)
/// - Rest of Header (4 bytes) - depends on type
#[derive(Debug, Clone)]
pub struct IcmpHeader {
    pub msg_type: IcmpType,
    pub code: u8,
    pub checksum: u16,
    pub identifier: u16,  // For echo request/reply
    pub sequence: u16,    // For echo request/reply
}

impl IcmpHeader {
    pub const SIZE: usize = 8;

    /// Parse ICMP header from bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }

        let msg_type = IcmpType::from(data[0]);
        let code = data[1];
        let checksum = u16::from_be_bytes([data[2], data[3]]);
        let identifier = u16::from_be_bytes([data[4], data[5]]);
        let sequence = u16::from_be_bytes([data[6], data[7]]);

        Some(IcmpHeader {
            msg_type,
            code,
            checksum,
            identifier,
            sequence,
        })
    }

    /// Calculate ICMP checksum
    pub fn calculate_checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;

        // Sum all 16-bit words
        while i < data.len() - 1 {
            let word = u16::from_be_bytes([data[i], data[i + 1]]);
            sum += word as u32;
            i += 2;
        }

        // Add remaining byte if odd length
        if i < data.len() {
            sum += (data[i] as u32) << 8;
        }

        // Fold 32-bit sum to 16 bits
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !sum as u16
    }

    /// Serialize ICMP header to bytes
    pub fn write_to(&self, buffer: &mut [u8]) -> Option<usize> {
        if buffer.len() < Self::SIZE {
            return None;
        }

        buffer[0] = self.msg_type as u8;
        buffer[1] = self.code;
        buffer[2..4].copy_from_slice(&self.checksum.to_be_bytes());
        buffer[4..6].copy_from_slice(&self.identifier.to_be_bytes());
        buffer[6..8].copy_from_slice(&self.sequence.to_be_bytes());

        Some(Self::SIZE)
    }
}

/// ICMP packet - header + data
#[derive(Debug)]
pub struct IcmpPacket {
    pub header: IcmpHeader,
    pub data: PacketBuffer,
}

impl IcmpPacket {
    /// Parse ICMP packet from bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        let header = IcmpHeader::parse(data)?;
        let payload = if data.len() > IcmpHeader::SIZE {
            PacketBuffer::from_slice(&data[IcmpHeader::SIZE..])
        } else {
            PacketBuffer::new(0)
        };

        Some(IcmpPacket {
            header,
            data: payload,
        })
    }

    /// Create an ICMP echo request (ping)
    pub fn echo_request(identifier: u16, sequence: u16, data: &[u8]) -> Self {
        IcmpPacket {
            header: IcmpHeader {
                msg_type: IcmpType::EchoRequest,
                code: 0,
                checksum: 0, // Will be calculated
                identifier,
                sequence,
            },
            data: PacketBuffer::from_slice(data),
        }
    }

    /// Create an ICMP echo reply (pong)
    pub fn echo_reply(identifier: u16, sequence: u16, data: &[u8]) -> Self {
        IcmpPacket {
            header: IcmpHeader {
                msg_type: IcmpType::EchoReply,
                code: 0,
                checksum: 0,
                identifier,
                sequence,
            },
            data: PacketBuffer::from_slice(data),
        }
    }

    /// Serialize ICMP packet to bytes (with checksum calculated)
    pub fn to_bytes(&self) -> PacketBuffer {
        let total_len = IcmpHeader::SIZE + self.data.len;
        let mut buffer = PacketBuffer::new(total_len);

        // Write header with checksum = 0
        let mut temp_header = self.header.clone();
        temp_header.checksum = 0;
        temp_header.write_to(&mut buffer.data[0..IcmpHeader::SIZE]);

        // Write data
        buffer.data[IcmpHeader::SIZE..total_len].copy_from_slice(self.data.as_slice());

        // Calculate checksum over entire packet
        let checksum = IcmpHeader::calculate_checksum(&buffer.data[..total_len]);

        // Write checksum
        buffer.data[2..4].copy_from_slice(&checksum.to_be_bytes());
        buffer.len = total_len;

        buffer
    }
}

/// Handle incoming ICMP packet
pub fn handle_icmp(ip_packet: &Ipv4Packet) -> Option<Ipv4Packet> {
    let icmp = IcmpPacket::parse(ip_packet.payload.as_slice())?;

    match icmp.header.msg_type {
        IcmpType::EchoRequest => {
            // Received ping request - send reply
            crate::println!(
                "ICMP: Echo request from {} (id={}, seq={})",
                ip_packet.header.src,
                icmp.header.identifier,
                icmp.header.sequence
            );

            // Create echo reply
            let reply_icmp = IcmpPacket::echo_reply(
                icmp.header.identifier,
                icmp.header.sequence,
                icmp.data.as_slice(),
            );

            // Create IP packet for reply
            let reply_packet = Ipv4Packet::new(
                ip_packet.header.dest,    // Our IP becomes source
                ip_packet.header.src,     // Their IP becomes dest
                IpProtocol::Icmp,
                reply_icmp.to_bytes(),
            );

            Some(reply_packet)
        }
        IcmpType::EchoReply => {
            // Received ping reply
            crate::println!(
                "ICMP: Echo reply from {} (id={}, seq={}, {} bytes)",
                ip_packet.header.src,
                icmp.header.identifier,
                icmp.header.sequence,
                icmp.data.len
            );
            None
        }
        IcmpType::DestinationUnreachable => {
            crate::println!(
                "ICMP: Destination unreachable from {} (code={})",
                ip_packet.header.src,
                icmp.header.code
            );
            None
        }
        IcmpType::TimeExceeded => {
            crate::println!(
                "ICMP: Time exceeded from {} (code={})",
                ip_packet.header.src,
                icmp.header.code
            );
            None
        }
        _ => {
            crate::println!(
                "ICMP: Unknown type {:?} from {}",
                icmp.header.msg_type,
                ip_packet.header.src
            );
            None
        }
    }
}

/// Send a ping to a destination
pub fn send_ping(src_ip: Ipv4Addr, dest_ip: Ipv4Addr, identifier: u16, sequence: u16) -> Ipv4Packet {
    // Create ping data (typically timestamp + padding)
    let mut data = alloc::vec![0u8; 56];
    // Add current uptime as timestamp
    let timestamp = crate::time::uptime_ms();
    data[0..8].copy_from_slice(&timestamp.to_be_bytes());

    // Create ICMP echo request
    let icmp = IcmpPacket::echo_request(identifier, sequence, &data);

    // Wrap in IP packet
    Ipv4Packet::new(src_ip, dest_ip, IpProtocol::Icmp, icmp.to_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icmp_echo_request() {
        let icmp = IcmpPacket::echo_request(1234, 1, b"Hello, Ping!");
        assert_eq!(icmp.header.msg_type, IcmpType::EchoRequest);
        assert_eq!(icmp.header.identifier, 1234);
        assert_eq!(icmp.header.sequence, 1);

        let bytes = icmp.to_bytes();
        let parsed = IcmpPacket::parse(bytes.as_slice()).unwrap();
        assert_eq!(parsed.header.msg_type, IcmpType::EchoRequest);
        assert_eq!(parsed.header.identifier, 1234);
    }

    #[test]
    fn test_checksum() {
        let data = [0x08, 0x00, 0x00, 0x00, 0x12, 0x34, 0x00, 0x01];
        let checksum = IcmpHeader::calculate_checksum(&data);
        assert_ne!(checksum, 0);
    }
}
