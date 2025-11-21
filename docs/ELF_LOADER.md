# ELF Binary Loader for MyOS

## Overview

MyOS now supports loading and parsing ELF64 (Executable and Linkable Format) binaries. This enables the OS to load compiled programs from the filesystem and execute them in user mode (Ring 3) with proper memory isolation.

## Features Implemented

### 1. ELF64 Parser (`src/elf.rs`)

- **ELF Header Validation**
  - Validates ELF magic number (`0x7F 'E' 'L' 'F'`)
  - Checks for 64-bit format
  - Verifies little-endian byte order
  - Ensures executable file type
  - Confirms x86-64 architecture

- **Program Header Parsing**
  - Parses all PT_LOAD segments
  - Extracts virtual addresses, file sizes, and memory sizes
  - Reads segment flags (R/W/X permissions)

- **Binary Loading**
  - Allocates physical frames for each page
  - Maps pages into process-specific page tables
  - Copies segment data from file
  - Initializes BSS (uninitialized data) to zero
  - Respects segment alignment and offsets

### 2. Process Integration (`src/process.rs`)

- **`Process::load_elf()`**
  - Loads an ELF binary into a process's address space
  - Uses per-process page tables for memory isolation
  - Updates process memory usage statistics
  - Returns the program entry point address

- **`Process::setup_user_stack()`**
  - Allocates an 8 MB user-mode stack
  - Places stack at `0x7FFFFFFF000` (top of user space)
  - Stack grows downward as per x86-64 convention
  - Returns aligned stack pointer (16-byte aligned for ABI compliance)

### 3. Memory Management Extensions (`src/memory.rs`)

- **`allocate_frame()`** - Global frame allocator access
- **`map_page()` - Map pages in current page table
- **`process_memory::map_page_in_table()`** - Map pages in specific process page table

### 4. Shell Command (`src/shell.rs`)

- **`loadelf <file>`** command
  - Reads ELF binary from VFS
  - Creates a new process
  - Loads the binary into the process
  - Sets up user stack
  - Displays loading information

## Memory Layout

### ELF Program Segments
- Loaded at virtual addresses specified in ELF program headers
- Typical layout for test binaries:
  - `0x3FF000` - ELF header and metadata
  - `0x400000` - Code (.text) - Read + Execute
  - `0x401000` - Data (.rodata, .data) - Read + Write

### User Stack
- **Location**: `0x7FFFFFFF000` (top of user space)
- **Size**: 8 MB (2048 pages)
- **Growth**: Downward
- **Alignment**: 16-byte aligned

### Page Table Flags
Based on ELF segment flags:
- **Read-only**: `PRESENT | USER_ACCESSIBLE | NO_EXECUTE`
- **Readable + Writable**: `PRESENT | USER_ACCESSIBLE | WRITABLE | NO_EXECUTE`
- **Readable + Executable**: `PRESENT | USER_ACCESSIBLE` (no `WRITABLE`, no `NO_EXECUTE`)

## Test Programs

### Location
`test_programs/` directory

### Building Test Programs
```bash
cd test_programs
make
```

### Test Program: `hello`
- **Source**: `hello.c`
- **Functionality**:
  - Prints "Hello from ELF!" using MyOS syscall interface
  - Exits with code 42
- **Syscalls used**:
  - Syscall 6: `sys_print` - Print message to console
  - Syscall 0: `sys_exit` - Exit process

### ELF Binary Properties
```
Type:        ELF 64-bit LSB executable
Machine:     x86-64
Entry point: 0x400000
Size:        ~9 KB
Segments:    3 PT_LOAD segments
```

## Usage

### Loading an ELF Binary

1. **Build the test program**:
   ```bash
   cd test_programs
   make
   ```

2. **Copy binary to MyOS filesystem** (not yet automated):
   - Binary needs to be available in VFS
   - Future: Add disk I/O integration

3. **Load and run in MyOS**:
   ```
   > loadelf /bin/hello
   ```

### Expected Output
```
Loading ELF binary: /bin/hello
Size: 9216 bytes
Entry point: 0x0000000000400000
User stack: 0x00007FFFFFFFF000
Memory usage: 8196 KB

ELF binary loaded successfully!
Process: hello (PID 2)

Jumping to user mode...

[User program executes here - output depends on program]
Hello from ELF!
[Process exits via syscall]
```

