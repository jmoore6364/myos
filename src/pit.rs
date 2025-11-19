// Programmable Interval Timer (PIT) driver
// Used for preemptive multitasking

use x86_64::instructions::port::Port;
use spin::Mutex;
use crate::println;

const PIT_FREQUENCY: u32 = 1193182; // Hz
const TARGET_FREQUENCY: u32 = 100;  // 100 Hz = 10ms intervals

static TICKS: Mutex<u64> = Mutex::new(0);

pub fn init() {
    let divisor = (PIT_FREQUENCY / TARGET_FREQUENCY) as u16;

    unsafe {
        // Command port: Channel 0, lobyte/hibyte, rate generator
        let mut command = Port::<u8>::new(0x43);
        command.write(0x36);

        // Data port: Send divisor
        let mut data = Port::<u8>::new(0x40);
        data.write((divisor & 0xFF) as u8);
        data.write((divisor >> 8) as u8);
    }

    println!("PIT initialized: {} Hz ({}ms intervals)", TARGET_FREQUENCY, 1000 / TARGET_FREQUENCY);
}

pub fn tick() {
    let mut ticks = TICKS.lock();
    *ticks += 1;

    // Every 100 ticks (1 second), trigger scheduler
    if *ticks % 100 == 0 {
        // Scheduler will be triggered
    }
}

pub fn get_ticks() -> u64 {
    *TICKS.lock()
}

pub fn get_seconds() -> u64 {
    get_ticks() / TARGET_FREQUENCY as u64
}
