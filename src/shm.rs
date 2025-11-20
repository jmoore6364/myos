/// Shared Memory module for MyOS
/// Provides System V-style shared memory for inter-process communication

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Shared memory key type (like key_t in Unix)
pub type ShmKey = i32;

/// Shared memory ID type
pub type ShmId = u64;

/// Special key value for private memory
pub const IPC_PRIVATE: ShmKey = 0;

/// Flags for shmget
pub const IPC_CREAT: i32 = 0o1000;
pub const IPC_EXCL: i32 = 0o2000;

/// Commands for shmctl
pub const IPC_RMID: i32 = 0;
pub const IPC_STAT: i32 = 1;
pub const IPC_SET: i32 = 2;

/// Shared memory flags for shmat
pub const SHM_RDONLY: i32 = 0o10000;
pub const SHM_RND: i32 = 0o20000;

static NEXT_SHM_ID: AtomicU64 = AtomicU64::new(1);

/// Shared memory segment permissions
#[derive(Debug, Clone, Copy)]
pub struct ShmPerm {
    /// Owner user ID
    pub uid: u32,
    /// Owner group ID
    pub gid: u32,
    /// Creator user ID
    pub cuid: u32,
    /// Creator group ID
    pub cgid: u32,
    /// Permission mode
    pub mode: u16,
}

impl ShmPerm {
    fn new() -> Self {
        ShmPerm {
            uid: 0,
            gid: 0,
            cuid: 0,
            cgid: 0,
            mode: 0o666, // rw-rw-rw-
        }
    }
}

/// Shared memory segment metadata
#[derive(Debug, Clone, Copy)]
pub struct ShmStat {
    /// Permissions
    pub perm: ShmPerm,
    /// Size of segment in bytes
    pub size: usize,
    /// Time of last shmat
    pub atime: u64,
    /// Time of last shmdt
    pub dtime: u64,
    /// Time of last change
    pub ctime: u64,
    /// PID of creator
    pub cpid: u64,
    /// PID of last shmat/shmdt
    pub lpid: u64,
    /// Number of current attaches
    pub nattch: usize,
}

/// Shared memory segment
pub struct SharedMemorySegment {
    id: ShmId,
    key: ShmKey,
    data: Vec<u8>,
    stat: ShmStat,
    attach_count: AtomicUsize,
    marked_for_removal: bool,
}

impl SharedMemorySegment {
    fn new(key: ShmKey, size: usize, creator_pid: u64) -> Self {
        let id = NEXT_SHM_ID.fetch_add(1, Ordering::Relaxed);
        let now = crate::time::uptime_seconds();

        SharedMemorySegment {
            id,
            key,
            data: alloc::vec![0u8; size],
            stat: ShmStat {
                perm: ShmPerm::new(),
                size,
                atime: 0,
                dtime: 0,
                ctime: now,
                cpid: creator_pid,
                lpid: creator_pid,
                nattch: 0,
            },
            attach_count: AtomicUsize::new(0),
            marked_for_removal: false,
        }
    }

    /// Get shared memory ID
    pub fn id(&self) -> ShmId {
        self.id
    }

    /// Get shared memory key
    pub fn key(&self) -> ShmKey {
        self.key
    }

    /// Get size
    pub fn size(&self) -> usize {
        self.stat.size
    }

    /// Get statistics
    pub fn stat(&self) -> ShmStat {
        self.stat
    }

    /// Get data pointer
    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Get mutable data pointer
    pub fn data_ptr_mut(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Attach to this segment
    pub fn attach(&mut self, pid: u64) {
        self.attach_count.fetch_add(1, Ordering::Relaxed);
        self.stat.nattch = self.attach_count.load(Ordering::Relaxed);
        self.stat.atime = crate::time::uptime_seconds();
        self.stat.lpid = pid;
    }

    /// Detach from this segment
    pub fn detach(&mut self, pid: u64) -> usize {
        let count = self.attach_count.fetch_sub(1, Ordering::Relaxed);
        self.stat.nattch = if count > 0 { count - 1 } else { 0 };
        self.stat.dtime = crate::time::uptime_seconds();
        self.stat.lpid = pid;
        self.stat.nattch
    }

    /// Mark for removal (will be deleted when attach count reaches 0)
    pub fn mark_for_removal(&mut self) {
        self.marked_for_removal = true;
    }

    /// Check if marked for removal
    pub fn is_marked_for_removal(&self) -> bool {
        self.marked_for_removal
    }

    /// Check if can be deleted (marked and no attachments)
    pub fn can_delete(&self) -> bool {
        self.marked_for_removal && self.stat.nattch == 0
    }
}

/// Global shared memory table
pub struct ShmTable {
    /// Map from shared memory ID to segment
    segments: BTreeMap<ShmId, SharedMemorySegment>,
    /// Map from key to ID for named segments
    key_map: BTreeMap<ShmKey, ShmId>,
}

impl ShmTable {
    pub const fn new() -> Self {
        ShmTable {
            segments: BTreeMap::new(),
            key_map: BTreeMap::new(),
        }
    }

