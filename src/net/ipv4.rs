// IPv4 - Internet Protocol version 4 (Layer 3)
// Handles IP addressing, routing, and fragmentation

use super::PacketBuffer;

/// IPv4 address (4 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv4Addr(pub [u8; 4]);

impl Ipv4Addr {
    pub const BROADCAST: Ipv4Addr = Ipv4Addr([255, 255, 255, 255]);
    pub const ZERO: Ipv4Addr = Ipv4Addr([0, 0, 0, 0]);
    pub const LOCALHOST: Ipv4Addr = Ipv4Addr([127, 0, 0, 1]);

    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Addr([a, b, c, d])
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 4 {
            return None;
        }
        let mut addr = [0u8; 4];
        addr.copy_from_slice(&bytes[0..4]);
        Some(Ipv4Addr(addr))
    }

    pub fn as_u32(&self) -> u32 {
        u32::from_be_bytes(self.0)
    }

    pub fn from_u32(value: u32) -> Self {
        Ipv4Addr(value.to_be_bytes())
    }

    pub fn is_broadcast(&self) -> bool {
        *self == Self::BROADCAST
    }

    pub fn is_loopback(&self) -> bool {
        self.0[0] == 127
    }

    pub fn is_private(&self) -> bool {
        match self.0[0] {
            10 => true,
            172 => self.0[1] >= 16 && self.0[1] <= 31,
            192 => self.0[1] == 168,
            _ => false,
        }
    }
}

impl core::fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

/// IP Protocol numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IpProtocol {
    Icmp = 1,
    Tcp = 6,
    Udp = 17,
    Unknown = 255,
}

impl From<u8> for IpProtocol {
    fn from(value: u8) -> Self {
        match value {
            1 => IpProtocol::Icmp,
            6 => IpProtocol::Tcp,
            17 => IpProtocol::Udp,
            _ => IpProtocol::Unknown,
        }
    }
}

/// IPv4 packet header
///
/// Format (minimum 20 bytes, up to 60 with options):
/// - Version (4 bits) + IHL (4 bits)
/// - Type of Service (1 byte)
/// - Total Length (2 bytes)
/// - Identification (2 bytes)
/// - Flags (3 bits) + Fragment Offset (13 bits)
/// - TTL (1 byte)
/// - Protocol (1 byte)
/// - Header Checksum (2 bytes)
/// - Source IP (4 bytes)
/// - Destination IP (4 bytes)
/// - Options (0-40 bytes)
#[derive(Debug, Clone)]
pub struct Ipv4Header {
    pub version: u8,        // Should be 4
    pub ihl: u8,           // Header length in 32-bit words (minimum 5)
    pub dscp: u8,          // Differentiated Services Code Point
    pub ecn: u8,           // Explicit Congestion Notification
    pub total_length: u16, // Total packet length including header and data
    pub identification: u16,
    pub flags: u8,         // DF, MF flags
    pub fragment_offset: u16,
    pub ttl: u8,           // Time to live
    pub protocol: IpProtocol,
    pub checksum: u16,
    pub src: Ipv4Addr,
    pub dest: Ipv4Addr,
}

impl Ipv4Header {
    pub const MIN_SIZE: usize = 20;

    /// Parse IPv4 header from packet
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::MIN_SIZE {
            return None;
        }

        let version_ihl = data[0];
        let version = version_ihl >> 4;
        let ihl = version_ihl & 0x0F;

        if version != 4 {
            return None; // Not IPv4
        }

        let tos = data[1];
        let dscp = tos >> 2;
        let ecn = tos & 0x03;

        let total_length = u16::from_be_bytes([data[2], data[3]]);
        let identification = u16::from_be_bytes([data[4], data[5]]);

        let flags_offset = u16::from_be_bytes([data[6], data[7]]);
        let flags = (flags_offset >> 13) as u8;
        let fragment_offset = flags_offset & 0x1FFF;

        let ttl = data[8];
        let protocol = IpProtocol::from(data[9]);
        let checksum = u16::from_be_bytes([data[10], data[11]]);

        let src = Ipv4Addr::from_bytes(&data[12..16])?;
        let dest = Ipv4Addr::from_bytes(&data[16..20])?;

