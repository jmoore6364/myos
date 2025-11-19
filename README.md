# MyOS - AI-Powered Operating System

A revolutionary operating system written in Rust that boots on bare metal x86_64 hardware with AI integration capabilities.

## Features

### Core Kernel
- **Bare Metal Boot**: Boots directly on x86_64 hardware
- **Memory Management**: Full paging and heap allocation
- **Interrupt Handling**: Hardware interrupts (keyboard, timer)
- **VGA Text Mode**: Color terminal output with 16-color palette
- **Keyboard Input**: Real-time PS/2 keyboard driver with full character support
- **Task Scheduler**: Cooperative round-robin task scheduling

### User Environment
- **Interactive Shell**: 39+ commands for system control
- **Virtual File System**: In-memory VFS with Unix-like commands (ls, cd, cat, mkdir, touch, rm, write, exec)
- **HAL Script Language**: Turing-complete language with 32 built-ins and module system (see [HALSCRIPT.md](HALSCRIPT.md))
- **Persistent REPL**: Variables and functions survive across commands
- **AI Natural Language Programming**: Convert English to code with `ai` command
- **Example Scripts**: 6 pre-loaded programs in `/scripts` directory
- **Standard Libraries**: Math and string utilities in `/lib` directory
- **Application Platform**: Built-in apps in `/apps` with `app` command to run them
- **Command-Line Arguments**: Pass arguments to scripts via the `args` global variable
- **Unit Tests**: Comprehensive test suite for language components

### Development
- **Rust Powered**: Memory-safe kernel with zero-cost abstractions
- **WebAssembly Ready**: Can be compiled to WASM for browser demos
- **Extensible**: Easy to add new commands and features

## Architecture

```
MyOS
├── Bootloader (bootloader crate)
├── Kernel Core
│   ├── GDT (Global Descriptor Table)
│   ├── IDT (Interrupt Descriptor Table)
│   ├── Memory Management (Paging + Heap)
│   ├── VGA Buffer Driver
│   └── Keyboard Driver
├── Shell & Scripting
│   ├── Interactive Shell (30+ commands)
│   ├── HAL Script Language (full Turing-complete)
│   ├── Persistent REPL
│   └── AI Natural Language Processor
├── File System
│   ├── Virtual File System (VFS)
│   ├── Directory Tree (/home, /scripts, /tmp)
│   └── Unix-like Commands
├── Planned Features
│   ├── Task Scheduler
│   ├── Disk Drivers (ATA/AHCI)
│   └── Network Stack
```

## Prerequisites

### Linux/macOS/Windows (WSL)

1. **Rust** (nightly toolchain)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly
```

2. **QEMU** (for testing)
```bash
# Ubuntu/Debian
sudo apt install qemu-system-x86

# macOS
brew install qemu

# Windows (via Chocolatey)
choco install qemu
```

3. **bootimage** tool
```bash
cargo install bootimage
```

## Building

```bash
# Build the OS
cargo build

# Build bootable image
cargo bootimage

# The bootable image will be at:
# target/x86_64-myos/debug/bootimage-myos.bin
```

## Running

### In QEMU (Recommended for testing)

```bash
# Run in QEMU
cargo run

# Or manually:
qemu-system-x86_64 -drive format=raw,file=target/x86_64-myos/debug/bootimage-myos.bin
```

## Quick Start Guide

Once the OS boots, try these commands:

```bash
# File system
> ls /
> cd /home
> cat welcome.txt
> mkdir /scripts
> write /scripts/hello.hal "print \"Hello, World!\""
> cat /scripts/hello.hal

# HAL Script programming
> run x = 42
> run print x * 2
> run fn fib(n) { if n < 2 { return n } return fib(n-1) + fib(n-2) }
> run print fib(10)

# File I/O from HAL Script
> run write_file("/tmp/test.txt", "Hello from HAL!")
> run content = read_file("/tmp/test.txt")
> run print content
> run files = list_dir("/scripts")
> run print files

# Maps (dictionaries)
> run user = {"name": "Alice", "age": 30, "city": "NYC"}
> run print user["name"]
> run print keys(user)
> run print map_size(user)

# AI natural language programming
> ai create a fibonacci function
> ai show prime numbers under 50
> ai count from 1 to 100
> ai calculate 10 factorial

# Execute scripts from VFS
> ls /scripts
> exec /scripts/fibonacci.hal
> exec /scripts/primes.hal

# Import and use library modules
> ls /lib
> exec /scripts/demo_app.hal
> run import "/lib/math.hal"
> run print factorial(6)

# Run applications with arguments
> app list
> app run calc 10 + 5
> app run greeter Alice
> app run primefind 50
> app run filemgr list /scripts

