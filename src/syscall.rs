/// System call module for MyOS
/// Provides the interface between user tasks and kernel services

use crate::{println, print};
use core::arch::asm;

/// System call numbers
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    Exit = 0,
    Yield = 1,
    Print = 2,
    GetTime = 3,
    GetTicks = 4,
    Sleep = 5,
    GetPid = 6,
    GetPPid = 7,
    Fork = 8,
    Wait = 9,
    Kill = 10,
    Exec = 11,
    Signal = 12,
    SigMask = 13,
    Pipe = 14,
    Read = 15,
    Write = 16,
    Close = 17,
}

impl SyscallNumber {
    pub fn from_u64(n: u64) -> Option<Self> {
        match n {
            0 => Some(SyscallNumber::Exit),
            1 => Some(SyscallNumber::Yield),
            2 => Some(SyscallNumber::Print),
            3 => Some(SyscallNumber::GetTime),
            4 => Some(SyscallNumber::GetTicks),
            5 => Some(SyscallNumber::Sleep),
            6 => Some(SyscallNumber::GetPid),
            7 => Some(SyscallNumber::GetPPid),
            8 => Some(SyscallNumber::Fork),
            9 => Some(SyscallNumber::Wait),
            10 => Some(SyscallNumber::Kill),
            11 => Some(SyscallNumber::Exec),
            12 => Some(SyscallNumber::Signal),
            13 => Some(SyscallNumber::SigMask),
            14 => Some(SyscallNumber::Pipe),
            15 => Some(SyscallNumber::Read),
            16 => Some(SyscallNumber::Write),
            17 => Some(SyscallNumber::Close),
            _ => None,
        }
    }
}

/// System call handler - called from the interrupt handler
///
/// Arguments are passed in registers following the System V AMD64 ABI:
/// - RAX: syscall number
/// - RDI: arg1
/// - RSI: arg2
/// - RDX: arg3
/// - R10: arg4 (RCX is used for return address in syscall instruction)
/// - R8: arg5
/// - R9: arg6
///
/// Return value is in RAX
pub fn syscall_handler(
    syscall_num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    _arg4: u64,
    _arg5: u64,
) -> u64 {
    match SyscallNumber::from_u64(syscall_num) {
        Some(SyscallNumber::Exit) => syscall_exit(arg1),
        Some(SyscallNumber::Yield) => syscall_yield(),
        Some(SyscallNumber::Print) => syscall_print(arg1, arg2),
        Some(SyscallNumber::GetTime) => syscall_get_time(),
        Some(SyscallNumber::GetTicks) => syscall_get_ticks(),
        Some(SyscallNumber::Sleep) => syscall_sleep(arg1),
        Some(SyscallNumber::GetPid) => syscall_getpid(),
        Some(SyscallNumber::GetPPid) => syscall_getppid(),
        Some(SyscallNumber::Fork) => syscall_fork(),
        Some(SyscallNumber::Wait) => syscall_wait(),
        Some(SyscallNumber::Kill) => syscall_kill(arg1, arg2),
        Some(SyscallNumber::Exec) => syscall_exec(arg1, arg2),
        Some(SyscallNumber::Signal) => syscall_signal(arg1, arg2),
        Some(SyscallNumber::SigMask) => syscall_sigmask(arg1, arg2),
        Some(SyscallNumber::Pipe) => syscall_pipe(arg1),
        Some(SyscallNumber::Read) => syscall_read(arg1, arg2, arg3),
        Some(SyscallNumber::Write) => syscall_write(arg1, arg2, arg3),
        Some(SyscallNumber::Close) => syscall_close(arg1),
        None => {
            println!("Unknown syscall: {}", syscall_num);
            u64::MAX // Error code
        }
    }
}

/// Syscall: Exit the current task
fn syscall_exit(exit_code: u64) -> u64 {
    println!("Task exiting with code: {}", exit_code);

    let mut scheduler = crate::SCHEDULER.lock();
    scheduler.terminate_current();

    0
}

/// Syscall: Yield CPU to another task
fn syscall_yield() -> u64 {
    let mut scheduler = crate::SCHEDULER.lock();
    scheduler.yield_current();

    0
}

/// Syscall: Print a string
/// arg1: pointer to string
/// arg2: length of string
fn syscall_print(str_ptr: u64, len: u64) -> u64 {
    if str_ptr == 0 || len == 0 || len > 4096 {
        return u64::MAX; // Invalid arguments
    }

    unsafe {
        let slice = core::slice::from_raw_parts(str_ptr as *const u8, len as usize);
        if let Ok(s) = core::str::from_utf8(slice) {
            print!("{}", s);
            0
        } else {
            u64::MAX // Invalid UTF-8
        }
    }
}

/// Syscall: Get system uptime in seconds
fn syscall_get_time() -> u64 {
    crate::time::uptime_seconds()
}

/// Syscall: Get system ticks (from PIT)
fn syscall_get_ticks() -> u64 {
    crate::pit::get_ticks()
}

