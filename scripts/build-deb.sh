#!/bin/bash
set -e

echo "Building CC-Switch TUI for ARM64 (Debian 10/11)..."

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo "Installing cross..."
    cargo install cross --git https://github.com/cross-rs/cross
fi

# Build for ARM64 using cross
echo "Building release binary for ARM64..."
cross build --release --target aarch64-unknown-linux-gnu

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
# Create a proper binary package control file
cat > debian-pkg/DEBIAN/control << 'EOF'
Package: cc-switch-tui
Version: 3.11.1
Architecture: arm64
Maintainer: Jason Young <jason@example.com>
Depends: libc6, libgcc-s1
Section: net
Priority: optional
Homepage: https://github.com/farion1231/cc-switch
Description: All-in-One Assistant for Claude Code, Codex & Gemini CLI
 Terminal UI version for managing AI provider proxies with automatic
 failover, circuit breaker, and MCP server integration.
 .
 Features:
  - HTTP proxy server for Claude Code, Codex, and Gemini CLI
  - Automatic failover between multiple providers
  - Circuit breaker for fault tolerance
  - MCP (Model Context Protocol) server management
  - Provider configuration and switching
  - Usage statistics and monitoring
  - Terminal-based user interface with keyboard navigation
EOF
cp debian/postinst debian-pkg/DEBIAN/
cp debian/prerm debian-pkg/DEBIAN/
chmod +x debian-pkg/DEBIAN/postinst
chmod +x debian-pkg/DEBIAN/prerm

# Build .deb package
echo "Building .deb package..."
dpkg-deb --build --root-owner-group -Zgzip debian-pkg cc-switch-tui_3.11.1_arm64.deb

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
