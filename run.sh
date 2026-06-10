#!/bin/bash
# Simple script to run MyOS in QEMU

# Build the kernel
echo "Building MyOS kernel..."
cargo build --release

# Create a simple bootable disk image
echo "Creating bootable image..."
mkdir -p target/boot
cp target/x86_64-unknown-none/release/myos target/boot/kernel.bin

# Run in QEMU with options for debugging
echo "Starting QEMU..."
qemu-system-x86_64 \
    -kernel target/boot/kernel.bin \
    -serial stdio \
    -display gtk \
    -m 128M \
    -cpu qemu64 \
    -smp 1 \
    -no-reboot \
    -no-shutdown
