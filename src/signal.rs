/// Signal handling module for MyOS
/// Provides Unix-like signal mechanism for inter-process communication

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

/// Signal numbers (Unix-compatible)
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Signal {
    /// Hangup detected on controlling terminal or death of controlling process
    SIGHUP = 1,
    /// Interrupt from keyboard (Ctrl+C)
    SIGINT = 2,
    /// Quit from keyboard (Ctrl+\)
    SIGQUIT = 3,
    /// Illegal instruction
    SIGILL = 4,
    /// Trace/breakpoint trap
    SIGTRAP = 5,
    /// Abort signal
    SIGABRT = 6,
    /// Bus error
    SIGBUS = 7,
    /// Floating point exception
    SIGFPE = 8,
    /// Kill signal (cannot be caught or ignored)
    SIGKILL = 9,
    /// User-defined signal 1
    SIGUSR1 = 10,
    /// Invalid memory reference
    SIGSEGV = 11,
    /// User-defined signal 2
    SIGUSR2 = 12,
    /// Broken pipe
    SIGPIPE = 13,
    /// Timer signal
    SIGALRM = 14,
    /// Termination signal
    SIGTERM = 15,
    /// Child stopped or terminated
    SIGCHLD = 17,
    /// Continue if stopped
    SIGCONT = 18,
    /// Stop process
    SIGSTOP = 19,
    /// Stop typed at terminal
    SIGTSTP = 20,
}

impl Signal {
    /// Convert from integer signal number
    pub fn from_u64(n: u64) -> Option<Self> {
        match n {
            1 => Some(Signal::SIGHUP),
            2 => Some(Signal::SIGINT),
            3 => Some(Signal::SIGQUIT),
            4 => Some(Signal::SIGILL),
            5 => Some(Signal::SIGTRAP),
            6 => Some(Signal::SIGABRT),
            7 => Some(Signal::SIGBUS),
            8 => Some(Signal::SIGFPE),
            9 => Some(Signal::SIGKILL),
            10 => Some(Signal::SIGUSR1),
            11 => Some(Signal::SIGSEGV),
            12 => Some(Signal::SIGUSR2),
            13 => Some(Signal::SIGPIPE),
            14 => Some(Signal::SIGALRM),
            15 => Some(Signal::SIGTERM),
            17 => Some(Signal::SIGCHLD),
            18 => Some(Signal::SIGCONT),
            19 => Some(Signal::SIGSTOP),
            20 => Some(Signal::SIGTSTP),
            _ => None,
        }
    }

    /// Check if signal can be caught or ignored
    pub fn is_catchable(self) -> bool {
        !matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }

    /// Get default action for this signal
    pub fn default_action(self) -> SignalAction {
        match self {
            Signal::SIGCHLD => SignalAction::Ignore,
            Signal::SIGCONT => SignalAction::Continue,
            Signal::SIGSTOP | Signal::SIGTSTP => SignalAction::Stop,
            _ => SignalAction::Terminate,
        }
    }

    /// Get signal name
    pub fn name(self) -> &'static str {
        match self {
            Signal::SIGHUP => "SIGHUP",
            Signal::SIGINT => "SIGINT",
            Signal::SIGQUIT => "SIGQUIT",
            Signal::SIGILL => "SIGILL",
            Signal::SIGTRAP => "SIGTRAP",
            Signal::SIGABRT => "SIGABRT",
            Signal::SIGBUS => "SIGBUS",
            Signal::SIGFPE => "SIGFPE",
            Signal::SIGKILL => "SIGKILL",
            Signal::SIGUSR1 => "SIGUSR1",
            Signal::SIGSEGV => "SIGSEGV",
            Signal::SIGUSR2 => "SIGUSR2",
            Signal::SIGPIPE => "SIGPIPE",
            Signal::SIGALRM => "SIGALRM",
            Signal::SIGTERM => "SIGTERM",
            Signal::SIGCHLD => "SIGCHLD",
            Signal::SIGCONT => "SIGCONT",
            Signal::SIGSTOP => "SIGSTOP",
            Signal::SIGTSTP => "SIGTSTP",
        }
    }
}

/// Signal action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalAction {
    /// Ignore the signal
    Ignore,
    /// Terminate the process
    Terminate,
    /// Stop the process
    Stop,
    /// Continue stopped process
    Continue,
    /// Call custom handler
    Handler(u64), // Handler function address
}

/// Signal disposition (how a process handles signals)
#[derive(Clone)]
pub struct SignalDisposition {
    /// Map of signal to action
    handlers: BTreeMap<u64, SignalAction>,
    /// Signal mask (blocked signals)
    blocked: u64,
    /// Pending signals
    pending: u64,
}

impl SignalDisposition {
    /// Create new signal disposition with default handlers
    pub fn new() -> Self {
        SignalDisposition {
            handlers: BTreeMap::new(),
            blocked: 0,
            pending: 0,
        }
    }

