use spin::Mutex;

static BOOT_TIME: Mutex<Option<u64>> = Mutex::new(None);
static TICKS: Mutex<u64> = Mutex::new(0);

pub fn init() {
    // Initialize boot time with current timestamp approximation
    // In a real system, we'd read from RTC
    *BOOT_TIME.lock() = Some(0);
}

pub fn tick() {
    // Called by timer interrupt
    *TICKS.lock() += 1;
}

pub fn uptime_ms() -> u64 {
    // Each tick is approximately 10ms (100Hz timer)
    *TICKS.lock() * 10
}

pub fn uptime_seconds() -> u64 {
    uptime_ms() / 1000
}
