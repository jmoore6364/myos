use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Double fault stack
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        // Kernel stack for privilege level 0 (used when transitioning from Ring 3 to Ring 0)
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut KERNEL_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &KERNEL_STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
                user_code_selector,
                user_data_selector,
            },
        )
    };
}

pub struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

// Export user segment selectors as constants for easy access
pub const USER_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(4, x86_64::PrivilegeLevel::Ring3);
pub const USER_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(5, x86_64::PrivilegeLevel::Ring3);

impl Selectors {
    pub fn get_user_code_selector() -> SegmentSelector {
        GDT.1.user_code_selector
    }

    pub fn get_user_data_selector() -> SegmentSelector {
        GDT.1.user_data_selector
    }
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}

/// Switch from kernel mode (Ring 0) to user mode (Ring 3)
/// This function sets up the stack frame and uses IRET to change privilege levels
///
/// # Safety
/// This function is unsafe because it directly manipulates the CPU state and
/// jumps to arbitrary code. The caller must ensure that:
/// - The user_fn pointer is valid
/// - The user stack is properly allocated and aligned
pub unsafe fn switch_to_usermode(user_fn: extern "C" fn() -> !, user_stack: VirtAddr) -> ! {
    use x86_64::registers::rflags::RFlags;

    let user_cs = Selectors::get_user_code_selector().0 as u64;
    let user_ss = Selectors::get_user_data_selector().0 as u64;
    let user_rflags = (RFlags::INTERRUPT_FLAG).bits(); // Enable interrupts in user mode
    let user_rsp = user_stack.as_u64();
    let user_rip = user_fn as *const () as u64;

    // Use inline assembly to set up the IRET frame and switch to user mode
    // Stack layout for IRET (from top to bottom):
    // - SS (stack segment)
    // - RSP (stack pointer)
    // - RFLAGS
    // - CS (code segment)
    // - RIP (instruction pointer)
    core::arch::asm!(
        // Set user data segments
        "mov ax, {user_ss:x}",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",

        // Push IRET frame
        "push {user_ss}",      // SS
        "push {user_rsp}",     // RSP
        "push {user_rflags}",  // RFLAGS
        "push {user_cs}",      // CS
        "push {user_rip}",     // RIP

        // Return to user mode
        "iretq",

        user_ss = in(reg) user_ss,
        user_rsp = in(reg) user_rsp,
        user_rflags = in(reg) user_rflags,
        user_cs = in(reg) user_cs,
        user_rip = in(reg) user_rip,
        options(noreturn)
    );
}
