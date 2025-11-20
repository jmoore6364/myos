/// Semaphore module for MyOS
/// Provides POSIX-style semaphores for process synchronization

use alloc::string::String;
use alloc::collections::{BTreeMap, VecDeque};
use spin::Mutex;
use core::sync::atomic::{AtomicU64, AtomicI32, Ordering};

/// Semaphore ID type
pub type SemId = u64;

static NEXT_SEM_ID: AtomicU64 = AtomicU64::new(1);

/// Semaphore flags for sem_open
pub const O_CREAT: i32 = 0o100;
pub const O_EXCL: i32 = 0o200;

/// Semaphore value limits
pub const SEM_VALUE_MAX: i32 = i32::MAX;

/// Semaphore structure
pub struct Semaphore {
    id: SemId,
    name: Option<String>,
    value: AtomicI32,
    max_value: i32,
    waiting_queue: Mutex<VecDeque<u64>>, // PIDs waiting on this semaphore
    created_by: u64,
    created_at: u64,
}

impl Semaphore {
    /// Create a new unnamed semaphore
    fn new_unnamed(initial_value: i32, creator_pid: u64) -> Self {
        let id = NEXT_SEM_ID.fetch_add(1, Ordering::Relaxed);

        Semaphore {
            id,
            name: None,
            value: AtomicI32::new(initial_value),
            max_value: SEM_VALUE_MAX,
            waiting_queue: Mutex::new(VecDeque::new()),
            created_by: creator_pid,
            created_at: crate::time::uptime_seconds(),
        }
    }

    /// Create a new named semaphore
    fn new_named(name: String, initial_value: i32, creator_pid: u64) -> Self {
        let id = NEXT_SEM_ID.fetch_add(1, Ordering::Relaxed);

        Semaphore {
            id,
            name: Some(name),
            value: AtomicI32::new(initial_value),
            max_value: SEM_VALUE_MAX,
            waiting_queue: Mutex::new(VecDeque::new()),
            created_by: creator_pid,
            created_at: crate::time::uptime_seconds(),
        }
    }

    /// Get semaphore ID
    pub fn id(&self) -> SemId {
        self.id
    }

    /// Get semaphore name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get current value (snapshot)
    pub fn value(&self) -> i32 {
        self.value.load(Ordering::Acquire)
    }

    /// Wait on semaphore (P operation / decrement)
    /// Returns true if acquired, false if would block
    pub fn wait(&self, pid: u64, blocking: bool) -> bool {
        loop {
            let current = self.value.load(Ordering::Acquire);

            if current > 0 {
                // Try to decrement
                if self.value.compare_exchange(
                    current,
                    current - 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ).is_ok() {
                    return true; // Successfully acquired
                }
                // CAS failed, retry
                continue;
            } else {
                // Semaphore is zero, would block
                if blocking {
                    // Add to waiting queue
                    let mut queue = self.waiting_queue.lock();
                    queue.push_back(pid);
                    drop(queue);
                    // In a real implementation, this would sleep the process
                    // For now, we just return false indicating it would block
                }
                return false;
            }
        }
    }

    /// Post to semaphore (V operation / increment)
    /// Returns Ok(()) on success, Err if would overflow
    pub fn post(&self) -> Result<(), &'static str> {
        loop {
            let current = self.value.load(Ordering::Acquire);

            if current >= self.max_value {
                return Err("Semaphore overflow");
            }

            // Try to increment
            if self.value.compare_exchange(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ).is_ok() {
                // Wake up one waiting process if any
                let mut queue = self.waiting_queue.lock();
                if let Some(_waiting_pid) = queue.pop_front() {
                    // In a real implementation, we would wake up the waiting process
                    // For now, we just remove it from the queue
                }
                return Ok(());
            }
            // CAS failed, retry
        }
    }

    /// Try wait on semaphore (non-blocking)
    pub fn try_wait(&self) -> bool {
        loop {
            let current = self.value.load(Ordering::Acquire);

            if current > 0 {
                // Try to decrement
                if self.value.compare_exchange(
                    current,
                    current - 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ).is_ok() {
                    return true;
                }
                // CAS failed, retry
                continue;
            } else {
                return false;
            }
        }
    }

    /// Get number of processes waiting
    pub fn waiting_count(&self) -> usize {
        self.waiting_queue.lock().len()
    }

    /// Get creator PID
    pub fn creator(&self) -> u64 {
        self.created_by
    }

    /// Get creation time
    pub fn created_at(&self) -> u64 {
        self.created_at
    }
}

/// Global semaphore table
pub struct SemaphoreTable {
    /// Map from semaphore ID to semaphore
    semaphores: BTreeMap<SemId, Semaphore>,
    /// Map from name to ID for named semaphores
    name_map: BTreeMap<String, SemId>,
}

impl SemaphoreTable {
    pub const fn new() -> Self {
        SemaphoreTable {
            semaphores: BTreeMap::new(),
            name_map: BTreeMap::new(),
        }
    }

    /// Create an unnamed semaphore
    pub fn sem_init(&mut self, initial_value: i32, creator_pid: u64) -> Result<SemId, &'static str> {
        if initial_value < 0 || initial_value > SEM_VALUE_MAX {
            return Err("Invalid initial value");
        }

