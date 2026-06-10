# Running MyOS

MyOS is a bare-metal operating system written in Rust that runs in QEMU or on real x86-64 hardware.

## Current Status

The OS builds successfully and has **100+ shell commands** including:
- File system operations (ls, cat, mkdir, etc.)
- Networking utilities (ping, wget, curl, ifconfig)
- Text processing (grep, sed, awk, sort)
- System administration (useradd, passwd, crontab, sudo)
- Compression (gzip, bzip2, xz)
- And many more!

## Compatibility Note

This project uses `bootloader 0.9.23` which has some compatibility issues with very recent Rust nightly toolchains. The kernel itself compiles perfectly, but creating a bootable image requires either:

### Option 1: Use a Compatible Rust Toolchain (Recommended)

```bash
# Install a specific Rust nightly from 2024
rustup toolchain install nightly-2024-03-01
rustup override set nightly-2024-03-01

# Install required tools
rustup component add llvm-tools-preview --toolchain nightly-2024-03-01
cargo +nightly-2024-03-01 install bootimage

# Build and run
cargo +nightly-2024-03-01 bootimage --release
```

### Option 2: Upgrade to Bootloader 0.11+ (Requires Code Changes)

The newer bootloader versions (0.11+) are compatible with modern Rust but have API changes:
- Removed `map_physical_memory` feature
- Changed entry point signature
- Different memory mapping approach

This would require updating `src/main.rs` and `src/memory.rs`.

### Option 3: Create ISO with GRUB

```bash
# Build the kernel
cargo build --release

# Create ISO directory structure
mkdir -p isodir/boot/grub

# Copy kernel
cp target/x86_64-unknown-none/release/myos isodir/boot/

# Create GRUB config
cat > isodir/boot/grub/grub.cfg <<EOF
menuentry "MyOS" {
    multiboot2 /boot/myos
    boot
}
EOF

# Create ISO
grub-mkrescue -o myos.iso isodir

# Run in QEMU
qemu-system-x86_64 -cdrom myos.iso
```

## What You Can Explore Now

Even without running in QEMU, you can explore the OS features:

### View All Commands
```bash
# See all 100+ implemented commands
grep '".*" =>' src/shell.rs | head -50
```

### Browse Command Categories
- **System**: help, about, sysinfo, status, uptime, etc.
- **File System**: ls, cd, cat, mkdir, rm, cp, mv, tree, find, etc.
- **Text Processing**: grep, sed, awk, sort, uniq, cut, tr, diff
- **Networking**: ping, ifconfig, netstat, wget, curl
- **Compression**: tar, zip, gzip, bzip2, xz
- **Administration**: useradd, passwd, sudo, crontab
- **Performance**: time, benchmark, top
- **Job Control**: jobs, bg, fg

### Try the HAL Script Language
The OS includes a custom scripting language! Check `src/halscript/` for the implementation.

## Features

- ✅ Custom VGA text mode driver
- ✅ Keyboard input handling  
- ✅ Memory management (heap allocator)
- ✅ Preemptive multitasking scheduler
- ✅ Virtual file system (VFS)
- ✅ Simple filesystem implementation
- ✅ IPC mechanisms (pipes, shared memory, semaphores, message queues)
- ✅ System call interface
- ✅ ELF binary loading
- ✅ Custom scripting language (HAL Script)
- ✅ 100+ shell commands

## Architecture

- **Bootloader**: bootloader 0.9 (loads kernel at boot)
- **Target**: x86_64 bare metal (no_std)
- **Allocator**: linked_list_allocator
- **Drivers**: VGA text, PS/2 keyboard, ATA disk, serial UART
- **Scheduler**: Round-robin preemptive multitasking
- **IPC**: Full Unix-style IPC suite

## Development

```bash
# Build kernel (always works)
cargo build --release

# Check the binary size
ls -lh target/x86_64-unknown-none/release/myos

# Run tests (if configured)
cargo test
```

## Future Improvements

1. Update to bootloader 0.11+ for better Rust compatibility
2. Add graphical mode support
3. Implement network stack (TCP/IP)
4. Add USB driver support
5. Real filesystem persistence
6. Multi-core support

## Credits

Built as an educational operating system to demonstrate OS concepts in Rust!
