/// SimpleFS - A simple filesystem for MyOS
///
/// This is a custom, simple filesystem designed for learning and demonstration.
/// It provides basic file and directory operations with persistent storage on disk.
///
/// Layout:
/// - Sector 0: Superblock (filesystem metadata)
/// - Sector 1-10: Inode bitmap (which inodes are allocated)
/// - Sector 11-20: Data block bitmap (which data blocks are allocated)
/// - Sector 21-520: Inode table (500 inodes, 1 inode per sector)
/// - Sector 521+: Data blocks (file/directory content)

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use lazy_static::lazy_static;

const SECTOR_SIZE: usize = 512;
const SUPERBLOCK_SECTOR: u32 = 0;
const INODE_BITMAP_START: u32 = 1;
const INODE_BITMAP_SECTORS: u32 = 10;
const DATA_BITMAP_START: u32 = 11;
const DATA_BITMAP_SECTORS: u32 = 10;
const INODE_TABLE_START: u32 = 21;
const MAX_INODES: u32 = 500;
const DATA_BLOCKS_START: u32 = 521;

const MAX_FILENAME: usize = 28;
const DIRECT_BLOCKS: usize = 10;

/// Filesystem superblock
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Superblock {
    magic: u32,              // Magic number to identify filesystem
    total_inodes: u32,       // Total number of inodes
    total_blocks: u32,       // Total number of data blocks
    free_inodes: u32,        // Number of free inodes
    free_blocks: u32,        // Number of free data blocks
    block_size: u32,         // Size of each block (512 bytes)
    inode_table_start: u32,  // First sector of inode table
    data_blocks_start: u32,  // First sector of data blocks
}

impl Superblock {
    fn new() -> Self {
        Superblock {
            magic: 0x53494D46, // "SIMF" in hex
            total_inodes: MAX_INODES,
            total_blocks: 1000, // Support 1000 data blocks = ~500KB
            free_inodes: MAX_INODES - 1, // Root inode is allocated
            free_blocks: 1000,
            block_size: SECTOR_SIZE as u32,
            inode_table_start: INODE_TABLE_START,
            data_blocks_start: DATA_BLOCKS_START,
        }
    }

    fn to_bytes(&self) -> [u8; SECTOR_SIZE] {
        let mut buffer = [0u8; SECTOR_SIZE];
        buffer[0..4].copy_from_slice(&self.magic.to_le_bytes());
        buffer[4..8].copy_from_slice(&self.total_inodes.to_le_bytes());
        buffer[8..12].copy_from_slice(&self.total_blocks.to_le_bytes());
        buffer[12..16].copy_from_slice(&self.free_inodes.to_le_bytes());
        buffer[16..20].copy_from_slice(&self.free_blocks.to_le_bytes());
        buffer[20..24].copy_from_slice(&self.block_size.to_le_bytes());
        buffer[24..28].copy_from_slice(&self.inode_table_start.to_le_bytes());
        buffer[28..32].copy_from_slice(&self.data_blocks_start.to_le_bytes());
        buffer
    }

    fn from_bytes(buffer: &[u8]) -> Option<Self> {
        if buffer.len() < 32 {
            return None;
        }

        let magic = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        if magic != 0x53494D46 {
            return None; // Invalid magic number
        }

        Some(Superblock {
            magic,
            total_inodes: u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            total_blocks: u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
            free_inodes: u32::from_le_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]),
            free_blocks: u32::from_le_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]),
            block_size: u32::from_le_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]),
            inode_table_start: u32::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]),
            data_blocks_start: u32::from_le_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]),
        })
    }
}

/// Inode types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    Free = 0,
    File = 1,
    Directory = 2,
}

impl From<u8> for InodeType {
    fn from(value: u8) -> Self {
        match value {
            1 => InodeType::File,
            2 => InodeType::Directory,
            _ => InodeType::Free,
        }
    }
}