/// Syscall: Sleep for specified milliseconds
fn syscall_sleep(ms: u64) -> u64 {
    let start_ticks = crate::pit::get_ticks();
    let sleep_ticks = ms / 10; // 10ms per tick (100 Hz)

    while crate::pit::get_ticks() - start_ticks < sleep_ticks {
        x86_64::instructions::hlt();
    }

    0
}

/// Syscall: Get current process ID
fn syscall_getpid() -> u64 {
    crate::process::current_pid().unwrap_or(0)
}

/// Syscall: Get parent process ID
fn syscall_getppid() -> u64 {
    if let Some(pid) = crate::process::current_pid() {
        let table = crate::process::PROCESS_TABLE.lock();
        if let Some(process) = table.get_process(pid) {
            return process.parent_pid().unwrap_or(0);
        }
    }
    0
}

/// Syscall: Fork current process (simplified - not fully implemented yet)
fn syscall_fork() -> u64 {
    // TODO: Full fork implementation requires copying process memory space
    // For now, return an error
    println!("fork() not yet implemented");
    u64::MAX
}

/// Syscall: Wait for child process
fn syscall_wait() -> u64 {
    if let Some(pid) = crate::process::current_pid() {
        if let Some((child_pid, exit_code)) = crate::process::wait_for_child(pid) {
            // Return child PID in high 32 bits, exit code in low 32 bits
            return ((child_pid as u64) << 32) | ((exit_code as u32) as u64);
        }
    }
    u64::MAX // No children to wait for
}

/// Syscall: Kill a process
/// arg1: target PID
/// arg2: signal number
fn syscall_kill(target_pid: u64, signal_num: u64) -> u64 {
    // Convert signal number to Signal enum
    let signal = match crate::signal::Signal::from_u64(signal_num) {
        Some(s) => s,
        None => return u64::MAX, // Invalid signal
    };

    // Send signal to process
    match crate::signal::send_signal(target_pid, signal) {
        Ok(()) => 0,
        Err(_) => u64::MAX,
    }
}

/// Syscall: Execute a program (simplified)
/// arg1: pointer to program name
/// arg2: length of program name
fn syscall_exec(_name_ptr: u64, _name_len: u64) -> u64 {
    // TODO: Full exec implementation requires program loading
    println!("exec() not yet implemented");
    u64::MAX
}

/// Syscall: Set signal handler
/// arg1: signal number
/// arg2: action (0=ignore, 1=default, addr=handler)
fn syscall_signal(signal_num: u64, action: u64) -> u64 {
    let signal = match crate::signal::Signal::from_u64(signal_num) {
        Some(s) => s,
        None => return u64::MAX, // Invalid signal
    };

    let signal_action = match action {
        0 => crate::signal::SignalAction::Ignore,
        1 => signal.default_action(),
        addr => crate::signal::SignalAction::Handler(addr),
    };

    if let Some(pid) = crate::process::current_pid() {
        let mut table = crate::process::PROCESS_TABLE.lock();
        if let Some(process) = table.get_process_mut(pid) {
            if let Some(disposition) = process.signal_disposition_mut() {
                match disposition.set_handler(signal, signal_action) {
                    Ok(()) => return 0,
                    Err(_) => return u64::MAX,
                }
            }
        }
    }

    u64::MAX
}

/// Syscall: Set signal mask (block/unblock signals)
/// arg1: operation (0=block, 1=unblock, 2=setmask)
/// arg2: signal number or mask
fn syscall_sigmask(operation: u64, signal_num: u64) -> u64 {
    let signal = match crate::signal::Signal::from_u64(signal_num) {
        Some(s) => s,
        None => return u64::MAX, // Invalid signal
    };

    if let Some(pid) = crate::process::current_pid() {
        let mut table = crate::process::PROCESS_TABLE.lock();
        if let Some(process) = table.get_process_mut(pid) {
            if let Some(disposition) = process.signal_disposition_mut() {
                match operation {
                    0 => disposition.block(signal),   // Block
                    1 => disposition.unblock(signal), // Unblock
                    _ => return u64::MAX,
                }
                return 0;
            }
        }
    }

    u64::MAX
}

/// Syscall: Create a pipe
/// arg1: pointer to array for [read_fd, write_fd]
fn syscall_pipe(fds_ptr: u64) -> u64 {
    match crate::pipe::create_pipe() {
        Ok((read_fd, write_fd)) => {
            // Write file descriptors to user memory
            unsafe {
                let fds = fds_ptr as *mut u32;
                if !fds.is_null() {
                    *fds.offset(0) = read_fd;
                    *fds.offset(1) = write_fd;
                    0
                } else {
                    u64::MAX
                }
            }
        }
        Err(_) => u64::MAX,
    }
}

/// Syscall: Read from file descriptor
/// arg1: file descriptor
/// arg2: buffer pointer
/// arg3: buffer length
fn syscall_read(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    if buf_ptr == 0 || len == 0 || len > 65536 {
        return u64::MAX;
    }

    unsafe {
        let buf = core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len as usize);

        // Try to read from pipe
        match crate::pipe::read_pipe(fd as u32, buf) {
            Ok(n) => n as u64,
            Err(_) => u64::MAX,
        }
    }
}

