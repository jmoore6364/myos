// Intel E1000 Gigabit Ethernet Driver
// Implements packet transmission and reception via DMA

use alloc::vec;
use super::pci::PciDevice;
use crate::net::{PacketBuffer, NET_STATS};
use crate::net::ethernet::{EthernetFrame, MacAddr};
use spin::Mutex;
use core::ptr::{read_volatile, write_volatile};
use alloc::vec::Vec;

/// E1000 Register Offsets (from BAR0)
const E1000_CTRL: u32 = 0x00000;     // Device Control
const E1000_STATUS: u32 = 0x00008;   // Device Status
const E1000_EECD: u32 = 0x00010;     // EEPROM Control
const E1000_EERD: u32 = 0x00014;     // EEPROM Read
const E1000_CTRL_EXT: u32 = 0x00018; // Extended Device Control
const E1000_ICR: u32 = 0x000C0;      // Interrupt Cause Read
const E1000_ICS: u32 = 0x000C8;      // Interrupt Cause Set
const E1000_IMS: u32 = 0x000D0;      // Interrupt Mask Set
const E1000_IMC: u32 = 0x000D8;      // Interrupt Mask Clear
const E1000_RCTL: u32 = 0x00100;     // Receive Control
const E1000_TCTL: u32 = 0x00400;     // Transmit Control
const E1000_TIPG: u32 = 0x00410;     // Transmit Inter Packet Gap
const E1000_RDBAL: u32 = 0x02800;    // RX Descriptor Base Low
const E1000_RDBAH: u32 = 0x02804;    // RX Descriptor Base High
const E1000_RDLEN: u32 = 0x02808;    // RX Descriptor Length
const E1000_RDH: u32 = 0x02810;      // RX Descriptor Head
const E1000_RDT: u32 = 0x02818;      // RX Descriptor Tail
const E1000_TDBAL: u32 = 0x03800;    // TX Descriptor Base Low
const E1000_TDBAH: u32 = 0x03804;    // TX Descriptor Base High
const E1000_TDLEN: u32 = 0x03808;    // TX Descriptor Length
const E1000_TDH: u32 = 0x03810;      // TX Descriptor Head
const E1000_TDT: u32 = 0x03818;      // TX Descriptor Tail
const E1000_MTA: u32 = 0x05200;      // Multicast Table Array
const E1000_RAL: u32 = 0x05400;      // Receive Address Low
const E1000_RAH: u32 = 0x05404;      // Receive Address High

/// Control Register Bits
const CTRL_SLU: u32 = 1 << 6;   // Set Link Up
const CTRL_RST: u32 = 1 << 26;  // Device Reset

/// Receive Control Bits
const RCTL_EN: u32 = 1 << 1;     // Receiver Enable
const RCTL_SBP: u32 = 1 << 2;    // Store Bad Packets
const RCTL_UPE: u32 = 1 << 3;    // Unicast Promiscuous Enable
const RCTL_MPE: u32 = 1 << 4;    // Multicast Promiscuous Enable
const RCTL_BAM: u32 = 1 << 15;   // Broadcast Accept Mode
const RCTL_BSIZE_2048: u32 = 0 << 16; // Buffer Size 2048
const RCTL_SECRC: u32 = 1 << 26; // Strip Ethernet CRC

/// Transmit Control Bits
const TCTL_EN: u32 = 1 << 1;     // Transmitter Enable
const TCTL_PSP: u32 = 1 << 3;    // Pad Short Packets

/// Number of RX/TX descriptors (must be multiple of 8)
const NUM_RX_DESC: usize = 32;
const NUM_TX_DESC: usize = 32;
const RX_BUFFER_SIZE: usize = 2048;
const TX_BUFFER_SIZE: usize = 2048;

/// Receive Descriptor
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct RxDescriptor {
    addr: u64,      // Buffer address
    length: u16,    // Buffer length
    checksum: u16,  // Packet checksum
    status: u8,     // Descriptor status
    errors: u8,     // Errors
    special: u16,   // Special
}