        Some(Ipv4Header {
            version,
            ihl,
            dscp,
            ecn,
            total_length,
            identification,
            flags,
            fragment_offset,
            ttl,
            protocol,
            checksum,
            src,
            dest,
        })
    }

    /// Calculate header checksum
    pub fn calculate_checksum(&self) -> u16 {
        let mut buffer = [0u8; Self::MIN_SIZE];
        self.write_to(&mut buffer, 0); // Write with checksum = 0

        // Internet checksum algorithm
        let mut sum: u32 = 0;
        for i in (0..Self::MIN_SIZE).step_by(2) {
            let word = u16::from_be_bytes([buffer[i], buffer[i + 1]]);
            sum += word as u32;
        }

        // Fold 32-bit sum to 16 bits
        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !sum as u16
    }

    /// Serialize IPv4 header to buffer
    pub fn write_to(&self, buffer: &mut [u8], checksum: u16) -> Option<usize> {
        if buffer.len() < Self::MIN_SIZE {
            return None;
        }

        buffer[0] = (self.version << 4) | self.ihl;
        buffer[1] = (self.dscp << 2) | self.ecn;
        buffer[2..4].copy_from_slice(&self.total_length.to_be_bytes());
        buffer[4..6].copy_from_slice(&self.identification.to_be_bytes());

        let flags_offset = ((self.flags as u16) << 13) | self.fragment_offset;
        buffer[6..8].copy_from_slice(&flags_offset.to_be_bytes());

        buffer[8] = self.ttl;
        buffer[9] = self.protocol as u8;
        buffer[10..12].copy_from_slice(&checksum.to_be_bytes());
        buffer[12..16].copy_from_slice(&self.src.0);
        buffer[16..20].copy_from_slice(&self.dest.0);

        Some(Self::MIN_SIZE)
    }

    /// Get header size in bytes
    pub fn header_len(&self) -> usize {
        (self.ihl as usize) * 4
    }
}

/// IPv4 packet - header + payload
#[derive(Debug)]
pub struct Ipv4Packet {
    pub header: Ipv4Header,
    pub payload: PacketBuffer,
}

impl Ipv4Packet {
    /// Parse IPv4 packet from bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        let header = Ipv4Header::parse(data)?;
        let header_len = header.header_len();

        if data.len() < header_len {
            return None;
        }

        let payload_len = header.total_length as usize - header_len;
        let payload = PacketBuffer::from_slice(&data[header_len..header_len + payload_len]);

        Some(Ipv4Packet { header, payload })
    }

    /// Create new IPv4 packet
    pub fn new(
        src: Ipv4Addr,
        dest: Ipv4Addr,
        protocol: IpProtocol,
        payload: PacketBuffer,
    ) -> Self {
        let total_length = (Ipv4Header::MIN_SIZE + payload.len) as u16;

        let header = Ipv4Header {
            version: 4,
            ihl: 5, // No options, so 5 * 4 = 20 bytes
            dscp: 0,
            ecn: 0,
            total_length,
            identification: 0, // Will be filled by network layer
            flags: 0x02,       // Don't Fragment
            fragment_offset: 0,
            ttl: 64,
            protocol,
            checksum: 0, // Will be calculated
            src,
            dest,
        };

        Ipv4Packet { header, payload }
    }

    /// Serialize packet to bytes
    pub fn to_bytes(&self) -> PacketBuffer {
        let total_len = Ipv4Header::MIN_SIZE + self.payload.len;
        let mut buffer = PacketBuffer::new(total_len);

        // Calculate checksum
        let checksum = self.header.calculate_checksum();

        // Write header
        self.header.write_to(&mut buffer.data[0..Ipv4Header::MIN_SIZE], checksum);

        // Write payload
        buffer.data[Ipv4Header::MIN_SIZE..total_len].copy_from_slice(self.payload.as_slice());
        buffer.len = total_len;

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_addr() {
        let ip = Ipv4Addr::new(192, 168, 1, 1);
        assert_eq!(ip.to_string(), "192.168.1.1");
        assert!(ip.is_private());
        assert!(!ip.is_loopback());

        let localhost = Ipv4Addr::LOCALHOST;
        assert!(localhost.is_loopback());
    }

    #[test]
    fn test_ipv4_packet() {
        let src = Ipv4Addr::new(192, 168, 1, 100);
        let dest = Ipv4Addr::new(192, 168, 1, 1);
        let payload = PacketBuffer::from_slice(b"Hello, World!");

        let packet = Ipv4Packet::new(src, dest, IpProtocol::Tcp, payload);
        assert_eq!(packet.header.version, 4);
        assert_eq!(packet.header.src, src);
        assert_eq!(packet.header.dest, dest);

        let bytes = packet.to_bytes();
        let parsed = Ipv4Packet::parse(bytes.as_slice()).unwrap();
        assert_eq!(parsed.header.src, src);
        assert_eq!(parsed.header.dest, dest);
    }
}