/// Syscall: Write to file descriptor
/// arg1: file descriptor
/// arg2: buffer pointer
/// arg3: buffer length
fn syscall_write(fd: u64, buf_ptr: u64, len: u64) -> u64 {
    if buf_ptr == 0 || len == 0 || len > 65536 {
        return u64::MAX;
    }

    unsafe {
        let buf = core::slice::from_raw_parts(buf_ptr as *const u8, len as usize);

        // Try to write to pipe
        match crate::pipe::write_pipe(fd as u32, buf) {
            Ok(n) => n as u64,
            Err(_) => u64::MAX,
        }
    }
}

/// Syscall: Close file descriptor
/// arg1: file descriptor
fn syscall_close(fd: u64) -> u64 {
    match crate::pipe::close_pipe(fd as u32) {
        Ok(()) => 0,
        Err(_) => u64::MAX,
    }
}

// User-space syscall API
// These functions can be called from tasks to invoke system calls

/// Exit the current task
#[inline(always)]
pub fn exit(code: u64) -> ! {
    unsafe {
        asm!(
            "mov rax, 0",  // SyscallNumber::Exit
            "mov rdi, {0}",
            "int 0x80",
            in(reg) code,
            options(noreturn)
        );
    }
}

/// Yield CPU to another task
#[inline(always)]
pub fn yield_cpu() {
    unsafe {
        asm!(
            "mov rax, 1",  // SyscallNumber::Yield
            "int 0x80",
            lateout("rax") _,
        );
    }
}

/// Print a string via syscall
#[inline(always)]
pub fn sys_print(s: &str) {
    unsafe {
        asm!(
            "mov rax, 2",  // SyscallNumber::Print
            "mov rdi, {0}",
            "mov rsi, {1}",
            "int 0x80",
            in(reg) s.as_ptr() as u64,
            in(reg) s.len() as u64,
            lateout("rax") _,
        );
    }
}

/// Get system uptime in seconds
#[inline(always)]
pub fn get_time() -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "mov rax, 3",  // SyscallNumber::GetTime
            "int 0x80",
            lateout("rax") result,
        );
    }
    result
}

/// Get system ticks
#[inline(always)]
pub fn get_ticks() -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "mov rax, 4",  // SyscallNumber::GetTicks
            "int 0x80",
            lateout("rax") result,
        );
    }
    result
}

/// Sleep for specified milliseconds
#[inline(always)]
pub fn sleep(ms: u64) {
    unsafe {
        asm!(
            "mov rax, 5",  // SyscallNumber::Sleep
            "mov rdi, {0}",
            "int 0x80",
            in(reg) ms,
            lateout("rax") _,
        );
    }
}

/// Get current process ID
#[inline(always)]
pub fn getpid() -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "mov rax, 6",  // SyscallNumber::GetPid
            "int 0x80",
            lateout("rax") result,
        );
    }
    result
}

/// Get parent process ID
#[inline(always)]
pub fn getppid() -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "mov rax, 7",  // SyscallNumber::GetPPid
            "int 0x80",
            lateout("rax") result,
        );
    }
    result
}

/// Fork current process (creates child process)
#[inline(always)]
pub fn fork() -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "mov rax, 8",  // SyscallNumber::Fork
            "int 0x80",
            lateout("rax") result,
        );
    }
    result
}

/// Wait for child process to terminate
/// Returns (child_pid << 32) | exit_code, or u64::MAX if no children
#[inline(always)]
pub fn wait() -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "mov rax, 9",  // SyscallNumber::Wait
            "int 0x80",
            lateout("rax") result,
        );
    }
    result
}

/// Kill a process with a signal
#[inline(always)]
pub fn kill(pid: u64, signal: u64) -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "mov rax, 10",  // SyscallNumber::Kill
            "mov rdi, {0}",
            "mov rsi, {1}",
            "int 0x80",
            in(reg) pid,
            in(reg) signal,
            lateout("rax") result,
        );
    }
    result
}

/// Syscall macro for easier invocation
#[macro_export]
macro_rules! syscall {
    ($num:expr) => {
        $crate::syscall::syscall0($num)
    };
    ($num:expr, $arg1:expr) => {
        $crate::syscall::syscall1($num, $arg1)
    };
    ($num:expr, $arg1:expr, $arg2:expr) => {
        $crate::syscall::syscall2($num, $arg1, $arg2)
    };
}

/// Raw syscall with 0 arguments
#[inline(always)]
pub fn syscall0(num: u64) -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num,
            lateout("rax") result,
        );
    }
    result
}

/// Raw syscall with 1 argument
#[inline(always)]
pub fn syscall1(num: u64, arg1: u64) -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") arg1,
            lateout("rax") result,
        );
    }
    result
}

/// Raw syscall with 2 arguments
#[inline(always)]
pub fn syscall2(num: u64, arg1: u64, arg2: u64) -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") arg1,
            in("rsi") arg2,
            lateout("rax") result,
        );
    }
    result
}