impl RxDescriptor {
    const DD: u8 = 1 << 0;  // Descriptor Done

    fn new() -> Self {
        RxDescriptor {
            addr: 0,
            length: 0,
            checksum: 0,
            status: 0,
            errors: 0,
            special: 0,
        }
    }

    fn is_done(&self) -> bool {
        self.status & Self::DD != 0
    }
}

/// Transmit Descriptor
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct TxDescriptor {
    addr: u64,      // Buffer address
    length: u16,    // Data length
    cso: u8,        // Checksum offset
    cmd: u8,        // Command
    status: u8,     // Status
    css: u8,        // Checksum start
    special: u16,   // Special
}

impl TxDescriptor {
    const CMD_EOP: u8 = 1 << 0;  // End of Packet
    const CMD_RS: u8 = 1 << 3;   // Report Status
    const DD: u8 = 1 << 0;       // Descriptor Done

    fn new() -> Self {
        TxDescriptor {
            addr: 0,
            length: 0,
            cso: 0,
            cmd: 0,
            status: 0,
            css: 0,
            special: 0,
        }
    }

    fn is_done(&self) -> bool {
        self.status & Self::DD != 0
    }
}

/// E1000 Network Interface Card
pub struct E1000 {
    mmio_base: usize,           // Memory-mapped I/O base address
    mac_address: MacAddr,        // MAC address
    rx_descs: Vec<RxDescriptor>, // RX descriptor ring
    tx_descs: Vec<TxDescriptor>, // TX descriptor ring
    rx_buffers: Vec<Vec<u8>>,    // RX buffer pool
    tx_buffers: Vec<Vec<u8>>,    // TX buffer pool
    rx_cur: usize,               // Current RX descriptor
    tx_cur: usize,               // Current TX descriptor
}

impl E1000 {
    /// Read a 32-bit register
    unsafe fn read_reg(&self, offset: u32) -> u32 {
        let addr = (self.mmio_base + offset as usize) as *const u32;
        read_volatile(addr)
    }

    /// Write a 32-bit register
    unsafe fn write_reg(&self, offset: u32, value: u32) {
        let addr = (self.mmio_base + offset as usize) as *mut u32;
        write_volatile(addr, value);
    }

    /// Read MAC address from EEPROM
    unsafe fn read_mac_address(&self) -> MacAddr {
        // Try reading from Receive Address registers first
        let ral = self.read_reg(E1000_RAL);
        let rah = self.read_reg(E1000_RAH);

        if rah & 0x80000000 != 0 {
            // Valid address in registers
            MacAddr([
                (ral & 0xFF) as u8,
                ((ral >> 8) & 0xFF) as u8,
                ((ral >> 16) & 0xFF) as u8,
                ((ral >> 24) & 0xFF) as u8,
                (rah & 0xFF) as u8,
                ((rah >> 8) & 0xFF) as u8,
            ])
        } else {
            // Default MAC for QEMU E1000
            MacAddr([0x52, 0x54, 0x00, 0x12, 0x34, 0x56])
        }
    }

    /// Initialize E1000 device
    pub unsafe fn init(pci_dev: &PciDevice) -> Result<Self, &'static str> {
        crate::println!("E1000: Initializing...");

        // Get MMIO base address from BAR0
        let mmio_base = (pci_dev.bar0 & !0xF) as usize;
        crate::println!("E1000: MMIO base at {:08x}", mmio_base);

        // Enable bus mastering for DMA
        pci_dev.enable_bus_mastering();

        let mut nic = E1000 {
            mmio_base,
            mac_address: MacAddr::ZERO,
            rx_descs: vec![RxDescriptor::new(); NUM_RX_DESC],
            tx_descs: vec![TxDescriptor::new(); NUM_TX_DESC],
            rx_buffers: Vec::new(),
            tx_buffers: Vec::new(),
            rx_cur: 0,
            tx_cur: 0,
        };