/// Inode structure (512 bytes, 1 per sector)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Inode {
    inode_type: u8,                      // File type
    _padding: [u8; 3],                   // Padding for alignment
    size: u32,                           // File size in bytes
    created: u64,                        // Creation timestamp
    modified: u64,                       // Modification timestamp
    direct_blocks: [u32; DIRECT_BLOCKS], // Direct block pointers
    name: [u8; MAX_FILENAME],            // Filename
}

impl Inode {
    fn new(inode_type: InodeType, name: &str) -> Self {
        let mut name_bytes = [0u8; MAX_FILENAME];
        let name_slice = name.as_bytes();
        let copy_len = core::cmp::min(name_slice.len(), MAX_FILENAME);
        name_bytes[..copy_len].copy_from_slice(&name_slice[..copy_len]);

        Inode {
            inode_type: inode_type as u8,
            _padding: [0; 3],
            size: 0,
            created: crate::time::uptime_seconds(),
            modified: crate::time::uptime_seconds(),
            direct_blocks: [0; DIRECT_BLOCKS],
            name: name_bytes,
        }
    }

    fn to_bytes(&self) -> [u8; SECTOR_SIZE] {
        let mut buffer = [0u8; SECTOR_SIZE];
        buffer[0] = self.inode_type;
        buffer[4..8].copy_from_slice(&self.size.to_le_bytes());
        buffer[8..16].copy_from_slice(&self.created.to_le_bytes());
        buffer[16..24].copy_from_slice(&self.modified.to_le_bytes());

        for (i, &block) in self.direct_blocks.iter().enumerate() {
            let offset = 24 + i * 4;
            buffer[offset..offset + 4].copy_from_slice(&block.to_le_bytes());
        }

        buffer[64..64 + MAX_FILENAME].copy_from_slice(&self.name);
        buffer
    }

    fn from_bytes(buffer: &[u8]) -> Self {
        let mut inode = Inode {
            inode_type: buffer[0],
            _padding: [0; 3],
            size: u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            created: u64::from_le_bytes([
                buffer[8], buffer[9], buffer[10], buffer[11],
                buffer[12], buffer[13], buffer[14], buffer[15],
            ]),
            modified: u64::from_le_bytes([
                buffer[16], buffer[17], buffer[18], buffer[19],
                buffer[20], buffer[21], buffer[22], buffer[23],
            ]),
            direct_blocks: [0; DIRECT_BLOCKS],
            name: [0; MAX_FILENAME],
        };

        for i in 0..DIRECT_BLOCKS {
            let offset = 24 + i * 4;
            inode.direct_blocks[i] = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
        }

        inode.name.copy_from_slice(&buffer[64..64 + MAX_FILENAME]);
        inode
    }

    fn get_name(&self) -> String {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(MAX_FILENAME);
        String::from_utf8_lossy(&self.name[..end]).into_owned()
    }
}

/// Directory entry (32 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DirEntry {
    inode: u32,               // Inode number
    name: [u8; MAX_FILENAME], // Filename
}

impl DirEntry {
    fn new(inode: u32, name: &str) -> Self {
        let mut name_bytes = [0u8; MAX_FILENAME];
        let name_slice = name.as_bytes();
        let copy_len = core::cmp::min(name_slice.len(), MAX_FILENAME);
        name_bytes[..copy_len].copy_from_slice(&name_slice[..copy_len]);

        DirEntry {
            inode,
            name: name_bytes,
        }
    }

    fn get_name(&self) -> String {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(MAX_FILENAME);
        String::from_utf8_lossy(&self.name[..end]).into_owned()
    }

    fn to_bytes(&self) -> [u8; 32] {
        let mut buffer = [0u8; 32];
        buffer[0..4].copy_from_slice(&self.inode.to_le_bytes());
        buffer[4..32].copy_from_slice(&self.name);
        buffer
    }

    fn from_bytes(buffer: &[u8]) -> Self {
        let mut entry = DirEntry {
            inode: u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            name: [0; MAX_FILENAME],
        };
        entry.name.copy_from_slice(&buffer[4..32]);
        entry
    }
}

/// SimpleFS filesystem
pub struct SimpleFS {
    superblock: Superblock,
    mounted: bool,
}

lazy_static! {
    pub static ref SIMPLEFS: Mutex<SimpleFS> = Mutex::new(SimpleFS::new());
}