The program transitions to Ring 3 and begins executing at the entry point!

## User-Mode Execution

### Implemented (NEW!)
The OS now **fully supports user-mode execution** of ELF binaries!

1. **IRETQ frame setup** - Proper transition from Ring 0 to Ring 3
2. **Page table switching** - Each process runs in its own isolated address space
3. **Segment descriptor setup** - User code/data segments (Ring 3)
4. **Syscall interface** - User programs can call kernel via INT 0x80

### How It Works

1. **Load ELF binary** - Parse and map into process page table
2. **Setup user stack** - 8 MB stack in user space
3. **Switch page tables** - Load process-specific CR3
4. **Set up IRETQ frame** - Push RIP, CS, RFLAGS, RSP, SS
5. **Execute IRETQ** - CPU transitions to Ring 3 automatically
6. **Program runs** - User code executes in isolated environment
7. **Syscalls** - INT 0x80 traps back to kernel

### Code: `src/usermode.rs`
```rust
pub unsafe fn jump_to_usermode(
    entry_point: VirtAddr,
    user_stack: VirtAddr,
    page_table_phys: Option<PhysAddr>,
) -> !
```

This function performs the actual privilege level transition using the IRETQ instruction.

## Current Limitations

2. **Dynamic linking** - Only static binaries supported
   - No ELF interpreter
   - No shared libraries (.so files)
   - No symbol resolution

3. **Advanced features**:
   - No copy-on-write for fork()
   - No demand paging (all pages allocated immediately)
   - No program arguments (argc/argv)
   - No environment variables

4. **Filesystem integration**:
   - Binaries must be in VFS (in-memory filesystem)
   - No integration with SimpleFS (persistent storage) yet

## Future Enhancements

### Near-term
1. **User-mode execution**
   - Integrate with task scheduler
   - Create Ring 3 tasks from ELF entry point
   - Switch page tables during context switch

2. **Program arguments**
   - Set up argc/argv on user stack
   - Parse command-line arguments

3. **Filesystem integration**
   - Load ELF binaries from SimpleFS
   - Support for `/bin`, `/usr/bin` directories

### Long-term
1. **Dynamic linking**
   - ELF interpreter support
   - Shared library loading
   - Dynamic symbol resolution

2. **Advanced memory management**
   - Copy-on-write for fork()
   - Demand paging
   - Memory-mapped files

3. **Standard library**
   - Port newlib or musl libc
   - POSIX API compatibility layer

## Architecture Details

### ELF Loading Flow

1. **Parse ELF header**
   - Validate magic number, class, endianness
   - Extract entry point and program header info

2. **Parse program headers**
   - Identify PT_LOAD segments
   - Extract virtual addresses and sizes

3. **Allocate and map pages**
   - For each segment:
     - Calculate page range
     - Allocate physical frames
     - Map into process page table
     - Set appropriate flags (R/W/X)

4. **Copy segment data**
   - Copy file data to mapped pages
   - Zero out BSS region

5. **Set up user stack**
   - Allocate 8 MB stack in user space
   - Return stack pointer

6. **Ready for execution**
   - Entry point and stack pointer known
   - Process ready to execute in Ring 3

### Security Considerations

- **Memory isolation**: Each process has its own page table
- **Privilege separation**: User code runs in Ring 3, kernel in Ring 0
- **NX bit**: Code pages are executable, data pages are not
- **User-accessible flag**: Prevents user code from accessing kernel memory

## References

- [ELF Specification](http://www.skyfree.org/linux/references/ELF_Format.pdf)
- [x86-64 ABI](https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf)
- [OSDev Wiki - ELF](https://wiki.osdev.org/ELF)

## Code Locations

- **ELF parser**: `src/elf.rs` (264 lines)
- **User-mode execution**: `src/usermode.rs` (169 lines) - NEW!
- **Process integration**: `src/process.rs` (lines 249-312)
- **Memory helpers**: `src/memory.rs` (lines 246-306)
- **GDT Ring 3 segments**: `src/gdt.rs` (lines 57-77, 89-138)
- **Shell command**: `src/shell.rs` (lines 1161-1250)
- **Test programs**: `test_programs/`
