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

lazy_static! {
    pub static ref SHELL: Mutex<shell::Shell> = Mutex::new(shell::Shell::new());
    pub static ref HAL_REPL: Mutex<halscript::Interpreter> = Mutex::new(halscript::Interpreter::new());
    pub static ref SCHEDULER: Mutex<task::Scheduler> = Mutex::new(task::Scheduler::new());
}

/// Test task functions
extern "C" fn task_a() {
    let mut counter = 0;
    loop {
        println!("Task A running (iteration {})", counter);
        counter += 1;

        // Yield to other tasks
        for _ in 0..100000 {
            core::hint::spin_loop();
        }
    }
}

extern "C" fn task_b() {
    let mut counter = 0;
    loop {
        println!("  Task B running (iteration {})", counter);
        counter += 1;

        // Yield to other tasks
        for _ in 0..100000 {
            core::hint::spin_loop();
        }
    }
}

extern "C" fn task_c() {
    let mut counter = 0;
    loop {
        println!("    Task C running (iteration {})", counter);
        counter += 1;

        // Yield to other tasks
        for _ in 0..100000 {
            core::hint::spin_loop();
        }
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

    println!("[7/9] Enabling interrupts...");
    x86_64::instructions::interrupts::enable();

    println!("[8/9] Creating test tasks...");
    init_test_tasks();

    println!("[9/9] Starting scheduler...");

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
