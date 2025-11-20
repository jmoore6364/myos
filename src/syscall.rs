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
    _arg3: u64,
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
