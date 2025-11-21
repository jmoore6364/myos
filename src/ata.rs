/// ATA/IDE disk driver for MyOS
/// Implements PIO (Programmed I/O) mode for reading and writing disk sectors
///
/// This driver supports:
/// - Primary ATA bus (master and slave drives)
/// - LBA28 addressing (up to 128 GB)
/// - Sector read/write operations
/// - Basic error handling

use x86_64::instructions::port::Port;
use spin::Mutex;
use lazy_static::lazy_static;

/// ATA Primary Bus I/O Ports
const ATA_PRIMARY_DATA: u16 = 0x1F0;
const ATA_PRIMARY_ERROR: u16 = 0x1F1;
const ATA_PRIMARY_FEATURES: u16 = 0x1F1;
const ATA_PRIMARY_SECTOR_COUNT: u16 = 0x1F2;
const ATA_PRIMARY_LBA_LOW: u16 = 0x1F3;
const ATA_PRIMARY_LBA_MID: u16 = 0x1F4;
const ATA_PRIMARY_LBA_HIGH: u16 = 0x1F5;
const ATA_PRIMARY_DRIVE_HEAD: u16 = 0x1F6;
const ATA_PRIMARY_STATUS: u16 = 0x1F7;
const ATA_PRIMARY_COMMAND: u16 = 0x1F7;

/// ATA Status Register Bits
const ATA_STATUS_ERR: u8 = 0x01;  // Error
const ATA_STATUS_DRQ: u8 = 0x08;  // Data Request Ready
const ATA_STATUS_SRV: u8 = 0x10;  // Overlapped Mode Service Request
const ATA_STATUS_DF: u8 = 0x20;   // Drive Fault Error
const ATA_STATUS_RDY: u8 = 0x40;  // Drive Ready
const ATA_STATUS_BSY: u8 = 0x80;  // Busy

/// ATA Commands
const ATA_CMD_READ_PIO: u8 = 0x20;
const ATA_CMD_WRITE_PIO: u8 = 0x30;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

/// Drive selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drive {
    Master = 0xE0,
    Slave = 0xF0,
}

/// ATA error codes
#[derive(Debug)]
pub enum AtaError {
    DriveNotReady,
    DriveFault,
    DataNotReady,
    WriteProtected,
    InvalidLBA,
}

/// ATA disk device
pub struct AtaDisk {
    drive: Drive,
    sectors: u32,
}

lazy_static! {
    pub static ref ATA_MASTER: Mutex<Option<AtaDisk>> = Mutex::new(None);
}

impl AtaDisk {
    /// Create a new ATA disk interface
    pub fn new(drive: Drive) -> Self {
        AtaDisk {
            drive,
            sectors: 0,
        }
    }

    /// Wait for the drive to be ready (BSY flag cleared)
    fn wait_ready(&self) -> Result<(), AtaError> {
        let mut status_port = Port::new(ATA_PRIMARY_STATUS);

        // Wait for BSY to clear (max ~30 seconds, but we'll use a reasonable limit)
        for _ in 0..100000 {
            let status: u8 = unsafe { status_port.read() };

            if status & ATA_STATUS_BSY == 0 {
                return Ok(());
            }
        }

        Err(AtaError::DriveNotReady)
    }

    /// Wait for data to be ready (DRQ flag set)
    fn wait_data_ready(&self) -> Result<(), AtaError> {
        let mut status_port = Port::new(ATA_PRIMARY_STATUS);

        // Wait for DRQ to be set
        for _ in 0..100000 {
            let status: u8 = unsafe { status_port.read() };

            if status & ATA_STATUS_ERR != 0 {
                return Err(AtaError::DriveFault);
            }

            if status & ATA_STATUS_DRQ != 0 {
                return Ok(());
            }
        }

        Err(AtaError::DataNotReady)
    }

    /// Read status register
    fn read_status(&self) -> u8 {
        let mut status_port = Port::new(ATA_PRIMARY_STATUS);
        unsafe { status_port.read() }
    }

    /// Identify drive and get information
    pub fn identify(&mut self) -> Result<(), AtaError> {
        self.wait_ready()?;

        // Select drive
        let mut drive_port = Port::new(ATA_PRIMARY_DRIVE_HEAD);
        unsafe { drive_port.write(self.drive as u8) };

        // Send IDENTIFY command
        let mut cmd_port = Port::new(ATA_PRIMARY_COMMAND);
        unsafe { cmd_port.write(ATA_CMD_IDENTIFY) };

        // Check if drive exists
        let status = self.read_status();
        if status == 0 {
            return Err(AtaError::DriveNotReady);
        }

        self.wait_data_ready()?;

        // Read 256 words (512 bytes) of identification data
        let mut data_port = Port::<u16>::new(ATA_PRIMARY_DATA);
        let mut identify_data = [0u16; 256];

        for i in 0..256 {
            identify_data[i] = unsafe { data_port.read() };
        }

        // Extract sector count (words 60-61 for LBA28)
        self.sectors = (identify_data[61] as u32) << 16 | (identify_data[60] as u32);

        Ok(())
    }

