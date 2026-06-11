// TCP - Transmission Control Protocol (Layer 4)
// Connection-oriented, reliable transport protocol
// This is the most complex protocol in the network stack!

use super::ipv4::Ipv4Packet;

/// TCP header (minimum 20 bytes, up to 60 with options)
///
/// Format:
/// - Source Port (2 bytes)
/// - Destination Port (2 bytes)
/// - Sequence Number (4 bytes)
/// - Acknowledgment Number (4 bytes)
/// - Data Offset (4 bits) + Reserved (3 bits) + Flags (9 bits)
/// - Window Size (2 bytes)
/// - Checksum (2 bytes)
/// - Urgent Pointer (2 bytes)
/// - Options (0-40 bytes)
#[derive(Debug, Clone)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dest_port: u16,
    pub seq_number: u32,
    pub ack_number: u32,
    pub data_offset: u8,  // Header length in 32-bit words
    pub flags: TcpFlags,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
}

/// TCP flags
#[derive(Debug, Clone, Copy)]
pub struct TcpFlags {
    pub fin: bool, // Finish
    pub syn: bool, // Synchronize
    pub rst: bool, // Reset
    pub psh: bool, // Push
    pub ack: bool, // Acknowledgment
    pub urg: bool, // Urgent
    pub ece: bool, // ECN Echo
    pub cwr: bool, // Congestion Window Reduced
    pub ns: bool,  // Nonce Sum
}

impl TcpFlags {
    pub fn new() -> Self {
        TcpFlags {
            fin: false,
            syn: false,
            rst: false,
            psh: false,
            ack: false,
            urg: false,
            ece: false,
            cwr: false,
            ns: false,
        }
    }

    pub fn from_u16(value: u16) -> Self {
        TcpFlags {
            fin: (value & 0x001) != 0,
            syn: (value & 0x002) != 0,
            rst: (value & 0x004) != 0,
            psh: (value & 0x008) != 0,
            ack: (value & 0x010) != 0,
            urg: (value & 0x020) != 0,
            ece: (value & 0x040) != 0,
            cwr: (value & 0x080) != 0,
            ns: (value & 0x100) != 0,
        }
    }

    pub fn to_u16(&self) -> u16 {
        let mut flags = 0u16;
        if self.fin { flags |= 0x001; }
        if self.syn { flags |= 0x002; }
        if self.rst { flags |= 0x004; }
        if self.psh { flags |= 0x008; }
        if self.ack { flags |= 0x010; }
        if self.urg { flags |= 0x020; }
        if self.ece { flags |= 0x040; }
        if self.cwr { flags |= 0x080; }
        if self.ns { flags |= 0x100; }
        flags
    }
}

/// TCP connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

/// Handle incoming TCP segment
pub fn handle_tcp(_ip_packet: &Ipv4Packet) {
    // TODO: Implement TCP state machine
    crate::println!("TCP: Segment received (handler not yet implemented)");
    crate::println!("TCP: Full implementation includes:");
    crate::println!("  - 3-way handshake (SYN, SYN-ACK, ACK)");
    crate::println!("  - Reliable delivery with retransmission");
    crate::println!("  - Flow control (sliding window)");
    crate::println!("  - Congestion control (slow start, congestion avoidance)");
    crate::println!("  - Connection termination (FIN, FIN-ACK)");
    crate::println!("  This is ~1500+ lines of complex state machine code!");
}
