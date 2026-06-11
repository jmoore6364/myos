#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;

mod vga_buffer;
mod serial;
mod interrupts;
mod gdt;
mod keyboard;
mod memory;
mod time;
mod pit;
mod shell;
mod halscript;
mod vfs;
mod task;
mod context;
mod syscall;
mod process;
mod signal;
mod pipe;
mod shm;
mod sem;
mod msgq;
mod ata;
mod simplefs;
mod elf;
mod usermode;
mod net;
mod drivers;

lazy_static! {
    pub static ref SHELL: Mutex<shell::Shell> = Mutex::new(shell::Shell::new());
    pub static ref HAL_REPL: Mutex<halscript::Interpreter> = Mutex::new(halscript::Interpreter::new());
    pub static ref SCHEDULER: Mutex<task::Scheduler> = Mutex::new(task::Scheduler::new());
}

/// Test task functions demonstrating syscalls
extern "C" fn task_a() {
    use crate::syscall;

    for i in 0..5 {
        let msg = alloc::format!("Task A iteration {}\n", i);
        syscall::sys_print(&msg);

        // Sleep using syscall
        syscall::sleep(500); // 500ms
    }

    syscall::sys_print("Task A completed!\n");
    syscall::exit(0);
}

extern "C" fn task_b() {
    use crate::syscall;

    for i in 0..5 {
        let msg = alloc::format!("  Task B iteration {}\n", i);
        syscall::sys_print(&msg);

        // Sleep using syscall
        syscall::sleep(700); // 700ms
    }

    syscall::sys_print("  Task B completed!\n");
    syscall::exit(0);
}

extern "C" fn task_c() {
    use crate::syscall;

    for i in 0..5 {
        let msg = alloc::format!("    Task C iteration {} (time: {}s)\n", i, syscall::get_time());
        syscall::sys_print(&msg);

        // Sleep using syscall
        syscall::sleep(1000); // 1000ms
    }

    syscall::sys_print("    Task C completed!\n");
    syscall::exit(0);
}

/// User mode test task - runs in Ring 3
extern "C" fn user_mode_task() -> ! {
    use crate::syscall;

    // This task runs in Ring 3 (user mode) and demonstrates syscalls
    syscall::sys_print("🔒 USER MODE: Starting user mode task (Ring 3)\n");

    for i in 0..3 {
        let msg = alloc::format!("🔒 USER MODE: Iteration {} (time: {}s)\n", i, syscall::get_time());
        syscall::sys_print(&msg);
        syscall::sleep(800); // 800ms
    }

    syscall::sys_print("🔒 USER MODE: User mode task completed successfully!\n");
    syscall::exit(0);

    // Should never reach here
    loop {
        x86_64::instructions::hlt();
    }
}

/// Initialize test tasks for demonstrating multitasking
fn init_test_tasks() {
    let mut scheduler = SCHEDULER.lock();

    // Create three test tasks
    let task_a = task::Task::new(alloc::string::String::from("Task A"), 1, task_a);
    let task_b = task::Task::new(alloc::string::String::from("Task B"), 1, task_b);
    let task_c = task::Task::new(alloc::string::String::from("Task C"), 1, task_c);

    scheduler.add_task(task_a);
    scheduler.add_task(task_b);
    scheduler.add_task(task_c);

    println!("  Created {} tasks", scheduler.task_count());
}

/// Allocate a user mode stack
fn allocate_user_stack(size: usize) -> x86_64::VirtAddr {
    use alloc::alloc::{alloc, Layout};

    unsafe {
        let layout = Layout::from_size_align(size, 16).expect("Invalid layout");
        let ptr = alloc(layout);
        if ptr.is_null() {
            panic!("Failed to allocate user stack");
        }

        // Return the top of the stack (stacks grow downward)
        x86_64::VirtAddr::from_ptr(ptr) + size
    }
}

/// Initialize and switch to user mode task (Ring 3 demonstration)
fn test_user_mode() {
    println!("\n[TEST] Switching to user mode (Ring 3)...");
    println!("[TEST] This demonstrates kernel/user privilege separation");

    // Allocate a stack for user mode
    const USER_STACK_SIZE: usize = 4096 * 4; // 16KB
    let user_stack = allocate_user_stack(USER_STACK_SIZE);

    println!("[TEST] User stack allocated at: {:#x}", user_stack.as_u64());
    println!("[TEST] Jumping to Ring 3...\n");

    // Switch to user mode - this never returns
    unsafe {
        gdt::switch_to_usermode(user_mode_task, user_stack);
    }
}

/// Entry point for the kernel
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static mut bootloader::BootInfo) -> ! {
    println!("╔════════════════════════════════════════════╗");
    println!("║          MyOS v0.1.0 - Booting...         ║");
    println!("╚════════════════════════════════════════════╝");
    println!();

    // Initialize kernel subsystems
    println!("[1/6] Initializing GDT...");
    gdt::init();

    println!("[2/6] Initializing IDT...");
    interrupts::init_idt();

    println!("[3/6] Initializing PIC...");
    unsafe { interrupts::PICS.lock().initialize() };

    println!("[4/6] Initializing memory...");
    memory::init(boot_info);

    println!("[5/7] Initializing time...");
    time::init();

    println!("[6/7] Initializing PIT...");
    pit::init();

    println!("[7/11] Initializing process subsystem...");
    process::init();

    println!("[8/12] Initializing ATA disk driver...");
    ata::init();

    println!("[9/13] Initializing filesystem...");
    simplefs::init();

    println!("[10/13] Initializing network stack...");
    if let Err(e) = drivers::e1000::init() {
        println!("  Warning: Network initialization failed: {}", e);
        println!("  Network commands will not be available.");
    } else {
        if let Some(mac) = drivers::e1000::get_mac_address() {
            println!("  ✓ Network card initialized - MAC: {}", mac);
        }
    }

    println!("[11/13] Enabling interrupts...");
    x86_64::instructions::interrupts::enable();

    println!("[12/13] Creating test tasks...");
    init_test_tasks();

    println!("[13/13] Starting scheduler...");

    println!();
    println!("✓ Kernel initialized successfully!");
    println!();
    println!("Welcome to MyOS - The AI-Powered Operating System");
    println!("System Information:");
    println!("  Architecture: x86_64");
    println!("  Build: Development");
    println!("  Features: Interrupts, Memory Management, Keyboard Input");
    println!();
    println!("Type 'help' for available commands");
    println!();

    #[cfg(test)]
    test_main();

    // Print initial shell prompt
    print!("> ");

    // Main kernel loop
    loop {
        x86_64::instructions::hlt();
    }
}

/// Panic handler
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
fn test_harness(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
