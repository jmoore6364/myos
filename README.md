# MyOS - AI-Powered Operating System

A revolutionary operating system written in Rust that boots on bare metal x86_64 hardware with AI integration capabilities.

## Features

- **Bare Metal Boot**: Boots directly on x86_64 hardware
- **Memory Management**: Full paging and heap allocation
- **Interrupt Handling**: Hardware interrupts (keyboard, timer)
- **VGA Text Mode**: Color terminal output
- **Keyboard Input**: Real-time keyboard driver
- **Rust Powered**: Memory-safe kernel with zero-cost abstractions
- **WebAssembly Ready**: Can be compiled to WASM for browser demos

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
├── Planned Features
│   ├── Task Scheduler
│   ├── Virtual File System
│   ├── Command Shell
│   └── AI Integration Layer
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
- [x] VGA text output
- [x] Interrupt handling
- [x] Memory management
- [x] Keyboard input

### Phase 2: Process Management (In Progress)
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
