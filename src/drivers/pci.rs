// PCI (Peripheral Component Interconnect) Bus
// Used to detect and communicate with hardware devices

use x86_64::instructions::port::Port;

/// PCI Configuration Space Ports
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// PCI configuration address format:
/// Bits 31    : Enable bit
/// Bits 30-24 : Reserved
/// Bits 23-16 : Bus number
/// Bits 15-11 : Device number
/// Bits 10-8  : Function number
/// Bits 7-2   : Register number
/// Bits 1-0   : Always 00
fn pci_config_address(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let bus = bus as u32;
    let device = (device as u32) & 0x1F;
    let function = (function as u32) & 0x07;
    let offset = (offset as u32) & 0xFC;

    0x80000000 | (bus << 16) | (device << 11) | (function << 8) | offset
}

/// Read from PCI configuration space
pub unsafe fn pci_config_read_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = pci_config_address(bus, device, function, offset);

    let mut addr_port = Port::<u32>::new(PCI_CONFIG_ADDRESS);
    let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);

    addr_port.write(address);
    data_port.read()
}

/// Read 16-bit value from PCI config
pub unsafe fn pci_config_read_u16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let val = pci_config_read_u32(bus, device, function, offset & 0xFC);
    ((val >> ((offset & 2) * 8)) & 0xFFFF) as u16
}

/// Write to PCI configuration space
pub unsafe fn pci_config_write_u32(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = pci_config_address(bus, device, function, offset);

    let mut addr_port = Port::<u32>::new(PCI_CONFIG_ADDRESS);
    let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);

    addr_port.write(address);
    data_port.write(value);
}

/// PCI device information
#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub bar0: u32,
    pub bar1: u32,
    pub irq_line: u8,
}

impl PciDevice {
    /// Read PCI device information
    pub unsafe fn read(bus: u8, device: u8, function: u8) -> Option<Self> {
        let vendor_id = pci_config_read_u16(bus, device, function, 0x00);

        // No device present
        if vendor_id == 0xFFFF {
            return None;
        }

        let device_id = pci_config_read_u16(bus, device, function, 0x02);
        let class_subclass = pci_config_read_u32(bus, device, function, 0x08);

        let class_code = ((class_subclass >> 24) & 0xFF) as u8;
        let subclass = ((class_subclass >> 16) & 0xFF) as u8;
        let prog_if = ((class_subclass >> 8) & 0xFF) as u8;

        let bar0 = pci_config_read_u32(bus, device, function, 0x10);
        let bar1 = pci_config_read_u32(bus, device, function, 0x14);
        let irq = pci_config_read_u32(bus, device, function, 0x3C);
        let irq_line = (irq & 0xFF) as u8;

        Some(PciDevice {
            bus,
            device,
            function,
            vendor_id,
            device_id,
            class_code,
            subclass,
            prog_if,
            bar0,
            bar1,
            irq_line,
        })
    }

    /// Enable PCI bus mastering (required for DMA)
    pub unsafe fn enable_bus_mastering(&self) {
        let mut command = pci_config_read_u16(self.bus, self.device, self.function, 0x04);
        command |= 0x04; // Bus Master Enable
        command |= 0x02; // Memory Space Enable

        let offset = 0x04;
        let addr = pci_config_address(self.bus, self.device, self.function, offset);

        let mut addr_port = Port::<u32>::new(PCI_CONFIG_ADDRESS);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);

        addr_port.write(addr);
        data_port.write(command as u32);
    }
}

/// Scan for PCI devices
pub fn scan_pci_devices() -> alloc::vec::Vec<PciDevice> {
    let mut devices = alloc::vec::Vec::new();

    unsafe {
        for bus in 0..=255 {
            for device in 0..32 {
                for function in 0..8 {
                    if let Some(dev) = PciDevice::read(bus, device, function) {
                        devices.push(dev);
                    }
                }
            }
        }
    }

    devices
}

/// Find E1000 network card
/// Intel E1000 Vendor ID: 0x8086, Device IDs: 0x100E, 0x100F, 0x10D3, etc.
pub fn find_e1000() -> Option<PciDevice> {
    let devices = scan_pci_devices();

    for dev in devices {
        // Intel vendor
        if dev.vendor_id == 0x8086 {
            // Check for E1000 device IDs
            match dev.device_id {
                0x100E | 0x100F | 0x10D3 | 0x10EA => {
                    crate::println!("Found E1000: vendor={:04x}, device={:04x}, BAR0={:08x}",
                        dev.vendor_id, dev.device_id, dev.bar0);
                    return Some(dev);
                }
                _ => {}
            }
        }
    }

    None
}
