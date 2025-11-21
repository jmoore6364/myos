//! ELF (Executable and Linkable Format) binary loader
//! Supports ELF64 format for x86_64 architecture

use alloc::vec::Vec;
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

/// ELF magic number (0x7F 'E' 'L' 'F')
const ELF_MAGIC: [u8; 4] = [0x7F, 0x45, 0x4C, 0x46];

/// ELF header for 64-bit binaries
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],     // ELF identification
    pub e_type: u16,           // Object file type
    pub e_machine: u16,        // Machine type
    pub e_version: u32,        // Object file version
    pub e_entry: u64,          // Entry point address
    pub e_phoff: u64,          // Program header offset
    pub e_shoff: u64,          // Section header offset
    pub e_flags: u32,          // Processor-specific flags
    pub e_ehsize: u16,         // ELF header size
    pub e_phentsize: u16,      // Program header entry size
    pub e_phnum: u16,          // Number of program header entries
    pub e_shentsize: u16,      // Section header entry size
    pub e_shnum: u16,          // Number of section header entries
    pub e_shstrndx: u16,       // Section header string table index
}

/// ELF program header for 64-bit binaries
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,           // Segment type
    pub p_flags: u32,          // Segment flags
    pub p_offset: u64,         // Segment offset in file
    pub p_vaddr: u64,          // Virtual address
    pub p_paddr: u64,          // Physical address (ignored)
    pub p_filesz: u64,         // Size in file
    pub p_memsz: u64,          // Size in memory
    pub p_align: u64,          // Segment alignment
}

// ELF file types
const ET_EXEC: u16 = 2;        // Executable file

// ELF machine types
const EM_X86_64: u16 = 62;     // AMD x86-64

// Program header types
const PT_LOAD: u32 = 1;        // Loadable segment

// Program header flags
const PF_X: u32 = 0x1;         // Execute
const PF_W: u32 = 0x2;         // Write
const PF_R: u32 = 0x4;         // Read

/// Represents a loaded ELF binary
#[derive(Debug)]
pub struct ElfBinary {
    pub entry_point: u64,
    pub segments: Vec<ElfSegment>,
}

/// Represents a loadable segment
#[derive(Debug)]
pub struct ElfSegment {
    pub vaddr: u64,
    pub data: Vec<u8>,
    pub memsz: usize,
    pub flags: u32,
}

