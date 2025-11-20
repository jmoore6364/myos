/// Pipe IPC module for MyOS
/// Provides Unix-like pipes for inter-process communication

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

static NEXT_PIPE_ID: AtomicU64 = AtomicU64::new(1);

/// Default pipe buffer size (4KB)
pub const PIPE_BUF_SIZE: usize = 4096;

/// Pipe state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeState {
    Open,
    ReadClosed,
    WriteClosed,
    BothClosed,
}

/// Pipe end type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeEnd {
    Read,
    Write,
}

/// Internal pipe structure with circular buffer
pub struct PipeInner {
    id: u64,
    buffer: VecDeque<u8>,
    capacity: usize,
    state: PipeState,
    readers: AtomicUsize,
    writers: AtomicUsize,
}

impl PipeInner {
    fn new(capacity: usize) -> Self {
        PipeInner {
            id: NEXT_PIPE_ID.fetch_add(1, Ordering::Relaxed),
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            state: PipeState::Open,
            readers: AtomicUsize::new(1),
            writers: AtomicUsize::new(1),
        }
    }

    /// Write data to pipe
    fn write(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if self.state == PipeState::ReadClosed || self.state == PipeState::BothClosed {
            return Err("Broken pipe (no readers)");
        }

        let available = self.capacity - self.buffer.len();
        if available == 0 {
            return Ok(0); // Buffer full, would block
        }

        let to_write = core::cmp::min(data.len(), available);
        for byte in &data[..to_write] {
            self.buffer.push_back(*byte);
        }

        Ok(to_write)
    }

    /// Read data from pipe
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if self.buffer.is_empty() {
            if self.state == PipeState::WriteClosed || self.state == PipeState::BothClosed {
                return Ok(0); // EOF
            }
            return Ok(0); // Would block
        }

        let to_read = core::cmp::min(buf.len(), self.buffer.len());
        for i in 0..to_read {
            buf[i] = self.buffer.pop_front().unwrap();
        }

        Ok(to_read)
    }

    /// Check if pipe can be written to
    fn can_write(&self) -> bool {
        self.buffer.len() < self.capacity
    }

    /// Check if pipe has data to read
    fn can_read(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// Get number of bytes available to read
    fn available(&self) -> usize {
        self.buffer.len()
    }

    /// Get pipe ID
    fn id(&self) -> u64 {
        self.id
    }

    /// Close read end
    fn close_read(&mut self) {
        let readers = self.readers.fetch_sub(1, Ordering::Relaxed);
        if readers == 1 {
            match self.state {
                PipeState::Open => self.state = PipeState::ReadClosed,
                PipeState::WriteClosed => self.state = PipeState::BothClosed,
                _ => {}
            }
        }
    }

    /// Close write end
    fn close_write(&mut self) {
        let writers = self.writers.fetch_sub(1, Ordering::Relaxed);
        if writers == 1 {
            match self.state {
                PipeState::Open => self.state = PipeState::WriteClosed,
                PipeState::ReadClosed => self.state = PipeState::BothClosed,
                _ => {}
            }
        }
    }
}

/// Pipe handle (reference-counted)
#[derive(Clone)]
pub struct Pipe {
    inner: Arc<Mutex<PipeInner>>,
    end: PipeEnd,
}

impl Pipe {
    /// Create a new pipe pair (read end, write end)
    pub fn new() -> (Self, Self) {
        let inner = Arc::new(Mutex::new(PipeInner::new(PIPE_BUF_SIZE)));

        let read_end = Pipe {
            inner: Arc::clone(&inner),
            end: PipeEnd::Read,
        };

        let write_end = Pipe {
            inner,
            end: PipeEnd::Write,
        };

        (read_end, write_end)
    }

    /// Write data to pipe (write end only)
    pub fn write(&self, data: &[u8]) -> Result<usize, &'static str> {
        if self.end != PipeEnd::Write {
            return Err("Cannot write to read end");
        }

        let mut inner = self.inner.lock();
        inner.write(data)
    }

    /// Read data from pipe (read end only)
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if self.end != PipeEnd::Read {
            return Err("Cannot read from write end");
        }

        let mut inner = self.inner.lock();
        inner.read(buf)
    }

    /// Check if pipe can be written to
    pub fn can_write(&self) -> bool {
        let inner = self.inner.lock();
        inner.can_write()
    }

    /// Check if pipe has data to read
    pub fn can_read(&self) -> bool {
        let inner = self.inner.lock();
        inner.can_read()
    }

    /// Get number of bytes available
    pub fn available(&self) -> usize {
        let inner = self.inner.lock();
        inner.available()
    }

    /// Get pipe ID
    pub fn id(&self) -> u64 {
        let inner = self.inner.lock();
        inner.id()
    }

    /// Get pipe end type
    pub fn end(&self) -> PipeEnd {
        self.end
    }

    /// Close this pipe end
    pub fn close(&self) {
        let mut inner = self.inner.lock();
        match self.end {
            PipeEnd::Read => inner.close_read(),
            PipeEnd::Write => inner.close_write(),
        }
    }
}

/// Global pipe table
use alloc::collections::BTreeMap;

pub struct PipeTable {
    pipes: BTreeMap<u64, Pipe>,
}