    /// Set handler for a signal
    pub fn set_handler(&mut self, signal: Signal, action: SignalAction) -> Result<(), &'static str> {
        if !signal.is_catchable() {
            return Err("Signal cannot be caught");
        }

        self.handlers.insert(signal as u64, action);
        Ok(())
    }

    /// Get action for a signal
    pub fn get_action(&self, signal: Signal) -> SignalAction {
        self.handlers
            .get(&(signal as u64))
            .copied()
            .unwrap_or_else(|| signal.default_action())
    }

    /// Block a signal
    pub fn block(&mut self, signal: Signal) {
        self.blocked |= 1 << (signal as u64);
    }

    /// Unblock a signal
    pub fn unblock(&mut self, signal: Signal) {
        self.blocked &= !(1 << (signal as u64));
    }

    /// Check if signal is blocked
    pub fn is_blocked(&self, signal: Signal) -> bool {
        (self.blocked & (1 << (signal as u64))) != 0
    }

    /// Add pending signal
    pub fn add_pending(&mut self, signal: Signal) {
        self.pending |= 1 << (signal as u64);
    }

    /// Remove pending signal
    pub fn remove_pending(&mut self, signal: Signal) {
        self.pending &= !(1 << (signal as u64));
    }

    /// Get next pending unblocked signal
    pub fn next_pending(&self) -> Option<Signal> {
        let unblocked_pending = self.pending & !self.blocked;

        if unblocked_pending == 0 {
            return None;
        }

        // Find first set bit
        for i in 1..=20 {
            if (unblocked_pending & (1 << i)) != 0 {
                return Signal::from_u64(i);
            }
        }

        None
    }

    /// Check if there are pending signals
    pub fn has_pending(&self) -> bool {
        (self.pending & !self.blocked) != 0
    }
}

/// Global signal statistics
static SIGNALS_SENT: AtomicU64 = AtomicU64::new(0);
static SIGNALS_DELIVERED: AtomicU64 = AtomicU64::new(0);

/// Send a signal to a process
pub fn send_signal(target_pid: u64, signal: Signal) -> Result<(), &'static str> {
    use crate::process::PROCESS_TABLE;

    SIGNALS_SENT.fetch_add(1, Ordering::Relaxed);

    let mut table = PROCESS_TABLE.lock();
    let process = table.get_process_mut(target_pid)
        .ok_or("Process not found")?;

    // Get signal disposition
    if let Some(disposition) = process.signal_disposition_mut() {
        // SIGKILL and SIGSTOP cannot be blocked
        if signal == Signal::SIGKILL || signal == Signal::SIGSTOP {
            // Immediately terminate or stop the process
            match signal.default_action() {
                SignalAction::Terminate => {
                    process.terminate(128 + signal as i32);
                }
                SignalAction::Stop => {
                    process.set_state(crate::process::ProcessState::Waiting);
                }
                _ => {}
            }
            SIGNALS_DELIVERED.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }

        // Add to pending signals
        disposition.add_pending(signal);
    }

    Ok(())
}

/// Deliver pending signals to a process
pub fn deliver_signals(pid: u64) -> Result<(), &'static str> {
    use crate::process::PROCESS_TABLE;

    let mut table = PROCESS_TABLE.lock();
    let process = table.get_process_mut(pid)
        .ok_or("Process not found")?;

    // Get next pending signal
    let signal = {
        let disposition = process.signal_disposition_mut()
            .ok_or("No signal disposition")?;

        match disposition.next_pending() {
            Some(sig) => {
                disposition.remove_pending(sig);
                sig
            }
            None => return Ok(()),
        }
    };

    // Get action for signal
    let action = {
        let disposition = process.signal_disposition()
            .ok_or("No signal disposition")?;
        disposition.get_action(signal)
    };

    SIGNALS_DELIVERED.fetch_add(1, Ordering::Relaxed);

    // Perform action
    match action {
        SignalAction::Ignore => {
            // Do nothing
        }
        SignalAction::Terminate => {
            process.terminate(128 + signal as i32);
        }
        SignalAction::Stop => {
            process.set_state(crate::process::ProcessState::Waiting);
        }
        SignalAction::Continue => {
            if process.state() == crate::process::ProcessState::Waiting {
                process.set_state(crate::process::ProcessState::Ready);
            }
        }
        SignalAction::Handler(_addr) => {
            // TODO: Jump to user-space signal handler
            // This requires setting up the task's context to call the handler
            crate::println!("Custom signal handlers not yet implemented");
        }
    }

    Ok(())
}

/// Get signal statistics
pub fn get_stats() -> (u64, u64) {
    (
        SIGNALS_SENT.load(Ordering::Relaxed),
        SIGNALS_DELIVERED.load(Ordering::Relaxed),
    )
}