    /// Read a single sector (512 bytes) from the disk
    ///
    /// # Arguments
    /// * `lba` - Logical Block Address (sector number)
    /// * `buffer` - Buffer to read data into (must be at least 512 bytes)
    pub fn read_sector(&self, lba: u32, buffer: &mut [u8]) -> Result<(), AtaError> {
        if buffer.len() < 512 {
            return Err(AtaError::InvalidLBA);
        }

        self.wait_ready()?;

        // Set up LBA and drive
        let mut drive_port = Port::new(ATA_PRIMARY_DRIVE_HEAD);
        let mut sector_count_port = Port::new(ATA_PRIMARY_SECTOR_COUNT);
        let mut lba_low_port = Port::new(ATA_PRIMARY_LBA_LOW);
        let mut lba_mid_port = Port::new(ATA_PRIMARY_LBA_MID);
        let mut lba_high_port = Port::new(ATA_PRIMARY_LBA_HIGH);
        let mut cmd_port = Port::new(ATA_PRIMARY_COMMAND);

        unsafe {
            // Select drive and LBA bits 24-27
            drive_port.write((self.drive as u8) | ((lba >> 24) as u8 & 0x0F));

            // Set sector count (1 sector)
            sector_count_port.write(1u8);

            // Set LBA
            lba_low_port.write((lba & 0xFF) as u8);
            lba_mid_port.write(((lba >> 8) & 0xFF) as u8);
            lba_high_port.write(((lba >> 16) & 0xFF) as u8);

            // Send READ command
            cmd_port.write(ATA_CMD_READ_PIO);
        }

        self.wait_data_ready()?;

        // Read 256 words (512 bytes)
        let mut data_port = Port::<u16>::new(ATA_PRIMARY_DATA);

        for i in (0..512).step_by(2) {
            let word = unsafe { data_port.read() };
            buffer[i] = (word & 0xFF) as u8;
            buffer[i + 1] = ((word >> 8) & 0xFF) as u8;
        }

        Ok(())
    }

    /// Write a single sector (512 bytes) to the disk
    ///
    /// # Arguments
    /// * `lba` - Logical Block Address (sector number)
    /// * `buffer` - Buffer containing data to write (must be exactly 512 bytes)
    pub fn write_sector(&self, lba: u32, buffer: &[u8]) -> Result<(), AtaError> {
        if buffer.len() < 512 {
            return Err(AtaError::InvalidLBA);
        }

        self.wait_ready()?;

        // Set up LBA and drive
        let mut drive_port = Port::new(ATA_PRIMARY_DRIVE_HEAD);
        let mut sector_count_port = Port::new(ATA_PRIMARY_SECTOR_COUNT);
        let mut lba_low_port = Port::new(ATA_PRIMARY_LBA_LOW);
        let mut lba_mid_port = Port::new(ATA_PRIMARY_LBA_MID);
        let mut lba_high_port = Port::new(ATA_PRIMARY_LBA_HIGH);
        let mut cmd_port = Port::new(ATA_PRIMARY_COMMAND);

        unsafe {
            // Select drive and LBA bits 24-27
            drive_port.write((self.drive as u8) | ((lba >> 24) as u8 & 0x0F));

            // Set sector count (1 sector)
            sector_count_port.write(1u8);

            // Set LBA
            lba_low_port.write((lba & 0xFF) as u8);
            lba_mid_port.write(((lba >> 8) & 0xFF) as u8);
            lba_high_port.write(((lba >> 16) & 0xFF) as u8);

            // Send WRITE command
            cmd_port.write(ATA_CMD_WRITE_PIO);
        }

        self.wait_data_ready()?;

        // Write 256 words (512 bytes)
        let mut data_port = Port::<u16>::new(ATA_PRIMARY_DATA);

        for i in (0..512).step_by(2) {
            let word = (buffer[i] as u16) | ((buffer[i + 1] as u16) << 8);
            unsafe { data_port.write(word) };
        }

        // Flush cache (wait for write to complete)
        self.wait_ready()?;

        Ok(())
    }

    /// Get the number of sectors on the disk
    pub fn sector_count(&self) -> u32 {
        self.sectors
    }

    /// Get the disk size in bytes
    pub fn size_bytes(&self) -> u64 {
        self.sectors as u64 * 512
    }

    /// Get the disk size in MB
    pub fn size_mb(&self) -> u64 {
        self.size_bytes() / (1024 * 1024)
    }
}

/// Initialize ATA driver
pub fn init() {
    // Try to initialize master drive
    let mut master = AtaDisk::new(Drive::Master);

    match master.identify() {
        Ok(_) => {
            let size_mb = master.size_mb();
            crate::println!("  ATA Master: {} MB ({} sectors)", size_mb, master.sector_count());
            *ATA_MASTER.lock() = Some(master);
        }
        Err(_) => {
            crate::println!("  ATA Master: Not detected");
        }
    }
}

/// Read a sector from the primary master drive
pub fn read_sector(lba: u32, buffer: &mut [u8]) -> Result<(), AtaError> {
    let disk = ATA_MASTER.lock();
    match disk.as_ref() {
        Some(d) => d.read_sector(lba, buffer),
        None => Err(AtaError::DriveNotReady),
    }
}

/// Write a sector to the primary master drive
pub fn write_sector(lba: u32, buffer: &[u8]) -> Result<(), AtaError> {
    let disk = ATA_MASTER.lock();
    match disk.as_ref() {
        Some(d) => d.write_sector(lba, buffer),
        None => Err(AtaError::DriveNotReady),
    }
}

/// Get disk information
pub fn disk_info() -> Option<(u32, u64)> {
    let disk = ATA_MASTER.lock();
    disk.as_ref().map(|d| (d.sector_count(), d.size_mb()))
}