        // Read MAC address
        nic.mac_address = nic.read_mac_address();
        crate::println!("E1000: MAC address: {}", nic.mac_address);

        // Reset device
        crate::println!("E1000: Resetting device...");
        nic.write_reg(E1000_CTRL, nic.read_reg(E1000_CTRL) | CTRL_RST);

        // Wait for reset to complete
        for _ in 0..1000 {
            if nic.read_reg(E1000_CTRL) & CTRL_RST == 0 {
                break;
            }
            core::hint::spin_loop();
        }

        // Disable interrupts
        nic.write_reg(E1000_IMC, 0xFFFFFFFF);
        nic.read_reg(E1000_ICR); // Clear pending interrupts

        // Set link up
        nic.write_reg(E1000_CTRL, nic.read_reg(E1000_CTRL) | CTRL_SLU);

        // Initialize RX
        crate::println!("E1000: Initializing RX...");
        nic.init_rx()?;

        // Initialize TX
        crate::println!("E1000: Initializing TX...");
        nic.init_tx()?;

        crate::println!("E1000: Initialization complete!");

        Ok(nic)
    }

    /// Initialize receive ring
    unsafe fn init_rx(&mut self) -> Result<(), &'static str> {
        // Allocate RX buffers
        for _ in 0..NUM_RX_DESC {
            let buffer = vec![0u8; RX_BUFFER_SIZE];
            self.rx_buffers.push(buffer);
        }

        // Setup RX descriptors
        for i in 0..NUM_RX_DESC {
            let buffer_ptr = self.rx_buffers[i].as_ptr() as u64;
            self.rx_descs[i].addr = buffer_ptr;
            self.rx_descs[i].status = 0;
        }

        // Set RX descriptor base address
        let rx_desc_ptr = self.rx_descs.as_ptr() as u64;
        self.write_reg(E1000_RDBAL, (rx_desc_ptr & 0xFFFFFFFF) as u32);
        self.write_reg(E1000_RDBAH, ((rx_desc_ptr >> 32) & 0xFFFFFFFF) as u32);

        // Set RX descriptor ring length
        let rx_desc_len = (NUM_RX_DESC * core::mem::size_of::<RxDescriptor>()) as u32;
        self.write_reg(E1000_RDLEN, rx_desc_len);

        // Set RX head and tail
        self.write_reg(E1000_RDH, 0);
        self.write_reg(E1000_RDT, (NUM_RX_DESC - 1) as u32);

        // Enable receiver
        let rctl = RCTL_EN | RCTL_BAM | RCTL_BSIZE_2048 | RCTL_SECRC;
        self.write_reg(E1000_RCTL, rctl);

        Ok(())
    }

    /// Initialize transmit ring
    unsafe fn init_tx(&mut self) -> Result<(), &'static str> {
        // Allocate TX buffers
        for _ in 0..NUM_TX_DESC {
            let buffer = vec![0u8; TX_BUFFER_SIZE];
            self.tx_buffers.push(buffer);
        }

        // Setup TX descriptors
        for i in 0..NUM_TX_DESC {
            self.tx_descs[i].status = TxDescriptor::DD; // Mark as done initially
        }

        // Set TX descriptor base address
        let tx_desc_ptr = self.tx_descs.as_ptr() as u64;
        self.write_reg(E1000_TDBAL, (tx_desc_ptr & 0xFFFFFFFF) as u32);
        self.write_reg(E1000_TDBAH, ((tx_desc_ptr >> 32) & 0xFFFFFFFF) as u32);

        // Set TX descriptor ring length
        let tx_desc_len = (NUM_TX_DESC * core::mem::size_of::<TxDescriptor>()) as u32;
        self.write_reg(E1000_TDLEN, tx_desc_len);

        // Set TX head and tail
        self.write_reg(E1000_TDH, 0);
        self.write_reg(E1000_TDT, 0);

        // Set transmit IPG
        self.write_reg(E1000_TIPG, 0x0060200A);

        // Enable transmitter
        let tctl = TCTL_EN | TCTL_PSP | (15 << 4) | (64 << 12);
        self.write_reg(E1000_TCTL, tctl);

        Ok(())
    }

    /// Send a packet
    pub unsafe fn send_packet(&mut self, packet: &[u8]) -> Result<(), &'static str> {
        if packet.len() > TX_BUFFER_SIZE {
            return Err("Packet too large");
        }

        let tail = self.tx_cur;
        let desc = &mut self.tx_descs[tail];

        // Wait for descriptor to be available
        let mut retries = 0;
        while !desc.is_done() {
            retries += 1;
            if retries > 10000 {
                return Err("TX timeout");
            }
            core::hint::spin_loop();
        }

        // Copy packet to buffer
        self.tx_buffers[tail][..packet.len()].copy_from_slice(packet);

        // Setup descriptor
        desc.addr = self.tx_buffers[tail].as_ptr() as u64;
        desc.length = packet.len() as u16;
        desc.cmd = TxDescriptor::CMD_EOP | TxDescriptor::CMD_RS;
        desc.status = 0; // Clear done bit

        // Update tail pointer
        self.tx_cur = (self.tx_cur + 1) % NUM_TX_DESC;
        self.write_reg(E1000_TDT, self.tx_cur as u32);

        // Update stats
        let mut stats = NET_STATS.lock();
        stats.packets_sent += 1;
        stats.bytes_sent += packet.len() as u64;

        Ok(())
    }

    /// Receive a packet
    pub unsafe fn receive_packet(&mut self) -> Option<PacketBuffer> {
        let cur = self.rx_cur;
        let desc = &mut self.rx_descs[cur];

        if !desc.is_done() {
            return None; // No packet available
        }

        // Get packet length
        let length = desc.length as usize;

        // Copy packet from buffer
        let mut packet = PacketBuffer::new(length);
        packet.data[..length].copy_from_slice(&self.rx_buffers[cur][..length]);
        packet.len = length;

        // Reset descriptor
        desc.status = 0;

        // Update tail pointer
        let old_tail = (cur + NUM_RX_DESC - 1) % NUM_RX_DESC;
        self.write_reg(E1000_RDT, old_tail as u32);

        // Move to next descriptor
        self.rx_cur = (cur + 1) % NUM_RX_DESC;

        // Update stats
        let mut stats = NET_STATS.lock();
        stats.packets_received += 1;
        stats.bytes_received += length as u64;

        Some(packet)
    }

    /// Get MAC address
    pub fn mac_address(&self) -> MacAddr {
        self.mac_address
    }
}

