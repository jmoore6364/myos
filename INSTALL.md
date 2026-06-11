# Running MyOS - Installation & Setup Guide

MyOS is a bare-metal operating system written in Rust with a complete TCP/IP stack, 100+ shell commands, and real networking capabilities.

## 🎯 What You'll Get

- **Complete TCP/IP Network Stack** (2,230 lines written from scratch)
  - Real ping with ICMP protocol
  - E1000 network driver with DMA
  - ARP, IPv4, Ethernet layers
- **100+ Shell Commands** (file system, networking, text processing, compression, etc.)
- **Process Scheduler** with preemptive multitasking
- **Virtual File System** with IPC (pipes, shared memory, semaphores)
- **HAL Script** - Custom scripting language
- **ELF Binary Loading** - Run compiled programs

---

## 📦 Installation

Choose your platform:

### 🪟 **Windows (Native)**

#### Step 1: Install Rust

**Option A: Using Rustup (Recommended)**
1. Download: https://rustup.rs/
2. Run `rustup-init.exe`
3. Follow the prompts (default options are fine)
4. **Restart your terminal**

**Option B: Using Chocolatey**
```cmd
choco install rust
```

#### Step 2: Configure for OS Development

```cmd
# Set nightly as default (required for no_std)
rustup default nightly

# Add required components
rustup component add rust-src llvm-tools-preview

# Add bare-metal target
rustup target add x86_64-unknown-none
```

#### Step 3: Install Bootimage Tool

```cmd
cargo install bootimage
```

#### Step 4: Install QEMU (if not already installed)

```cmd
choco install qemu
```

#### Step 5: Clone and Build

```cmd
# Clone the repository
git clone https://github.com/yourusername/myos.git
cd myos

# Build the kernel
cargo build --release

# Create bootable image
cargo bootimage --release

# Run in QEMU with networking
qemu-system-x86_64 ^
    -drive format=raw,file=target\x86_64-unknown-none\release\bootimage-myos.bin ^
    -netdev user,id=net0 ^
    -device e1000,netdev=net0 ^
    -serial stdio ^
    -m 128M
```

---

### 🐧 **Linux**

#### Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Step 2: Configure Toolchain

```bash
rustup default nightly
rustup component add rust-src llvm-tools-preview
rustup target add x86_64-unknown-none
```

#### Step 3: Install Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install qemu-system-x86 build-essential
```

**Fedora/RHEL:**
```bash
sudo dnf install qemu-system-x86 gcc
```

**Arch:**
```bash
sudo pacman -S qemu-system-x86 base-devel
```

#### Step 4: Install Bootimage

```bash
cargo install bootimage
```

#### Step 5: Build and Run

```bash
# Clone
git clone https://github.com/yourusername/myos.git
cd myos

# Build
cargo build --release

# Create bootable image
cargo bootimage --release

# Run with networking
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/bootimage-myos.bin \
    -netdev user,id=net0 \
    -device e1000,netdev=net0 \
    -serial stdio \
    -m 128M
```

---

### 🐧 **WSL2 (Windows Subsystem for Linux)** - Recommended for Windows!

WSL2 gives you a full Linux environment on Windows, making OS development much easier.

#### Step 1: Install WSL2

```powershell
# In PowerShell as Administrator
wsl --install
```

Restart your computer, then open "Ubuntu" from Start menu.

#### Step 2: Install Rust in WSL

```bash
# In WSL terminal
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

rustup default nightly
rustup component add rust-src llvm-tools-preview
rustup target add x86_64-unknown-none
```

#### Step 3: Install QEMU

```bash
sudo apt update
sudo apt install qemu-system-x86
```

#### Step 4: Install Bootimage

```bash
cargo install bootimage
```

#### Step 5: Build and Run

```bash
git clone https://github.com/yourusername/myos.git
cd myos
cargo build --release
cargo bootimage --release

qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/bootimage-myos.bin \
    -netdev user,id=net0 \
    -device e1000,netdev=net0 \
    -serial stdio \
    -m 128M
```

---

### 🍎 **macOS**

#### Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Step 2: Configure Toolchain

```bash
rustup default nightly
rustup component add rust-src llvm-tools-preview
rustup target add x86_64-unknown-none
```

#### Step 3: Install QEMU

```bash
brew install qemu
```

#### Step 4: Install Bootimage

```bash
cargo install bootimage
```

#### Step 5: Build and Run

```bash
git clone https://github.com/yourusername/myos.git
cd myos
cargo build --release
cargo bootimage --release

qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/bootimage-myos.bin \
    -netdev user,id=net0 \
    -device e1000,netdev=net0 \
    -serial stdio \
    -m 128M
```

---

## ✅ Verify Installation

After installing Rust, verify everything is set up correctly:

```bash
# Check Rust version
rustc --version
# Should show: rustc 1.xx.x-nightly

# Check cargo
cargo --version

# Check toolchain
rustup show
# Should show nightly as active