impl PipeTable {
    pub const fn new() -> Self {
        PipeTable {
            pipes: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, pipe: Pipe) -> u64 {
        let id = pipe.id();
        self.pipes.insert(id, pipe);
        id
    }

    pub fn get(&self, id: u64) -> Option<&Pipe> {
        self.pipes.get(&id)
    }

    pub fn remove(&mut self, id: u64) -> Option<Pipe> {
        self.pipes.remove(&id)
    }
}

pub static PIPE_TABLE: Mutex<PipeTable> = Mutex::new(PipeTable::new());

/// Create a new pipe and return (read_fd, write_fd) for the current process
pub fn create_pipe() -> Result<(u32, u32), &'static str> {
    use crate::process::{PROCESS_TABLE, current_pid};

    let pid = current_pid().ok_or("No current process")?;

    // Create pipe
    let (read_pipe, write_pipe) = Pipe::new();

    // Store in global pipe table
    let mut pipe_table = PIPE_TABLE.lock();
    let read_pipe_id = pipe_table.insert(read_pipe.clone());
    let write_pipe_id = pipe_table.insert(write_pipe.clone());
    drop(pipe_table);

    // Add to process file descriptors
    let mut proc_table = PROCESS_TABLE.lock();
    let process = proc_table.get_process_mut(pid).ok_or("Process not found")?;

    let read_fd = process.open_fd(
        alloc::format!("pipe:{}", read_pipe_id),
        0 // Read only
    );

    let write_fd = process.open_fd(
        alloc::format!("pipe:{}", write_pipe_id),
        1 // Write only
    );

    Ok((read_fd, write_fd))
}

/// Read from a pipe file descriptor
pub fn read_pipe(fd: u32, buf: &mut [u8]) -> Result<usize, &'static str> {
    use crate::process::{PROCESS_TABLE, current_pid};

    let pid = current_pid().ok_or("No current process")?;

    // Get file descriptor
    let fd_info = {
        let proc_table = PROCESS_TABLE.lock();
        let process = proc_table.get_process(pid).ok_or("Process not found")?;
        process.get_fd(fd).ok_or("Invalid file descriptor")?.clone()
    };

    // Extract pipe ID from path
    if !fd_info.path.starts_with("pipe:") {
        return Err("Not a pipe");
    }

    let pipe_id = fd_info.path.strip_prefix("pipe:")
        .ok_or("Invalid pipe path")?
        .parse::<u64>()
        .map_err(|_| "Invalid pipe ID")?;

    // Get pipe from table
    let pipe_table = PIPE_TABLE.lock();
    let pipe = pipe_table.get(pipe_id).ok_or("Pipe not found")?;

    pipe.read(buf)
}

/// Write to a pipe file descriptor
pub fn write_pipe(fd: u32, data: &[u8]) -> Result<usize, &'static str> {
    use crate::process::{PROCESS_TABLE, current_pid};

    let pid = current_pid().ok_or("No current process")?;

    // Get file descriptor
    let fd_info = {
        let proc_table = PROCESS_TABLE.lock();
        let process = proc_table.get_process(pid).ok_or("Process not found")?;
        process.get_fd(fd).ok_or("Invalid file descriptor")?.clone()
    };

    // Extract pipe ID from path
    if !fd_info.path.starts_with("pipe:") {
        return Err("Not a pipe");
    }

    let pipe_id = fd_info.path.strip_prefix("pipe:")
        .ok_or("Invalid pipe path")?
        .parse::<u64>()
        .map_err(|_| "Invalid pipe ID")?;

    // Get pipe from table
    let pipe_table = PIPE_TABLE.lock();
    let pipe = pipe_table.get(pipe_id).ok_or("Pipe not found")?;

    pipe.write(data)
}

/// Close a pipe file descriptor
pub fn close_pipe(fd: u32) -> Result<(), &'static str> {
    use crate::process::{PROCESS_TABLE, current_pid};

    let pid = current_pid().ok_or("No current process")?;

    // Get and remove file descriptor
    let fd_info = {
        let mut proc_table = PROCESS_TABLE.lock();
        let process = proc_table.get_process_mut(pid).ok_or("Process not found")?;
        process.get_fd(fd).ok_or("Invalid file descriptor")?.clone()
    };

    // Extract pipe ID
    if !fd_info.path.starts_with("pipe:") {
        return Err("Not a pipe");
    }

    let pipe_id = fd_info.path.strip_prefix("pipe:")
        .ok_or("Invalid pipe path")?
        .parse::<u64>()
        .map_err(|_| "Invalid pipe ID")?;

    // Close pipe end
    {
        let pipe_table = PIPE_TABLE.lock();
        if let Some(pipe) = pipe_table.get(pipe_id) {
            pipe.close();
        }
    }

    // Remove from process file descriptors
    let mut proc_table = PROCESS_TABLE.lock();
    let process = proc_table.get_process_mut(pid).ok_or("Process not found")?;
    process.close_fd(fd)?;

    Ok(())
}

/// Get pipe statistics
pub fn get_stats() -> (usize, usize) {
    let pipe_table = PIPE_TABLE.lock();
    let total_pipes = pipe_table.pipes.len();
    let active_bytes: usize = pipe_table.pipes.values()
        .map(|p| p.available())
        .sum();

    (total_pipes, active_bytes)
}
