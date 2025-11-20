/// Process management module for MyOS
/// Provides process abstraction, lifecycle management, and IPC

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

static NEXT_PID: AtomicU64 = AtomicU64::new(1); // PID 0 reserved for kernel

/// Process ID type
pub type Pid = u64;

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is ready to run
    Ready,
    /// Process is currently running
    Running,
    /// Process is waiting for I/O or event
    Waiting,
    /// Process is sleeping
    Sleeping,
    /// Process has terminated
    Zombie,
    /// Process has been fully cleaned up
    Dead,
}

/// Process priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Realtime = 4,
}

/// Process control block (PCB)
pub struct Process {
    /// Process ID
    pid: Pid,
    /// Parent process ID
    parent_pid: Option<Pid>,
    /// Process name
    name: String,
    /// Process state
    state: ProcessState,
    /// Process priority
    priority: Priority,
    /// Exit code (when zombie)
    exit_code: Option<i32>,
    /// Child process IDs
    children: Vec<Pid>,
    /// Current working directory
    cwd: String,
    /// Open file descriptors
    file_descriptors: BTreeMap<u32, FileDescriptor>,
    /// Next available file descriptor number
    next_fd: u32,
    /// Process memory usage (bytes)
    memory_usage: usize,
    /// CPU time used (ticks)
    cpu_time: u64,
    /// Signal disposition
    signal_disposition: crate::signal::SignalDisposition,
}

/// File descriptor (simplified for now)
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub path: String,
    pub flags: u32,
    pub offset: usize,
}

impl Process {
    /// Create a new process
    pub fn new(name: String, parent_pid: Option<Pid>) -> Self {
        let pid = NEXT_PID.fetch_add(1, Ordering::Relaxed);

        Process {
            pid,
            parent_pid,
            name,
            state: ProcessState::Ready,
            priority: Priority::Normal,
            exit_code: None,
            children: Vec::new(),
            cwd: String::from("/"),
            file_descriptors: BTreeMap::new(),
            next_fd: 3, // 0=stdin, 1=stdout, 2=stderr
            memory_usage: 0,
            cpu_time: 0,
            signal_disposition: crate::signal::SignalDisposition::new(),
        }
    }

    /// Get process ID
    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// Get parent process ID
    pub fn parent_pid(&self) -> Option<Pid> {
        self.parent_pid
    }

    /// Get process name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get process state
    pub fn state(&self) -> ProcessState {
        self.state
    }

    /// Set process state
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }

    /// Get process priority
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// Set process priority
    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }

    /// Get exit code (if zombie)
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Mark process as terminated
    pub fn terminate(&mut self, exit_code: i32) {
        self.state = ProcessState::Zombie;
        self.exit_code = Some(exit_code);
    }

    /// Add a child process
    pub fn add_child(&mut self, child_pid: Pid) {
        self.children.push(child_pid);
    }

    /// Remove a child process
    pub fn remove_child(&mut self, child_pid: Pid) {
        self.children.retain(|&pid| pid != child_pid);
    }

    /// Get children PIDs
    pub fn children(&self) -> &[Pid] {
        &self.children
    }

    /// Get current working directory
    pub fn cwd(&self) -> &str {
        &self.cwd
    }

    /// Set current working directory
    pub fn set_cwd(&mut self, cwd: String) {
        self.cwd = cwd;
    }

    /// Open a file descriptor
    pub fn open_fd(&mut self, path: String, flags: u32) -> u32 {
        let fd_num = self.next_fd;
        self.next_fd += 1;

        self.file_descriptors.insert(
            fd_num,
            FileDescriptor {
                path,
                flags,
                offset: 0,
            },
        );

        fd_num
    }

    /// Close a file descriptor
    pub fn close_fd(&mut self, fd: u32) -> Result<(), &'static str> {
        self.file_descriptors
            .remove(&fd)
            .map(|_| ())
            .ok_or("Invalid file descriptor")
    }

    /// Get file descriptor
    pub fn get_fd(&self, fd: u32) -> Option<&FileDescriptor> {
        self.file_descriptors.get(&fd)
    }

    /// Get mutable file descriptor
    pub fn get_fd_mut(&mut self, fd: u32) -> Option<&mut FileDescriptor> {
        self.file_descriptors.get_mut(&fd)
    }

    /// Get memory usage
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }

    /// Set memory usage
    pub fn set_memory_usage(&mut self, usage: usize) {
        self.memory_usage = usage;
    }

    /// Get CPU time
    pub fn cpu_time(&self) -> u64 {
        self.cpu_time
    }

    /// Increment CPU time
    pub fn add_cpu_time(&mut self, ticks: u64) {
        self.cpu_time += ticks;
    }

    /// Get signal disposition
    pub fn signal_disposition(&self) -> Option<&crate::signal::SignalDisposition> {
        Some(&self.signal_disposition)
    }

    /// Get mutable signal disposition
    pub fn signal_disposition_mut(&mut self) -> Option<&mut crate::signal::SignalDisposition> {
        Some(&mut self.signal_disposition)
    }
}

/// Process table - global registry of all processes
pub struct ProcessTable {
    processes: BTreeMap<Pid, Process>,
    current_pid: Option<Pid>,
}