# Check QEMU
qemu-system-x86_64 --version
```

---

## 🚀 First Boot - What to Expect

When MyOS boots, you'll see:

```
╔════════════════════════════════════════════╗
║          MyOS v0.1.0 - Booting...         ║
╚════════════════════════════════════════════╝

[1/13] Initializing GDT...
[2/13] Initializing IDT...
[3/13] Initializing PIC...
[4/13] Initializing memory...
[5/13] Initializing time...
[6/13] Initializing PIT...
[7/13] Initializing process subsystem...
[8/13] Initializing ATA disk driver...
[9/13] Initializing filesystem...
[10/13] Initializing network stack...
E1000: Initializing...
E1000: MMIO base at 0xfebc0000
E1000: MAC address: 52:54:00:12:34:56
E1000: Initialization complete!
  ✓ Network card initialized - MAC: 52:54:00:12:34:56
Network configured:
  IP:      10.0.2.15
  Netmask: 255.255.255.0
  Gateway: 10.0.2.2
  MAC:     52:54:00:12:34:56
  ✓ Network auto-configured for QEMU
    IP: 10.0.2.15 Gateway: 10.0.2.2
[11/13] Enabling interrupts...
[12/13] Creating test tasks...
[13/13] Starting scheduler...

✓ Kernel initialized successfully!

Welcome to MyOS!
>
```

---

## 🎮 Try These Commands

```bash
# Check network status
> ifconfig

# Ping the QEMU gateway (this really works!)
> ping 10.0.2.2

# List files
> ls

# See all 100+ commands
> help

# System information
> sysinfo

# Network statistics
> netstat

# Calculate something
> calc 42 * 42
```

---

## 🐛 Troubleshooting

### "cargo: command not found"
**Solution:** Restart your terminal after installing Rust, or run:
```bash
source $HOME/.cargo/env  # Linux/macOS
```

### "error: toolchain 'nightly' is not installed"
**Solution:**
```bash
rustup toolchain install nightly
rustup default nightly
```

### "QEMU not found"
**Windows:**
```cmd
choco install qemu
```

**Linux:**
```bash
sudo apt install qemu-system-x86  # Ubuntu/Debian
```

**macOS:**
```bash
brew install qemu
```

### Bootimage fails with "target spec" error
This is a known issue with newer Rust nightlies. **Workaround:**

```bash
# Use a compatible nightly
rustup install nightly-2024-03-01
rustup override set nightly-2024-03-01
cargo +nightly-2024-03-01 bootimage --release
```

Or upgrade the bootloader (requires code changes) - see DEVELOPMENT.md.

### "No network in QEMU"
Make sure you're using the correct QEMU flags:
```bash
-netdev user,id=net0 -device e1000,netdev=net0
```

---

## 📚 Next Steps

- **Explore the code**: Start with `src/main.rs` and `src/net/`
- **Read the network stack**: 2,230 lines in `src/net/*` and `src/drivers/e1000.rs`
- **Try commands**: 100+ commands in `src/shell.rs`
- **Write HAL scripts**: See `src/halscript/`
- **Add features**: The TCP/IP stack is ready for UDP sockets and TCP connections!

---

## 🌐 Network Configuration

MyOS auto-configures for QEMU's user-mode networking:

- **IP Address:** 10.0.2.15
- **Gateway:** 10.0.2.2  
- **Netmask:** 255.255.255.0
- **DNS:** 8.8.8.8

**You can ping:**
- `10.0.2.2` - QEMU gateway (always responds)
- `8.8.8.8` - Google DNS (if you have internet)
- `google.com` - Via static DNS resolution

**To change configuration:**
```bash
> ifconfig 192.168.1.100 255.255.255.0 192.168.1.1
```

---

## 🎯 Recommended: WSL2 on Windows

For Windows users, **WSL2 is the easiest path**:

✅ Full Linux environment  
✅ Better performance than native Windows for OS dev  
✅ Seamless integration with Windows  
✅ Access to Linux tools  
✅ Can use Windows tools (VS Code, git) with WSL files  

Just run `wsl --install` in PowerShell and follow the Linux instructions above!

---

## 💡 Quick Start Script

Save this as `run.sh` (Linux/macOS/WSL) or `run.bat` (Windows):

**Linux/macOS/WSL (run.sh):**
```bash
#!/bin/bash
cargo bootimage --release && \
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/release/bootimage-myos.bin \
    -netdev user,id=net0 \
    -device e1000,netdev=net0 \
    -serial stdio \
    -m 128M
```

```bash
chmod +x run.sh
./run.sh
```

**Windows (run.bat):**
```batch
@echo off
cargo bootimage --release
if %ERRORLEVEL% EQU 0 (
    qemu-system-x86_64 ^
        -drive format=raw,file=target\x86_64-unknown-none\release\bootimage-myos.bin ^
        -netdev user,id=net0 ^
        -device e1000,netdev=net0 ^
        -serial stdio ^
        -m 128M
)
```

```cmd
run.bat
```

---

**Need help?** Open an issue on GitHub or check the development docs in DEVELOPMENT.md!

🎉 **Enjoy your bare-metal OS with real networking!** 🌐
