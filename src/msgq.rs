/// Message Queue module for MyOS
/// Provides System V-style message queues for inter-process communication

use alloc::vec::Vec;
use alloc::collections::{BTreeMap, VecDeque};
use spin::Mutex;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Message queue ID type
pub type MsgQId = u64;

/// Message queue key type (like key_t in Unix)
pub type MsgKey = i32;

/// Special key value for private queues
pub const IPC_PRIVATE: MsgKey = 0;

/// Flags for msgget
pub const IPC_CREAT: i32 = 0o1000;
pub const IPC_EXCL: i32 = 0o2000;

/// Commands for msgctl
pub const IPC_RMID: i32 = 0;
pub const IPC_STAT: i32 = 1;
pub const IPC_SET: i32 = 2;

/// Message queue limits
pub const MSGMAX: usize = 8192;      // Max message size (8KB)
pub const MSGMNB: usize = 16384;     // Max queue size in bytes (16KB)
pub const MSGMNI: usize = 256;       // Max number of message queues

static NEXT_MSGQ_ID: AtomicU64 = AtomicU64::new(1);

/// Message structure
#[derive(Clone)]
pub struct Message {
    /// Message type (must be > 0)
    pub msg_type: i64,
    /// Message data
    pub data: Vec<u8>,
    /// Sender PID
    pub sender_pid: u64,
    /// Send time
    pub send_time: u64,
}

impl Message {
    fn new(msg_type: i64, data: Vec<u8>, sender_pid: u64) -> Result<Self, &'static str> {
        if msg_type <= 0 {
            return Err("Message type must be positive");
        }

        if data.len() > MSGMAX {
            return Err("Message too large");
        }

        Ok(Message {
            msg_type,
            data,
            sender_pid,
            send_time: crate::time::uptime_seconds(),
        })
    }

    /// Get message size (including type field)
    fn size(&self) -> usize {
        core::mem::size_of::<i64>() + self.data.len()
    }
}

/// Message queue permissions
#[derive(Debug, Clone, Copy)]
pub struct MsgPerm {
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

impl MsgPerm {
    fn new() -> Self {
        MsgPerm {
            uid: 0,
            gid: 0,
            cuid: 0,
            cgid: 0,
            mode: 0o666, // rw-rw-rw-
        }
    }
}

/// Message queue statistics
#[derive(Debug, Clone, Copy)]
pub struct MsgStat {
    /// Permissions
    pub perm: MsgPerm,
    /// Number of messages in queue
    pub msg_qnum: usize,
    /// Max number of bytes in queue
    pub msg_qbytes: usize,
    /// PID of last msgsnd
    pub msg_lspid: u64,
    /// PID of last msgrcv
    pub msg_lrpid: u64,
    /// Time of last msgsnd
    pub msg_stime: u64,
    /// Time of last msgrcv
    pub msg_rtime: u64,
    /// Time of last change
    pub msg_ctime: u64,
}

/// Message queue
pub struct MessageQueue {
    id: MsgQId,
    key: MsgKey,
    messages: VecDeque<Message>,
    stat: MsgStat,
    current_bytes: AtomicUsize,
    created_by: u64,
}

impl MessageQueue {
    fn new(key: MsgKey, creator_pid: u64) -> Self {
        let id = NEXT_MSGQ_ID.fetch_add(1, Ordering::Relaxed);
        let now = crate::time::uptime_seconds();

        MessageQueue {
            id,
            key,
            messages: VecDeque::new(),
            stat: MsgStat {
                perm: MsgPerm::new(),
                msg_qnum: 0,
                msg_qbytes: MSGMNB,
                msg_lspid: 0,
                msg_lrpid: 0,
                msg_stime: 0,
                msg_rtime: 0,
                msg_ctime: now,
            },
            current_bytes: AtomicUsize::new(0),
            created_by: creator_pid,
        }
    }

    /// Get message queue ID
    pub fn id(&self) -> MsgQId {
        self.id
    }

    /// Get message queue key
    pub fn key(&self) -> MsgKey {
        self.key
    }

    /// Get statistics
    pub fn stat(&self) -> MsgStat {
        self.stat
    }

    /// Send a message to the queue
    pub fn send(&mut self, msg: Message) -> Result<(), &'static str> {
        let msg_size = msg.size();

        // Check queue size limit
        let current = self.current_bytes.load(Ordering::Acquire);
        if current + msg_size > self.stat.msg_qbytes {
            return Err("Message queue full");
        }

        // Add message to queue
        self.messages.push_back(msg.clone());
        self.current_bytes.fetch_add(msg_size, Ordering::Release);

        // Update statistics
        self.stat.msg_qnum = self.messages.len();
        self.stat.msg_lspid = msg.sender_pid;
        self.stat.msg_stime = crate::time::uptime_seconds();

        Ok(())
    }

    /// Receive a message from the queue
    /// msg_type: 0 = first message, > 0 = first message of type, < 0 = first message with type <= |msg_type|
    pub fn receive(&mut self, msg_type: i64, receiver_pid: u64) -> Result<Message, &'static str> {
        if self.messages.is_empty() {
            return Err("No messages available");
        }

        let msg_index = if msg_type == 0 {
            // Get first message
            Some(0)
        } else if msg_type > 0 {
            // Get first message of specified type
            self.messages.iter().position(|m| m.msg_type == msg_type)
        } else {
            // Get first message with type <= |msg_type|
            let abs_type = msg_type.abs();
            self.messages.iter().position(|m| m.msg_type <= abs_type)
        };