    /// Create or get a shared memory segment
    pub fn shmget(&mut self, key: ShmKey, size: usize, flags: i32, creator_pid: u64) -> Result<ShmId, &'static str> {
        // Check if segment with this key already exists
        if key != IPC_PRIVATE {
            if let Some(&existing_id) = self.key_map.get(&key) {
                // Segment exists
                if (flags & IPC_CREAT) != 0 && (flags & IPC_EXCL) != 0 {
                    return Err("Shared memory segment already exists");
                }

                // Return existing segment
                return Ok(existing_id);
            }
        }

        // Create flag must be set to create new segment
        if (flags & IPC_CREAT) == 0 && key != IPC_PRIVATE {
            return Err("Shared memory segment not found");
        }

        // Create new segment
        let segment = SharedMemorySegment::new(key, size, creator_pid);
        let id = segment.id();

        if key != IPC_PRIVATE {
            self.key_map.insert(key, id);
        }

        self.segments.insert(id, segment);
        Ok(id)
    }

    /// Attach to a shared memory segment
    pub fn shmat(&mut self, id: ShmId, pid: u64) -> Result<*const u8, &'static str> {
        let segment = self.segments.get_mut(&id)
            .ok_or("Shared memory segment not found")?;

        segment.attach(pid);
        Ok(segment.data_ptr())
    }

    /// Detach from a shared memory segment
    pub fn shmdt(&mut self, id: ShmId, pid: u64) -> Result<(), &'static str> {
        let should_delete = {
            let segment = self.segments.get_mut(&id)
                .ok_or("Shared memory segment not found")?;

            let _remaining = segment.detach(pid);
            segment.can_delete()
        };

        // Delete if marked for removal and no attachments
        if should_delete {
            self.remove_segment(id);
        }

        Ok(())
    }

    /// Control operations on shared memory segment
    pub fn shmctl(&mut self, id: ShmId, cmd: i32) -> Result<ShmStat, &'static str> {
        let segment = self.segments.get_mut(&id)
            .ok_or("Shared memory segment not found")?;

        match cmd {
            IPC_STAT => {
                // Return statistics
                Ok(segment.stat())
            }
            IPC_RMID => {
                // Mark for removal
                segment.mark_for_removal();

                // Delete immediately if no attachments
                let should_delete = segment.can_delete();
                let stat = segment.stat();

                if should_delete {
                    self.remove_segment(id);
                }

                Ok(stat)
            }
            IPC_SET => {
                // Set permissions (not fully implemented)
                Ok(segment.stat())
            }
            _ => Err("Invalid shmctl command"),
        }
    }

    /// Remove a segment from the table
    fn remove_segment(&mut self, id: ShmId) {
        if let Some(segment) = self.segments.remove(&id) {
            if segment.key() != IPC_PRIVATE {
                self.key_map.remove(&segment.key());
            }
        }
    }

    /// Get statistics for a segment
    pub fn get_stat(&self, id: ShmId) -> Option<ShmStat> {
        self.segments.get(&id).map(|s| s.stat())
    }

    /// Get total number of segments
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Get total shared memory size
    pub fn total_size(&self) -> usize {
        self.segments.values().map(|s| s.size()).sum()
    }
}

pub static SHM_TABLE: Mutex<ShmTable> = Mutex::new(ShmTable::new());

/// Process shared memory attachment tracking
pub struct ShmAttachment {
    pub shm_id: ShmId,
    pub addr: usize,
    pub flags: i32,
}

/// Get or create shared memory segment
pub fn shmget(key: ShmKey, size: usize, flags: i32) -> Result<ShmId, &'static str> {
    use crate::process::current_pid;

    let pid = current_pid().ok_or("No current process")?;
    let mut table = SHM_TABLE.lock();
    table.shmget(key, size, flags, pid)
}

/// Attach to shared memory segment
pub fn shmat(id: ShmId, _flags: i32) -> Result<usize, &'static str> {
    use crate::process::{PROCESS_TABLE, current_pid};

    let pid = current_pid().ok_or("No current process")?;

    // Get pointer from shared memory table
    let ptr = {
        let mut table = SHM_TABLE.lock();
        table.shmat(id, pid)? as usize
    };

    // Track attachment in process
    let mut proc_table = PROCESS_TABLE.lock();
    let _process = proc_table.get_process_mut(pid)
        .ok_or("Process not found")?;

    // Store attachment info in process (future: track in process struct)
    // For now, just return the address

    Ok(ptr)
}

/// Detach from shared memory segment
pub fn shmdt(addr: usize) -> Result<(), &'static str> {
    use crate::process::current_pid;

    let pid = current_pid().ok_or("No current process")?;

    // Find segment by address
    // For simplicity, we'll detach based on the address matching the segment's data pointer
    let id = {
        let table = SHM_TABLE.lock();
        let mut found_id = None;

        for (seg_id, segment) in &table.segments {
            if segment.data_ptr() as usize == addr {
                found_id = Some(*seg_id);
                break;
            }
        }

        found_id.ok_or("Shared memory segment not found at address")?
    };

    let mut table = SHM_TABLE.lock();
    table.shmdt(id, pid)
}

/// Control shared memory segment
pub fn shmctl(id: ShmId, cmd: i32) -> Result<ShmStat, &'static str> {
    let mut table = SHM_TABLE.lock();
    table.shmctl(id, cmd)
}

/// Get shared memory statistics
pub fn get_stats() -> (usize, usize) {
    let table = SHM_TABLE.lock();
    (table.segment_count(), table.total_size())
}