impl ProcessTable {
    /// Create a new process table
    pub const fn new() -> Self {
        ProcessTable {
            processes: BTreeMap::new(),
            current_pid: None,
        }
    }

    /// Add a process to the table
    pub fn add_process(&mut self, process: Process) -> Pid {
        let pid = process.pid();
        self.processes.insert(pid, process);
        pid
    }

    /// Remove a process from the table
    pub fn remove_process(&mut self, pid: Pid) -> Result<Process, &'static str> {
        self.processes.remove(&pid).ok_or("Process not found")
    }

    /// Get a process by PID
    pub fn get_process(&self, pid: Pid) -> Option<&Process> {
        self.processes.get(&pid)
    }

    /// Get a mutable process by PID
    pub fn get_process_mut(&mut self, pid: Pid) -> Option<&mut Process> {
        self.processes.get_mut(&pid)
    }

    /// Get current process PID
    pub fn current_pid(&self) -> Option<Pid> {
        self.current_pid
    }

    /// Set current process
    pub fn set_current(&mut self, pid: Option<Pid>) {
        self.current_pid = pid;
    }

    /// Get current process
    pub fn current_process(&self) -> Option<&Process> {
        self.current_pid.and_then(|pid| self.get_process(pid))
    }

    /// Get current process (mutable)
    pub fn current_process_mut(&mut self) -> Option<&mut Process> {
        self.current_pid.and_then(|pid| self.get_process_mut(pid))
    }

    /// List all processes
    pub fn list_processes(&self) -> Vec<&Process> {
        self.processes.values().collect()
    }

    /// Count processes by state
    pub fn count_by_state(&self, state: ProcessState) -> usize {
        self.processes
            .values()
            .filter(|p| p.state() == state)
            .count()
    }

    /// Get total process count
    pub fn count(&self) -> usize {
        self.processes.len()
    }

    /// Find processes by name
    pub fn find_by_name(&self, name: &str) -> Vec<&Process> {
        self.processes
            .values()
            .filter(|p| p.name() == name)
            .collect()
    }

    /// Kill a process and all its children
    pub fn kill_process_tree(&mut self, pid: Pid, signal: i32) -> Result<(), &'static str> {
        // Get children before modifying
        let children = {
            let process = self.get_process(pid).ok_or("Process not found")?;
            process.children().to_vec()
        };

        // Kill all children first
        for child_pid in children {
            let _ = self.kill_process_tree(child_pid, signal);
        }

        // Kill this process
        let process = self.get_process_mut(pid).ok_or("Process not found")?;
        process.terminate(signal);

        Ok(())
    }

    /// Reap zombie processes (cleanup after terminated children)
    pub fn reap_zombies(&mut self, parent_pid: Pid) -> Vec<(Pid, i32)> {
        let mut reaped = Vec::new();

        let children = {
            let parent = match self.get_process(parent_pid) {
                Some(p) => p,
                None => return reaped,
            };
            parent.children().to_vec()
        };

        for child_pid in children {
            if let Some(child) = self.get_process(child_pid) {
                if child.state() == ProcessState::Zombie {
                    if let Some(exit_code) = child.exit_code() {
                        reaped.push((child_pid, exit_code));
                    }
                }
            }
        }

        // Remove reaped processes
        for (pid, _) in &reaped {
            let _ = self.remove_process(*pid);
        }

        reaped
    }
}

/// Global process table
pub static PROCESS_TABLE: Mutex<ProcessTable> = Mutex::new(ProcessTable::new());

/// Initialize the process subsystem
pub fn init() {
    let mut table = PROCESS_TABLE.lock();

    // Create the init process (PID 1)
    let init_process = Process::new(String::from("init"), None);
    table.add_process(init_process);
    table.set_current(Some(1));
}

/// Create a new process
pub fn create_process(name: String, parent_pid: Option<Pid>) -> Result<Pid, &'static str> {
    let mut table = PROCESS_TABLE.lock();

    let mut process = Process::new(name, parent_pid);

    // Add to parent's children list
    if let Some(ppid) = parent_pid {
        let parent = table.get_process_mut(ppid).ok_or("Parent not found")?;
        parent.add_child(process.pid());
    }

    let pid = table.add_process(process);
    Ok(pid)
}

/// Terminate a process
pub fn exit_process(pid: Pid, exit_code: i32) -> Result<(), &'static str> {
    let mut table = PROCESS_TABLE.lock();
    let process = table.get_process_mut(pid).ok_or("Process not found")?;
    process.terminate(exit_code);
    Ok(())
}

/// Wait for a child process to terminate
pub fn wait_for_child(parent_pid: Pid) -> Option<(Pid, i32)> {
    let mut table = PROCESS_TABLE.lock();
    let reaped = table.reap_zombies(parent_pid);
    reaped.into_iter().next()
}

/// Get current process ID
pub fn current_pid() -> Option<Pid> {
    PROCESS_TABLE.lock().current_pid()
}

/// Kill a process
pub fn kill(pid: Pid, signal: i32) -> Result<(), &'static str> {
    let mut table = PROCESS_TABLE.lock();
    table.kill_process_tree(pid, signal)
}