        if let Some(index) = msg_index {
            let msg = self.messages.remove(index).unwrap();
            let msg_size = msg.size();

            self.current_bytes.fetch_sub(msg_size, Ordering::Release);

            // Update statistics
            self.stat.msg_qnum = self.messages.len();
            self.stat.msg_lrpid = receiver_pid;
            self.stat.msg_rtime = crate::time::uptime_seconds();

            Ok(msg)
        } else {
            Err("No matching message type")
        }
    }

    /// Get number of messages
    pub fn msg_count(&self) -> usize {
        self.messages.len()
    }

    /// Get current bytes used
    pub fn bytes_used(&self) -> usize {
        self.current_bytes.load(Ordering::Acquire)
    }

    /// Get creator PID
    pub fn creator(&self) -> u64 {
        self.created_by
    }
}

/// Global message queue table
pub struct MsgQTable {
    /// Map from message queue ID to queue
    queues: BTreeMap<MsgQId, MessageQueue>,
    /// Map from key to ID for named queues
    key_map: BTreeMap<MsgKey, MsgQId>,
}

impl MsgQTable {
    pub const fn new() -> Self {
        MsgQTable {
            queues: BTreeMap::new(),
            key_map: BTreeMap::new(),
        }
    }

    /// Get or create a message queue
    pub fn msgget(&mut self, key: MsgKey, flags: i32, creator_pid: u64) -> Result<MsgQId, &'static str> {
        // Check if we've reached the limit
        if self.queues.len() >= MSGMNI {
            return Err("Too many message queues");
        }

        // Check if queue with this key already exists
        if key != IPC_PRIVATE {
            if let Some(&existing_id) = self.key_map.get(&key) {
                // Queue exists
                if (flags & IPC_CREAT) != 0 && (flags & IPC_EXCL) != 0 {
                    return Err("Message queue already exists");
                }
                return Ok(existing_id);
            }
        }

        // Create flag must be set to create new queue
        if (flags & IPC_CREAT) == 0 && key != IPC_PRIVATE {
            return Err("Message queue not found");
        }

        // Create new queue
        let queue = MessageQueue::new(key, creator_pid);
        let id = queue.id();

        if key != IPC_PRIVATE {
            self.key_map.insert(key, id);
        }

        self.queues.insert(id, queue);
        Ok(id)
    }

    /// Send a message to a queue
    pub fn msgsnd(
        &mut self,
        id: MsgQId,
        msg_type: i64,
        data: Vec<u8>,
        sender_pid: u64,
    ) -> Result<(), &'static str> {
        let queue = self.queues.get_mut(&id)
            .ok_or("Message queue not found")?;

        let msg = Message::new(msg_type, data, sender_pid)?;
        queue.send(msg)
    }

    /// Receive a message from a queue
    pub fn msgrcv(
        &mut self,
        id: MsgQId,
        msg_type: i64,
        receiver_pid: u64,
    ) -> Result<Message, &'static str> {
        let queue = self.queues.get_mut(&id)
            .ok_or("Message queue not found")?;

        queue.receive(msg_type, receiver_pid)
    }

    /// Control operations on message queue
    pub fn msgctl(&mut self, id: MsgQId, cmd: i32) -> Result<MsgStat, &'static str> {
        match cmd {
            IPC_STAT => {
                let queue = self.queues.get(&id)
                    .ok_or("Message queue not found")?;
                Ok(queue.stat())
            }
            IPC_RMID => {
                let queue = self.queues.remove(&id)
                    .ok_or("Message queue not found")?;

                if queue.key() != IPC_PRIVATE {
                    self.key_map.remove(&queue.key());
                }

                Ok(queue.stat())
            }
            IPC_SET => {
                let queue = self.queues.get(&id)
                    .ok_or("Message queue not found")?;
                Ok(queue.stat())
            }
            _ => Err("Invalid msgctl command"),
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> MsgQStats {
        let total_queues = self.queues.len();
        let total_messages: usize = self.queues.values().map(|q| q.msg_count()).sum();
        let total_bytes: usize = self.queues.values().map(|q| q.bytes_used()).sum();

        MsgQStats {
            total_queues,
            total_messages,
            total_bytes,
        }
    }
}

/// Message queue statistics
#[derive(Debug, Clone, Copy)]
pub struct MsgQStats {
    pub total_queues: usize,
    pub total_messages: usize,
    pub total_bytes: usize,
}

pub static MSGQ_TABLE: Mutex<MsgQTable> = Mutex::new(MsgQTable::new());

// Public API functions

/// Get or create a message queue
pub fn msgget(key: MsgKey, flags: i32) -> Result<MsgQId, &'static str> {
    use crate::process::current_pid;

    let pid = current_pid().ok_or("No current process")?;
    let mut table = MSGQ_TABLE.lock();
    table.msgget(key, flags, pid)
}

/// Send a message to a queue
pub fn msgsnd(id: MsgQId, msg_type: i64, data: Vec<u8>) -> Result<(), &'static str> {
    use crate::process::current_pid;

    let pid = current_pid().ok_or("No current process")?;
    let mut table = MSGQ_TABLE.lock();
    table.msgsnd(id, msg_type, data, pid)
}

/// Receive a message from a queue
pub fn msgrcv(id: MsgQId, msg_type: i64) -> Result<Message, &'static str> {
    use crate::process::current_pid;

    let pid = current_pid().ok_or("No current process")?;
    let mut table = MSGQ_TABLE.lock();
    table.msgrcv(id, msg_type, pid)
}

/// Control a message queue
pub fn msgctl(id: MsgQId, cmd: i32) -> Result<MsgStat, &'static str> {
    let mut table = MSGQ_TABLE.lock();
    table.msgctl(id, cmd)
}

/// Get message queue statistics
pub fn get_stats() -> MsgQStats {
    let table = MSGQ_TABLE.lock();
    table.get_stats()
}
