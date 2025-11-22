use x86_64::{
    structures::paging::{PageTable, OffsetPageTable, PhysFrame, Size4KiB, FrameAllocator, PageTableFlags, Mapper, Page},
    VirtAddr, PhysAddr,
};
use spin::Mutex;
use lazy_static::lazy_static;

// Global frame allocator for runtime page table management
lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);
    pub static ref PHYS_MEM_OFFSET: Mutex<VirtAddr> = Mutex::new(VirtAddr::new(0));
}

/// Initialize memory management
pub fn init(boot_info: &'static mut bootloader::BootInfo) {
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { init_mapper(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // Store the physical memory offset and frame allocator globally
    *PHYS_MEM_OFFSET.lock() = phys_mem_offset;
    *FRAME_ALLOCATOR.lock() = Some(frame_allocator);
}

/// Returns a mutable reference to the active level 4 table
unsafe fn init_mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map
pub struct BootInfoFrameAllocator {
    memory_map: &'static bootloader::bootinfo::MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map
    pub unsafe fn init(memory_map: &'static bootloader::bootinfo::MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == bootloader::bootinfo::MemoryRegionType::Usable);
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub mod allocator {
    use linked_list_allocator::LockedHeap;
    use x86_64::{
        structures::paging::{
            mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
        },
        VirtAddr,
    };

    pub const HEAP_START: usize = 0x_4444_4444_0000;
    pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

    #[global_allocator]
    static ALLOCATOR: LockedHeap = LockedHeap::empty();

    pub fn init_heap(
        mapper: &mut impl Mapper<Size4KiB>,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    ) -> Result<(), MapToError<Size4KiB>> {
        let page_range = {
            let heap_start = VirtAddr::new(HEAP_START as u64);
            let heap_end = heap_start + HEAP_SIZE - 1u64;
            let heap_start_page = Page::containing_address(heap_start);
            let heap_end_page = Page::containing_address(heap_end);
            Page::range_inclusive(heap_start_page, heap_end_page)
        };

        for page in page_range {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            unsafe {
                mapper.map_to(page, frame, flags, frame_allocator)?.flush();
            }
        }

        unsafe {
            ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
        }

        Ok(())
    }
}

/// Per-process page table management
pub mod process_memory {
    use super::*;
    use x86_64::structures::paging::{PageTableFlags, PhysFrame, FrameDeallocator};
    use x86_64::registers::control::Cr3;

    /// User space memory region (starts at 0x1000_0000 = 256 MB)
    pub const USER_SPACE_START: u64 = 0x1000_0000;
    pub const USER_SPACE_END: u64 = 0x8000_0000; // 2 GB

    /// Create a new page table for a process by cloning the kernel mappings
    ///
    /// This creates a new level 4 page table and copies the kernel-space mappings
    /// from the current page table. User space is left unmapped initially.
    ///
    /// Returns the physical address of the new page table (to be loaded into CR3)
    pub fn create_process_page_table() -> Option<PhysAddr> {
        let phys_offset = *PHYS_MEM_OFFSET.lock();
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        let frame_allocator = frame_allocator.as_mut()?;

        // Allocate a frame for the new level 4 table
        let new_l4_frame = frame_allocator.allocate_frame()?;
        let new_l4_phys = new_l4_frame.start_address();

        unsafe {
            // Get the current level 4 table
            let (current_l4_frame, _) = Cr3::read();
            let current_l4_phys = current_l4_frame.start_address();
            let current_l4_virt = phys_offset + current_l4_phys.as_u64();
            let current_l4_table: &PageTable = &*(current_l4_virt.as_ptr() as *const PageTable);

            // Get the new level 4 table
            let new_l4_virt = phys_offset + new_l4_phys.as_u64();
            let new_l4_table: &mut PageTable = &mut *(new_l4_virt.as_mut_ptr() as *mut PageTable);

            // Clear the new table
            new_l4_table.zero();

            // Copy kernel-space mappings (upper half: entries 256-511)
            // User space (entries 0-255) is left empty for per-process mappings
            for i in 256..512 {
                new_l4_table[i] = current_l4_table[i].clone();
            }
        }

        Some(new_l4_phys)
    }

    /// Free a process page table
    ///
    /// This recursively frees all page tables associated with a process.
    /// Kernel mappings are NOT freed as they are shared.
    ///
    /// # Safety
    /// This must not be called while the page table is active in CR3
    pub unsafe fn free_process_page_table(page_table_phys: PhysAddr) {
        // For now, we don't implement recursive freeing to avoid complexity
        // In a real OS, we would:
        // 1. Walk through user space entries (0-255)
        // 2. Free all user page tables recursively
        // 3. Free the L4 table itself
        // This is left as a future enhancement
        let _ = page_table_phys; // Silence unused warning
    }

    /// Switch to a process's page table
    ///
    /// # Safety
    /// The caller must ensure the page table is valid and properly set up
    pub unsafe fn switch_to_page_table(page_table_phys: PhysAddr) {
        use x86_64::registers::control::Cr3;
        use x86_64::structures::paging::PageTableFlags as Flags;

        let frame = PhysFrame::containing_address(page_table_phys);
        let flags = Cr3::read().1; // Keep current flags
        Cr3::write(frame, flags);
    }

    /// Get the current page table physical address
    pub fn get_current_page_table() -> PhysAddr {
        use x86_64::registers::control::Cr3;
        let (frame, _) = Cr3::read();
        frame.start_address()
    }

    /// Copy a process's page table for fork()
    ///
    /// This creates a new page table that is an exact copy of the source,
    /// including all user space mappings and kernel mappings.
    ///
    /// Returns the physical address of the new page table
    pub fn copy_page_table(source_pt_phys: PhysAddr) -> Option<PhysAddr> {
        let phys_offset = *PHYS_MEM_OFFSET.lock();
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        let frame_allocator = frame_allocator.as_mut()?;

        // Allocate a frame for the new level 4 table
        let new_l4_frame = frame_allocator.allocate_frame()?;
        let new_l4_phys = new_l4_frame.start_address();

        unsafe {
            // Get source and destination L4 tables
            let src_l4_virt = phys_offset + source_pt_phys.as_u64();
            let src_l4_table: &PageTable = &*(src_l4_virt.as_ptr() as *const PageTable);

            let dst_l4_virt = phys_offset + new_l4_phys.as_u64();
            let dst_l4_table: &mut PageTable = &mut *(dst_l4_virt.as_mut_ptr() as *mut PageTable);

            // Clear the new table
            dst_l4_table.zero();

            // Copy kernel-space mappings (upper half: entries 256-511) - these are shared
            for i in 256..512 {
                dst_l4_table[i] = src_l4_table[i].clone();
            }

            // Copy user-space mappings (lower half: entries 0-255)
            for l4_index in 0..256 {
                let l4_entry = &src_l4_table[l4_index];
                if !l4_entry.flags().contains(PageTableFlags::PRESENT) {
                    continue; // Skip non-present entries
                }

                // Allocate new L3 table for this entry
                let new_l3_frame = frame_allocator.allocate_frame()?;
                let new_l3_phys = new_l3_frame.start_address();

                // Get source L3 table
                let src_l3_phys = l4_entry.addr();
                let src_l3_virt = phys_offset + src_l3_phys.as_u64();
                let src_l3_table: &PageTable = &*(src_l3_virt.as_ptr() as *const PageTable);

                let dst_l3_virt = phys_offset + new_l3_phys.as_u64();
                let dst_l3_table: &mut PageTable = &mut *(dst_l3_virt.as_mut_ptr() as *mut PageTable);

                dst_l3_table.zero();

                // Copy L3 entries
                for l3_index in 0..512 {
                    let l3_entry = &src_l3_table[l3_index];
                    if !l3_entry.flags().contains(PageTableFlags::PRESENT) {
                        continue;
                    }

                    // Allocate new L2 table
                    let new_l2_frame = frame_allocator.allocate_frame()?;
                    let new_l2_phys = new_l2_frame.start_address();

                    let src_l2_phys = l3_entry.addr();
                    let src_l2_virt = phys_offset + src_l2_phys.as_u64();
                    let src_l2_table: &PageTable = &*(src_l2_virt.as_ptr() as *const PageTable);

                    let dst_l2_virt = phys_offset + new_l2_phys.as_u64();
                    let dst_l2_table: &mut PageTable = &mut *(dst_l2_virt.as_mut_ptr() as *mut PageTable);

                    dst_l2_table.zero();

                    // Copy L2 entries
                    for l2_index in 0..512 {
                        let l2_entry = &src_l2_table[l2_index];
                        if !l2_entry.flags().contains(PageTableFlags::PRESENT) {
                            continue;
                        }

                        // Allocate new L1 table
                        let new_l1_frame = frame_allocator.allocate_frame()?;
                        let new_l1_phys = new_l1_frame.start_address();

                        let src_l1_phys = l2_entry.addr();
                        let src_l1_virt = phys_offset + src_l1_phys.as_u64();
                        let src_l1_table: &PageTable = &*(src_l1_virt.as_ptr() as *const PageTable);

                        let dst_l1_virt = phys_offset + new_l1_phys.as_u64();
                        let dst_l1_table: &mut PageTable = &mut *(dst_l1_virt.as_mut_ptr() as *mut PageTable);

                        dst_l1_table.zero();

                        // Copy L1 entries (actual page mappings)
                        for l1_index in 0..512 {
                            let l1_entry = &src_l1_table[l1_index];
                            if !l1_entry.flags().contains(PageTableFlags::PRESENT) {
                                continue;
                            }

                            // Allocate a new physical frame for the page data
                            let new_data_frame = frame_allocator.allocate_frame()?;
                            let new_data_phys = new_data_frame.start_address();

                            // Copy the page data from source to destination
                            let src_data_phys = l1_entry.addr();
                            let src_data_virt = phys_offset + src_data_phys.as_u64();
                            let dst_data_virt = phys_offset + new_data_phys.as_u64();

                            core::ptr::copy_nonoverlapping(
                                src_data_virt.as_ptr::<u8>(),
                                dst_data_virt.as_mut_ptr::<u8>(),
                                4096
                            );

                            // Set the L1 entry to point to the new frame with same flags
                            dst_l1_table[l1_index].set_addr(new_data_phys, l1_entry.flags());
                        }

                        // Set the L2 entry to point to the new L1 table
                        dst_l2_table[l2_index].set_addr(new_l2_phys, l2_entry.flags());
                    }

                    // Set the L3 entry to point to the new L2 table
                    dst_l3_table[l3_index].set_addr(new_l2_phys, l3_entry.flags());
                }

                // Set the L4 entry to point to the new L3 table
                dst_l4_table[l4_index].set_addr(new_l3_phys, l4_entry.flags());
            }
        }

        Some(new_l4_phys)
    }

    /// Allocate a page in user space for a process
    ///
    /// Returns the virtual address of the allocated page
    pub fn allocate_user_page(page_table_phys: PhysAddr, virt_addr: VirtAddr) -> Option<()> {
        let phys_offset = *PHYS_MEM_OFFSET.lock();
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        let frame_allocator = frame_allocator.as_mut()?;

        // Allocate a physical frame
        let frame = frame_allocator.allocate_frame()?;

        unsafe {
            // Create a mapper for this page table
            let l4_virt = phys_offset + page_table_phys.as_u64();
            let l4_table: &mut PageTable = &mut *(l4_virt.as_mut_ptr() as *mut PageTable);
            let mut mapper = OffsetPageTable::new(l4_table, phys_offset);

            // Map the page with user-accessible flags
            let page = Page::containing_address(virt_addr);
            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE;

            mapper
                .map_to(page, frame, flags, frame_allocator)
                .ok()?
                .flush();
        }

        Some(())
    }

    /// Map a page in a specific page table with custom flags
    ///
    /// This is used by the ELF loader to map program segments
    pub fn map_page_in_table(
        page_table_phys: PhysAddr,
        page: Page,
        frame: PhysFrame,
        flags: PageTableFlags,
    ) -> Result<(), &'static str> {
        let phys_offset = *PHYS_MEM_OFFSET.lock();
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        let frame_allocator = frame_allocator.as_mut().ok_or("Frame allocator not initialized")?;

        unsafe {
            // Create a mapper for this page table
            let l4_virt = phys_offset + page_table_phys.as_u64();
            let l4_table: &mut PageTable = &mut *(l4_virt.as_mut_ptr() as *mut PageTable);
            let mut mapper = OffsetPageTable::new(l4_table, phys_offset);

            // Map the page
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .map_err(|_| "Failed to map page")?
                .flush();
        }

        Ok(())
    }
}

/// Allocate a physical frame
pub fn allocate_frame() -> Option<PhysFrame> {
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    frame_allocator.as_mut()?.allocate_frame()
}

/// Map a page with specific flags (in the current page table)
///
/// # Safety
/// This function is unsafe because it modifies page tables
pub unsafe fn map_page(
    page: Page,
    frame: PhysFrame,
    flags: PageTableFlags,
) -> Result<(), &'static str> {
    let phys_offset = *PHYS_MEM_OFFSET.lock();
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator.as_mut().ok_or("Frame allocator not initialized")?;

    // Get the current page table
    let l4_table = active_level_4_table(phys_offset);
    let mut mapper = OffsetPageTable::new(l4_table, phys_offset);

    // Map the page
    mapper
        .map_to(page, frame, flags, frame_allocator)
        .map_err(|_| "Failed to map page")?
        .flush();

    Ok(())
}