        let sem = Semaphore::new_unnamed(initial_value, creator_pid);
        let id = sem.id();
        self.semaphores.insert(id, sem);
        Ok(id)
    }

    /// Open or create a named semaphore
    pub fn sem_open(
        &mut self,
        name: String,
        flags: i32,
        initial_value: i32,
        creator_pid: u64,
    ) -> Result<SemId, &'static str> {
        // Check if semaphore with this name exists
        if let Some(&existing_id) = self.name_map.get(&name) {
            // Semaphore exists
            if (flags & O_CREAT) != 0 && (flags & O_EXCL) != 0 {
                return Err("Semaphore already exists");
            }
            return Ok(existing_id);
        }

        // Create new named semaphore
        if (flags & O_CREAT) == 0 {
            return Err("Semaphore not found");
        }

        if initial_value < 0 || initial_value > SEM_VALUE_MAX {
            return Err("Invalid initial value");
        }

        let sem = Semaphore::new_named(name.clone(), initial_value, creator_pid);
        let id = sem.id();

        self.name_map.insert(name, id);
        self.semaphores.insert(id, sem);
        Ok(id)
    }

    /// Wait on semaphore
    pub fn sem_wait(&self, id: SemId, pid: u64, blocking: bool) -> Result<bool, &'static str> {
        let sem = self.semaphores.get(&id)
            .ok_or("Semaphore not found")?;
        Ok(sem.wait(pid, blocking))
    }

    /// Try wait on semaphore (non-blocking)
    pub fn sem_trywait(&self, id: SemId) -> Result<bool, &'static str> {
        let sem = self.semaphores.get(&id)
            .ok_or("Semaphore not found")?;
        Ok(sem.try_wait())
    }

    /// Post to semaphore
    pub fn sem_post(&self, id: SemId) -> Result<(), &'static str> {
        let sem = self.semaphores.get(&id)
            .ok_or("Semaphore not found")?;
        sem.post()
    }

    /// Get semaphore value
    pub fn sem_getvalue(&self, id: SemId) -> Result<i32, &'static str> {
        let sem = self.semaphores.get(&id)
            .ok_or("Semaphore not found")?;
        Ok(sem.value())
    }

    /// Destroy a semaphore
    pub fn sem_destroy(&mut self, id: SemId) -> Result<(), &'static str> {
        let sem = self.semaphores.remove(&id)
            .ok_or("Semaphore not found")?;

        // Remove from name map if named
        if let Some(name) = sem.name() {
            self.name_map.remove(name);
        }

        // Check if any processes are waiting
        if sem.waiting_count() > 0 {
            return Err("Processes still waiting on semaphore");
        }

        Ok(())
    }

    /// Unlink a named semaphore
    pub fn sem_unlink(&mut self, name: &str) -> Result<(), &'static str> {
        let id = self.name_map.remove(name)
            .ok_or("Semaphore not found")?;

        // Remove the semaphore itself
        self.semaphores.remove(&id);
        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> SemStats {
        let total_semaphores = self.semaphores.len();
        let named_semaphores = self.name_map.len();
        let total_waiting: usize = self.semaphores.values()
            .map(|s| s.waiting_count())
            .sum();

        SemStats {
            total_semaphores,
            named_semaphores,
            total_waiting,
        }
    }

    /// Get semaphore by name
    pub fn find_by_name(&self, name: &str) -> Option<SemId> {
        self.name_map.get(name).copied()
    }
}

/// Semaphore statistics
#[derive(Debug, Clone, Copy)]
pub struct SemStats {
    pub total_semaphores: usize,
    pub named_semaphores: usize,
    pub total_waiting: usize,
}

pub static SEMAPHORE_TABLE: Mutex<SemaphoreTable> = Mutex::new(SemaphoreTable::new());

// Public API functions

/// Initialize a new unnamed semaphore
pub fn sem_init(initial_value: i32) -> Result<SemId, &'static str> {
    use crate::process::current_pid;

    let pid = current_pid().ok_or("No current process")?;
    let mut table = SEMAPHORE_TABLE.lock();
    table.sem_init(initial_value, pid)
}

/// Open or create a named semaphore
pub fn sem_open(name: String, flags: i32, initial_value: i32) -> Result<SemId, &'static str> {
    use crate::process::current_pid;

    let pid = current_pid().ok_or("No current process")?;
    let mut table = SEMAPHORE_TABLE.lock();
    table.sem_open(name, flags, initial_value, pid)
}

/// Wait on a semaphore (blocking)
pub fn sem_wait(id: SemId) -> Result<bool, &'static str> {
    use crate::process::current_pid;

    let pid = current_pid().ok_or("No current process")?;
    let table = SEMAPHORE_TABLE.lock();
    table.sem_wait(id, pid, true)
}

/// Try wait on a semaphore (non-blocking)
pub fn sem_trywait(id: SemId) -> Result<bool, &'static str> {
    let table = SEMAPHORE_TABLE.lock();
    table.sem_trywait(id)
}

/// Post to a semaphore
pub fn sem_post(id: SemId) -> Result<(), &'static str> {
    let table = SEMAPHORE_TABLE.lock();
    table.sem_post(id)
}

/// Get semaphore value
pub fn sem_getvalue(id: SemId) -> Result<i32, &'static str> {
    let table = SEMAPHORE_TABLE.lock();
    table.sem_getvalue(id)
}

/// Destroy a semaphore
pub fn sem_destroy(id: SemId) -> Result<(), &'static str> {
    let mut table = SEMAPHORE_TABLE.lock();
    table.sem_destroy(id)
}

/// Unlink a named semaphore
pub fn sem_unlink(name: String) -> Result<(), &'static str> {
    let mut table = SEMAPHORE_TABLE.lock();
    table.sem_unlink(&name)
}

/// Get semaphore statistics
pub fn get_stats() -> SemStats {
    let table = SEMAPHORE_TABLE.lock();
    table.get_stats()
}