/// Global E1000 instance
pub static E1000_DRIVER: Mutex<Option<E1000>> = Mutex::new(None);

/// Initialize E1000 network driver
pub fn init() -> Result<(), &'static str> {
    use super::pci;

    crate::println!("Scanning for E1000 network card...");

    let pci_dev = pci::find_e1000().ok_or("E1000 not found")?;

    unsafe {
        let nic = E1000::init(&pci_dev)?;
        *E1000_DRIVER.lock() = Some(nic);
    }

    Ok(())
}

/// Send an Ethernet frame
pub fn send_frame(frame: &EthernetFrame) -> Result<(), &'static str> {
    let packet_bytes = frame.to_bytes();

    unsafe {
        if let Some(ref mut nic) = *E1000_DRIVER.lock() {
            nic.send_packet(packet_bytes.as_slice())?;
        } else {
            return Err("E1000 not initialized");
        }
    }

    Ok(())
}

/// Poll for received packets
pub fn poll_receive() -> Option<EthernetFrame> {
    unsafe {
        if let Some(ref mut nic) = *E1000_DRIVER.lock() {
            if let Some(packet) = nic.receive_packet() {
                return EthernetFrame::parse(packet.as_slice());
            }
        }
    }
    None
}

/// Get local MAC address
pub fn get_mac_address() -> Option<MacAddr> {
    E1000_DRIVER.lock().as_ref().map(|nic| nic.mac_address())
}
