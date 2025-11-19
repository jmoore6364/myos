.PHONY: build run clean test usb help

# Default target
all: build

# Build the OS
build:
	@echo "Building MyOS..."
	cargo bootimage

# Run in QEMU
run:
	@echo "Starting MyOS in QEMU..."
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-myos/debug/bootimage-myos.bin -serial stdio

# Run in QEMU with KVM acceleration (Linux only)
run-kvm:
	@echo "Starting MyOS in QEMU with KVM..."
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-myos/debug/bootimage-myos.bin -serial stdio -enable-kvm

# Run in release mode
run-release: build-release
	@echo "Starting MyOS (Release) in QEMU..."
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-myos/release/bootimage-myos.bin -serial stdio

# Build release version
build-release:
	@echo "Building MyOS (Release)..."
	cargo bootimage --release

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Create VirtualBox VDI image
vdi: build
	@echo "Creating VirtualBox VDI image..."
	qemu-img convert -f raw -O vdi target/x86_64-myos/debug/bootimage-myos.bin myos.vdi
	@echo "VDI created: myos.vdi"

# Write to USB drive (BE CAREFUL!)
usb: build
	@echo "⚠️  WARNING: This will erase the USB drive!"
	@read -p "Enter USB device path (e.g., /dev/sdb): " device && \
	sudo dd if=target/x86_64-myos/debug/bootimage-myos.bin of=$$device bs=4M status=progress && sync
	@echo "✓ USB drive created successfully"

# Show help
help:
	@echo "MyOS Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  make build         - Build the OS"
	@echo "  make run           - Run in QEMU"
	@echo "  make run-kvm       - Run in QEMU with KVM (faster, Linux only)"
	@echo "  make run-release   - Run optimized release build"
	@echo "  make test          - Run tests"
	@echo "  make vdi           - Create VirtualBox VDI image"
	@echo "  make usb           - Write to USB drive (⚠️  destructive!)"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make help          - Show this help"

# Install dependencies
install-deps:
	@echo "Installing build dependencies..."
	rustup default nightly
	rustup component add rust-src llvm-tools-preview
	cargo install bootimage
	@echo "✓ Dependencies installed"
	@echo ""
	@echo "Don't forget to install QEMU:"
	@echo "  Ubuntu/Debian: sudo apt install qemu-system-x86"
	@echo "  macOS: brew install qemu"
	@echo "  Windows: choco install qemu"