impl SimpleFS {
    fn new() -> Self {
        SimpleFS {
            superblock: Superblock::new(),
            mounted: false,
        }
    }

    /// Format the disk with SimpleFS
    pub fn format(&mut self) -> Result<(), &'static str> {
        crate::println!("Formatting disk with SimpleFS...");

        // Write superblock
        let sb_bytes = self.superblock.to_bytes();
        crate::ata::write_sector(SUPERBLOCK_SECTOR, &sb_bytes)
            .map_err(|_| "Failed to write superblock")?;

        // Initialize inode bitmap (all free except inode 0 for root)
        let mut bitmap = [0u8; SECTOR_SIZE];
        bitmap[0] = 0x01; // Mark inode 0 as used
        for sector in INODE_BITMAP_START..(INODE_BITMAP_START + INODE_BITMAP_SECTORS) {
            crate::ata::write_sector(sector, &bitmap)
                .map_err(|_| "Failed to write inode bitmap")?;
            bitmap[0] = 0x00; // Rest are all free
        }

        // Initialize data block bitmap (all free)
        let bitmap = [0u8; SECTOR_SIZE];
        for sector in DATA_BITMAP_START..(DATA_BITMAP_START + DATA_BITMAP_SECTORS) {
            crate::ata::write_sector(sector, &bitmap)
                .map_err(|_| "Failed to write data bitmap")?;
        }

        // Create root directory inode (inode 0)
        let root_inode = Inode::new(InodeType::Directory, "/");
        let root_bytes = root_inode.to_bytes();
        crate::ata::write_sector(INODE_TABLE_START, &root_bytes)
            .map_err(|_| "Failed to write root inode")?;

        // Clear remaining inodes
        let empty_inode = Inode::new(InodeType::Free, "");
        let empty_bytes = empty_inode.to_bytes();
        for inode_num in 1..MAX_INODES {
            let sector = INODE_TABLE_START + inode_num;
            crate::ata::write_sector(sector, &empty_bytes)
                .map_err(|_| "Failed to clear inodes")?;
        }

        crate::println!("Filesystem formatted successfully");
        Ok(())
    }

    /// Mount the filesystem
    pub fn mount(&mut self) -> Result<(), &'static str> {
        let mut buffer = [0u8; SECTOR_SIZE];
        crate::ata::read_sector(SUPERBLOCK_SECTOR, &mut buffer)
            .map_err(|_| "Failed to read superblock")?;

        self.superblock = Superblock::from_bytes(&buffer)
            .ok_or("Invalid superblock - filesystem may not be formatted")?;

        self.mounted = true;
        crate::println!("SimpleFS mounted successfully");
        crate::println!("  Total inodes: {}", self.superblock.total_inodes);
        crate::println!("  Free inodes: {}", self.superblock.free_inodes);
        crate::println!("  Total blocks: {}", self.superblock.total_blocks);
        crate::println!("  Free blocks: {}", self.superblock.free_blocks);

        Ok(())
    }

    /// Check if filesystem is mounted
    pub fn is_mounted(&self) -> bool {
        self.mounted
    }
}

/// Initialize the filesystem
pub fn init() {
    // Try to mount existing filesystem
    let mut fs = SIMPLEFS.lock();
    match fs.mount() {
        Ok(_) => {
            crate::println!("  SimpleFS: Mounted");
        }
        Err(_) => {
            crate::println!("  SimpleFS: Not formatted (use 'fsformat' to format)");
        }
    }
}

/// Format the filesystem
pub fn format() -> Result<(), &'static str> {
    let mut fs = SIMPLEFS.lock();
    fs.format()?;
    fs.mount()
}

/// Get filesystem info
pub fn get_info() -> Option<(u32, u32, u32, u32)> {
    let fs = SIMPLEFS.lock();
    if fs.is_mounted() {
        Some((
            fs.superblock.total_inodes,
            fs.superblock.free_inodes,
            fs.superblock.total_blocks,
            fs.superblock.free_blocks,
        ))
    } else {
        None
    }
}
