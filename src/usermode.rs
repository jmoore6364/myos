//! User-mode task execution support
//!
//! This module provides functions to create and execute tasks in Ring 3 (user mode)
//! with proper privilege separation and memory isolation.

use x86_64::VirtAddr;
use x86_64::PhysAddr;
use crate::gdt;

/// User-mode context for transitioning to Ring 3
///
/// This structure represents the interrupt return frame that will be used
/// by the IRETQ instruction to transition from Ring 0 (kernel) to Ring 3 (user).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UserContext {
    // User program state
    pub entry_point: u64,    // RIP - where to start executing
    pub stack_pointer: u64,  // RSP - user stack pointer

    // Segment selectors (must be Ring 3 selectors)
    pub code_selector: u16,  // CS
    pub data_selector: u16,  // SS, DS, ES, FS, GS

    // Process page table
    pub page_table_phys: Option<PhysAddr>,
}

impl UserContext {
    /// Create a new user-mode context
    pub fn new(entry_point: u64, stack_pointer: u64, page_table_phys: Option<PhysAddr>) -> Self {
        UserContext {
            entry_point,
            stack_pointer,
            code_selector: gdt::Selectors::get_user_code_selector().0,
            data_selector: gdt::Selectors::get_user_data_selector().0,
            page_table_phys,
        }
    }
}

/// IRETQ stack frame for returning to Ring 3
///
/// The IRETQ instruction expects this exact layout on the stack:
/// - RIP (instruction pointer)
/// - CS (code segment)
/// - RFLAGS (CPU flags)
/// - RSP (stack pointer)
/// - SS (stack segment)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct IretqFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

/// Jump to user mode (Ring 3) using IRETQ
///
/// This function sets up the proper stack frame and uses the IRETQ instruction
/// to transition from Ring 0 (kernel mode) to Ring 3 (user mode).
///
/// # Safety
/// This function is unsafe because:
/// - It modifies CPU privilege level
/// - It changes code and stack segments
/// - The entry point and stack must be valid user-mode addresses
/// - The page table must be properly set up
///
/// # Arguments
/// * `entry_point` - Virtual address where user program starts
/// * `user_stack` - Virtual address of user stack top
/// * `page_table_phys` - Optional physical address of process page table
///
/// # Note
/// This function does NOT return - it transfers control to user mode
pub unsafe fn jump_to_usermode(
    entry_point: VirtAddr,
    user_stack: VirtAddr,
    page_table_phys: Option<PhysAddr>,
) -> ! {
    // Switch to user page table if provided
    if let Some(pt_phys) = page_table_phys {
        crate::memory::process_memory::switch_to_page_table(pt_phys);
    }

    // Set up data segments for user mode (Ring 3)
    let user_data_seg = gdt::Selectors::get_user_data_selector().0 as u64;

    core::arch::asm!(
        // Set DS, ES, FS, GS to user data segment
        "mov ax, {0:x}",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        in(reg) user_data_seg,
    );

    // RFLAGS value for user mode
    // Bit 9 (IF) = 1: Interrupts enabled
    // Bit 1 = 1: Reserved (always 1)
    // Bit 0 (CF) = 0: Carry flag cleared
    let rflags: u64 = 0x200 | 0x2; // IF | Reserved

    // Use IRETQ to jump to Ring 3
    // The stack frame must contain (from low to high address):
    // 1. RIP (entry point)
    // 2. CS (code segment selector with RPL=3)
    // 3. RFLAGS
    // 4. RSP (user stack pointer)
    // 5. SS (stack segment selector with RPL=3)

    core::arch::asm!(
        // Push IRETQ frame onto kernel stack
        "push {ss}",           // SS (user data segment with RPL=3)
        "push {rsp}",          // RSP (user stack)
        "push {rflags}",       // RFLAGS
        "push {cs}",           // CS (user code segment with RPL=3)
        "push {rip}",          // RIP (entry point)

        // Return to Ring 3
        "iretq",

        ss = in(reg) gdt::Selectors::get_user_data_selector().0 as u64,
        rsp = in(reg) user_stack.as_u64(),
        rflags = in(reg) rflags,
        cs = in(reg) gdt::Selectors::get_user_code_selector().0 as u64,
        rip = in(reg) entry_point.as_u64(),
        options(noreturn)
    );
}

/// Create a kernel task that will jump to user mode
///
/// This creates a kernel-mode task whose job is to set up and transition
/// to user mode. The task itself runs in Ring 0 but immediately jumps to Ring 3.
pub fn create_usermode_task_entry(
    entry_point: u64,
    user_stack: u64,
    page_table_phys: Option<PhysAddr>,
) -> extern "C" fn() {
    // We need to create a unique function for each user-mode task
    // For now, we'll use a generic trampoline

    // Store the context in a static variable (this is a simplified approach)
    // In a real implementation, this would be per-task data

    extern "C" fn usermode_trampoline() {
        // This will be customized per-task
        // For now, this is just a placeholder
        crate::println!("Usermode trampoline called - not yet implemented!");
        loop {
            x86_64::instructions::hlt();
        }
    }

    usermode_trampoline
}

/// Execute a user-mode program
///
/// This is the high-level function to run a user program. It:
/// 1. Switches to the user page table
/// 2. Sets up segment registers
/// 3. Jumps to user mode using IRETQ
///
/// # Safety
/// Unsafe because it changes privilege level and segment registers
pub unsafe fn execute_user_program(
    entry_point: VirtAddr,
    user_stack: VirtAddr,
    page_table_phys: Option<PhysAddr>,
) -> ! {
    crate::println!("Jumping to user mode at 0x{:016x}", entry_point.as_u64());
    crate::println!("User stack at 0x{:016x}", user_stack.as_u64());

    jump_to_usermode(entry_point, user_stack, page_table_phys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_context_creation() {
        let ctx = UserContext::new(0x400000, 0x7FFFFFFF000, None);
        assert_eq!(ctx.entry_point, 0x400000);
        assert_eq!(ctx.stack_pointer, 0x7FFFFFFF000);
    }
}
