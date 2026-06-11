// Ethernet (Layer 2) - Data Link Layer
// Handles MAC addresses and Ethernet frames

use super::PacketBuffer;

/// MAC address (6 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddr(pub [u8; 6]);

impl MacAddr {
    pub const BROADCAST: MacAddr = MacAddr([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    pub const ZERO: MacAddr = MacAddr([0, 0, 0, 0, 0, 0]);

    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        MacAddr([a, b, c, d, e, f])
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 6 {
            return None;
        }
        let mut mac = [0u8; 6];
        mac.copy_from_slice(&bytes[0..6]);
        Some(MacAddr(mac))
    }

    pub fn is_broadcast(&self) -> bool {
        *self == Self::BROADCAST
    }

    pub fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 != 0
    }
}

impl core::fmt::Display for MacAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// EtherType - protocol identifier in Ethernet frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum EtherType {
    Ipv4 = 0x0800,
    Arp = 0x0806,
    Ipv6 = 0x86DD,
    Unknown = 0xFFFF,
}

impl From<u16> for EtherType {
    fn from(value: u16) -> Self {
        match value {
            0x0800 => EtherType::Ipv4,
            0x0806 => EtherType::Arp,
            0x86DD => EtherType::Ipv6,
            _ => EtherType::Unknown,
        }
    }
}

impl From<EtherType> for u16 {
    fn from(val: EtherType) -> u16 {
        val as u16
    }
}

/// Ethernet frame header (14 bytes)
///
/// Format:
/// - Destination MAC (6 bytes)
/// - Source MAC (6 bytes)
/// - EtherType (2 bytes)
#[derive(Debug, Clone)]
pub struct EthernetHeader {
    pub dest: MacAddr,
    pub src: MacAddr,
    pub ethertype: EtherType,
}

impl EthernetHeader {
    pub const SIZE: usize = 14;

    /// Parse Ethernet header from packet
    pub fn parse(packet: &[u8]) -> Option<Self> {
        if packet.len() < Self::SIZE {
            return None;
        }

        let dest = MacAddr::from_bytes(&packet[0..6])?;
        let src = MacAddr::from_bytes(&packet[6..12])?;
        let ethertype = u16::from_be_bytes([packet[12], packet[13]]);

        Some(EthernetHeader {
            dest,
            src,
            ethertype: EtherType::from(ethertype),
        })
    }

    /// Serialize Ethernet header into buffer
    pub fn write_to(&self, buffer: &mut [u8]) -> Option<usize> {
        if buffer.len() < Self::SIZE {
            return None;
        }

        buffer[0..6].copy_from_slice(&self.dest.0);
        buffer[6..12].copy_from_slice(&self.src.0);
        let ethertype_bytes = u16::to_be_bytes(self.ethertype as u16);
        buffer[12..14].copy_from_slice(&ethertype_bytes);

        Some(Self::SIZE)
    }
}

/// Ethernet frame - complete packet
#[derive(Debug)]
pub struct EthernetFrame {
    pub header: EthernetHeader,
    pub payload: PacketBuffer,
}

impl EthernetFrame {
    /// Parse an Ethernet frame from raw bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        let header = EthernetHeader::parse(data)?;
        let payload = PacketBuffer::from_slice(&data[EthernetHeader::SIZE..]);

        Some(EthernetFrame { header, payload })
    }

    /// Create a new Ethernet frame
    pub fn new(dest: MacAddr, src: MacAddr, ethertype: EtherType, payload: PacketBuffer) -> Self {
        EthernetFrame {
            header: EthernetHeader {
                dest,
                src,
                ethertype,
            },
            payload,
        }
    }

    /// Serialize frame to bytes
    pub fn to_bytes(&self) -> PacketBuffer {
        let total_len = EthernetHeader::SIZE + self.payload.len;
        let mut buffer = PacketBuffer::new(total_len);

        self.header.write_to(&mut buffer.data[0..EthernetHeader::SIZE]);
        buffer.data[EthernetHeader::SIZE..total_len].copy_from_slice(self.payload.as_slice());
        buffer.len = total_len;

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_addr_parse() {
        let bytes = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let mac = MacAddr::from_bytes(&bytes).unwrap();
        assert_eq!(mac.0, bytes);
    }

    #[test]
    fn test_ethernet_header() {
        let dest = MacAddr::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55);
        let src = MacAddr::new(0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF);
        let header = EthernetHeader {
            dest,
            src,
            ethertype: EtherType::Ipv4,
        };

        let mut buffer = [0u8; 14];
        header.write_to(&mut buffer);

        let parsed = EthernetHeader::parse(&buffer).unwrap();
        assert_eq!(parsed.dest, dest);
        assert_eq!(parsed.src, src);
        assert_eq!(parsed.ethertype, EtherType::Ipv4);
    }
}
