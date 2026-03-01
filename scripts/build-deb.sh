#!/bin/bash
set -e

echo "Building CC-Switch TUI for ARM64 (Debian 10/11)..."

# Check if cross-compilation toolchain is installed
if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
    echo "Error: ARM64 cross-compilation toolchain not found"
    echo "Please install: sudo apt-get install gcc-aarch64-linux-gnu"
    exit 1
fi

# Add ARM64 target if not already added
rustup target add aarch64-unknown-linux-gnu || true

# Build for ARM64
echo "Building release binary for ARM64..."
cargo build --release --target aarch64-unknown-linux-gnu

# Create debian package structure
echo "Creating debian package structure..."
rm -rf debian-pkg
mkdir -p debian-pkg/usr/bin
mkdir -p debian-pkg/DEBIAN

# Copy binary
echo "Copying binary..."
cp target/aarch64-unknown-linux-gnu/release/cc-switch-tui debian-pkg/usr/bin/
chmod +x debian-pkg/usr/bin/cc-switch-tui

# Copy control files
echo "Copying control files..."
cp debian/control debian-pkg/DEBIAN/
cp debian/postinst debian-pkg/DEBIAN/
cp debian/prerm debian-pkg/DEBIAN/
chmod +x debian-pkg/DEBIAN/postinst
chmod +x debian-pkg/DEBIAN/prerm

# Build .deb package
echo "Building .deb package..."
dpkg-deb --build debian-pkg cc-switch-tui_3.11.1_arm64.deb

# Get package info
echo ""
echo "Package built successfully!"
echo "File: cc-switch-tui_3.11.1_arm64.deb"
echo ""
echo "Package info:"
dpkg-deb -I cc-switch-tui_3.11.1_arm64.deb

echo ""
echo "To install on Radxa A7Z:"
echo "  1. Transfer: scp cc-switch-tui_3.11.1_arm64.deb radxa@radxa-a7z:/tmp/"
echo "  2. Install:  ssh radxa@radxa-a7z 'sudo dpkg -i /tmp/cc-switch-tui_3.11.1_arm64.deb'"
echo "  3. Run:      ssh radxa@radxa-a7z 'cc-switch-tui'"