# Task management
> ps
> spawn worker1 5
> spawn worker2 10
> ps
> sched
> kill 0

# System commands
> help
> about
> sysinfo
> uptime
> colors
```

## Built-in HAL Script Functions (32 total)

**String/Type:** `len()`, `str()`, `num()`
**String Methods:** `split()`, `trim()`, `upper()`, `lower()`, `replace()`, `starts_with()`, `ends_with()`, `substring()`
**Math:** `abs()`, `min()`, `max()`, `pow()`, `sqrt()`
**Array:** `range()`, `push()`, `sum()`, `pop()`, `reverse()`, `join()`
**Map:** `keys()`, `values()`, `has_key()`, `map_size()`
**File I/O:** `read_file()`, `write_file()`, `file_exists()`, `list_dir()`
**System:** `uptime()`

**Control Flow:** `break`, `continue` (for loops and while loops)

See [HALSCRIPT.md](HALSCRIPT.md) for complete language reference with examples.
```

### On Real Hardware (USB Boot)

⚠️ **WARNING**: This will overwrite the USB drive!

```bash
# Write to USB drive (replace /dev/sdX with your USB device)
sudo dd if=target/x86_64-myos/debug/bootimage-myos.bin of=/dev/sdX bs=4M && sync
```

Then boot from the USB drive on your PC.

### In VirtualBox

1. Convert the image to VDI format:
```bash
qemu-img convert -f raw -O vdi \
  target/x86_64-myos/debug/bootimage-myos.bin \
  myos.vdi
```

2. Create a new VM in VirtualBox:
   - Type: Other
   - Version: Other/Unknown (64-bit)
   - Use existing virtual hard disk: `myos.vdi`

3. Boot the VM

## Development

### Project Structure

```
myos/
├── src/
│   ├── main.rs          # Kernel entry point
│   ├── vga_buffer.rs    # VGA text mode driver
│   ├── serial.rs        # Serial port for debugging
│   ├── interrupts.rs    # Interrupt handling
│   ├── gdt.rs           # Global Descriptor Table
│   ├── keyboard.rs      # Keyboard driver
│   └── memory.rs        # Memory management
├── Cargo.toml           # Rust dependencies
├── x86_64-myos.json     # Custom target specification
└── rust-toolchain.toml  # Rust toolchain config
```

### Testing

```bash
# Run kernel tests
cargo test
```

### Debugging

Serial output is available on COM1 (0x3F8) for debugging:

```bash
# Run with serial output
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-myos/debug/bootimage-myos.bin \
  -serial stdio
```

## Roadmap

### Phase 1: Core Kernel ✅
- [x] Bootloader integration
- [x] VGA text output with 16 colors
- [x] Interrupt handling (IDT, GDT, PIC)
- [x] Memory management (paging + heap)
- [x] Keyboard input (PS/2 driver)
- [x] Interactive shell with 30+ commands
- [x] HAL Script programming language
- [x] Virtual file system (VFS)
- [x] Persistent REPL
- [x] AI natural language programming

### Phase 2: Process Management (Next)
- [ ] Task scheduler
- [ ] Multitasking
- [ ] Process isolation

### Phase 3: Storage & I/O
- [ ] Virtual File System
- [ ] ATA/AHCI disk driver
- [ ] Basic filesystem (FAT32 or custom)

### Phase 4: User Interface
- [ ] Command shell
- [ ] Text-based UI
- [ ] Command parser

### Phase 5: AI Integration
- [ ] AI command processing
- [ ] Natural language interface
- [ ] Embedded ML models (TinyML)

### Phase 6: Advanced Features
- [ ] Networking (TCP/IP stack)
- [ ] WebAssembly runtime
- [ ] Browser-based demo version

## Performance

MyOS is designed to be:
- **Fast**: Rust's zero-cost abstractions ensure C/C++ level performance
- **Safe**: Memory safety without garbage collection
- **Small**: Minimal kernel footprint (~100KB base)

## Contributing

This is an experimental OS project. Contributions welcome!

## License

MIT License

## Technical Details

### Boot Process

1. BIOS/UEFI loads bootloader
2. Bootloader enters protected mode
3. Bootloader loads kernel at 1MB
4. Control transfers to `_start()`
5. Kernel initializes GDT, IDT, memory
6. Interrupts enabled
7. Enter main loop

### Memory Layout

```
0x0000_0000 - 0x0010_0000  : Real mode (1MB)
0x0010_0000 - 0x0020_0000  : Kernel code
0x4444_4444_0000          : Heap start (100KB)
0xb8000                   : VGA text buffer
```

## Resources

- [OSDev Wiki](https://wiki.osdev.org/)
- [Writing an OS in Rust](https://os.phil-opp.com/)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)