impl ElfBinary {
    /// Parse an ELF binary from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < core::mem::size_of::<Elf64Header>() {
            return Err("File too small to be valid ELF");
        }

        // Parse header
        let header = unsafe {
            &*(data.as_ptr() as *const Elf64Header)
        };

        // Validate ELF magic
        if header.e_ident[0..4] != ELF_MAGIC {
            return Err("Invalid ELF magic number");
        }

        // Check ELF class (64-bit)
        if header.e_ident[4] != 2 {
            return Err("Not a 64-bit ELF file");
        }

        // Check endianness (little-endian)
        if header.e_ident[5] != 1 {
            return Err("Not little-endian");
        }

        // Check file type (executable)
        if header.e_type != ET_EXEC {
            return Err("Not an executable file");
        }

        // Check machine type (x86-64)
        if header.e_machine != EM_X86_64 {
            return Err("Not an x86-64 binary");
        }

        // Parse program headers
        let mut segments = Vec::new();
        let phoff = header.e_phoff as usize;
        let phentsize = header.e_phentsize as usize;
        let phnum = header.e_phnum as usize;

        for i in 0..phnum {
            let ph_offset = phoff + i * phentsize;
            if ph_offset + core::mem::size_of::<Elf64ProgramHeader>() > data.len() {
                return Err("Program header out of bounds");
            }

            let ph = unsafe {
                &*((data.as_ptr() as usize + ph_offset) as *const Elf64ProgramHeader)
            };

            // Only process PT_LOAD segments
            if ph.p_type == PT_LOAD {
                let file_offset = ph.p_offset as usize;
                let filesz = ph.p_filesz as usize;

                if file_offset + filesz > data.len() {
                    return Err("Segment data out of bounds");
                }

                // Copy segment data
                let mut seg_data = Vec::new();
                seg_data.extend_from_slice(&data[file_offset..file_offset + filesz]);

                segments.push(ElfSegment {
                    vaddr: ph.p_vaddr,
                    data: seg_data,
                    memsz: ph.p_memsz as usize,
                    flags: ph.p_flags,
                });
            }
        }

        Ok(ElfBinary {
            entry_point: header.e_entry,
            segments,
        })
    }

    /// Get page table flags for a segment based on ELF flags
    pub fn get_page_flags(flags: u32) -> PageTableFlags {
        let mut page_flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;

        if flags & PF_W != 0 {
            page_flags |= PageTableFlags::WRITABLE;
        }

        if flags & PF_X == 0 {
            page_flags |= PageTableFlags::NO_EXECUTE;
        }

        page_flags
    }

    /// Load the binary into a process's page table
    /// Returns the entry point address
    pub fn load_into_process(&self, page_table_phys: x86_64::PhysAddr) -> Result<u64, &'static str> {
        use crate::memory;
        use crate::memory::PHYS_MEM_OFFSET;
        use x86_64::structures::paging::PhysFrame;

        let phys_offset = *PHYS_MEM_OFFSET.lock();

        for segment in &self.segments {
            // Calculate page range
            let start_addr = VirtAddr::new(segment.vaddr);
            let end_addr = start_addr + segment.memsz as u64 - 1u64;

            let start_page = Page::<Size4KiB>::containing_address(start_addr);
            let end_page = Page::<Size4KiB>::containing_address(end_addr);

            let page_flags = Self::get_page_flags(segment.flags);

            // Track allocated frames so we can copy data into them
            let mut frames = Vec::new();

            // Allocate and map pages for this segment
            for page in Page::range_inclusive(start_page, end_page) {
                // Allocate a frame for this page
                let frame = memory::allocate_frame()
                    .ok_or("Failed to allocate frame")?;

                // Map the page into the process's page table
                memory::process_memory::map_page_in_table(
                    page_table_phys,
                    page,
                    frame,
                    page_flags,
                )?;

                // Zero out the frame
                unsafe {
                    let frame_virt = phys_offset + frame.start_address().as_u64();
                    let frame_ptr = frame_virt.as_mut_ptr::<u8>();
                    core::ptr::write_bytes(frame_ptr, 0, 4096);
                }

                frames.push(frame);
            }

            // Copy segment data to the allocated frames page by page
            let mut data_offset = 0;
            let page_offset_in_first_page = (segment.vaddr as usize) & 0xFFF;

            for (page_idx, frame) in frames.iter().enumerate() {
                // Calculate offset within this page
                let offset_in_page = if page_idx == 0 {
                    page_offset_in_first_page
                } else {
                    0
                };

                // Calculate how much data to copy to this page
                let remaining_data = segment.data.len().saturating_sub(data_offset);
                if remaining_data == 0 {
                    break; // All data copied, remaining pages are BSS (already zeroed)
                }

                let bytes_to_copy = core::cmp::min(4096 - offset_in_page, remaining_data);

                // Copy data to the frame
                unsafe {
                    let frame_virt = phys_offset + frame.start_address().as_u64();
                    let dest_ptr = (frame_virt.as_u64() + offset_in_page as u64) as *mut u8;

                    core::ptr::copy_nonoverlapping(
                        segment.data.as_ptr().add(data_offset),
                        dest_ptr,
                        bytes_to_copy
                    );
                }

                data_offset += bytes_to_copy;
            }
        }

        Ok(self.entry_point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_magic() {
        assert_eq!(ELF_MAGIC, [0x7F, 0x45, 0x4C, 0x46]);
    }
}
