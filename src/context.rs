/// CPU context containing all registers that need to be saved/restored during a context switch
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Context {
    // Callee-saved registers (preserved across function calls)
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbx: u64,
    pub rbp: u64,

    // Instruction pointer (where to resume execution)
    pub rip: u64,

    // Stack pointer
    pub rsp: u64,

    // RFLAGS (CPU flags register)
    pub rflags: u64,
}

impl Context {
    /// Create a new context initialized to zero
    pub const fn new() -> Self {
        Context {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
            rip: 0,
            rsp: 0,
            rflags: 0x200, // Interrupts enabled flag
        }
    }

    /// Initialize a context for a new task with an entry point and stack
    pub fn init(&mut self, entry_point: extern "C" fn(), stack_top: u64) {
        self.rip = entry_point as u64;
        self.rsp = stack_top;
        self.rflags = 0x200; // Enable interrupts (IF flag)

        // Set up a simple stack frame
        // The stack should be 16-byte aligned before a call instruction
        self.rsp &= !0xF; // Align to 16 bytes

        // Clear other registers
        self.r15 = 0;
        self.r14 = 0;
        self.r13 = 0;
        self.r12 = 0;
        self.rbx = 0;
        self.rbp = 0;
    }
}

/// Switch from the current context to a new context
///
/// # Safety
/// This function is unsafe because it directly manipulates CPU registers
/// and can cause undefined behavior if contexts are not properly initialized
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(old: *mut Context, new: *const Context) {
    core::arch::naked_asm!(
        // Save current context (pointed to by RDI - first parameter)
        "mov [rdi + 0x00], r15",
        "mov [rdi + 0x08], r14",
        "mov [rdi + 0x10], r13",
        "mov [rdi + 0x18], r12",
        "mov [rdi + 0x20], rbx",
        "mov [rdi + 0x28], rbp",

        // Save return address as RIP
        "mov rax, [rsp]",        // Get return address from stack
        "mov [rdi + 0x30], rax", // Save as RIP

        // Save stack pointer (after popping return address)
        "lea rax, [rsp + 8]",    // RSP after the ret would pop 8 bytes
        "mov [rdi + 0x38], rax",

        // Save RFLAGS
        "pushfq",
        "pop rax",
        "mov [rdi + 0x40], rax",

        // Load new context (pointed to by RSI - second parameter)
        "mov r15, [rsi + 0x00]",
        "mov r14, [rsi + 0x08]",
        "mov r13, [rsi + 0x10]",
        "mov r12, [rsi + 0x18]",
        "mov rbx, [rsi + 0x20]",
        "mov rbp, [rsi + 0x28]",

        // Load new RIP (instruction pointer)
        "mov rax, [rsi + 0x30]",

        // Load new RSP (stack pointer)
        "mov rsp, [rsi + 0x38]",

        // Load RFLAGS
        "mov rcx, [rsi + 0x40]",
        "push rcx",
        "popfq",

        // Jump to new RIP
        "jmp rax",
    )
}

/// Task entry point wrapper that calls the actual task function
pub extern "C" fn task_entry_wrapper() {
    // This will be customized per-task later
    // For now, just loop
    loop {
        x86_64::instructions::hlt();
    }
}
