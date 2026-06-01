#!/usr/bin/env bash
# Installation Script for ai-cli

set -euo pipefail

# Configuration constants
if [[ -d "/data/data/com.termux/files/usr" ]]; then
    # Termux environment
    BIN_DIR="/data/data/com.termux/files/usr/bin"
    CONFIG_DIR="$HOME/.config/ai/default"
else
    # Standard Linux/macOS
    BIN_DIR="$HOME/.local/bin"
    CONFIG_DIR="$HOME/.config/ai/default"
fi

# Helper functions
info() { echo "[INFO] $1"; }
error() { echo "[ERROR] $1" >&2; }
warn() { echo "[WARN] $1"; }

# Create bin directory
mkdir -p "$BIN_DIR"

# Download binary from GitHub Releases
info "Downloading ai binary..."
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$OS" in
    linux)
        case "$ARCH" in
            x86_64)  FILE="ai-linux-x86_64" ;;
            aarch64) FILE="ai-linux-aarch64" ;;
            armv7l|armv8l)  FILE="ai-linux-armv7" ;;
            armv6l)  FILE="ai-linux-armv6" ;;
            *) error "Unsupported arch: $ARCH"; exit 1 ;;
        esac ;;
    darwin)
        case "$ARCH" in
            x86_64)  FILE="ai-macos-intel" ;;
            arm64)   FILE="ai-macos-apple-silicon" ;;
            *) error "Unsupported arch: $ARCH"; exit 1 ;;
        esac ;;
    *)
        error "Unsupported OS: $OS"
        exit 1
        ;;
esac

TAG=$(curl -s "https://api.github.com/repos/johnthesmith/ai-cli/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')
curl -L "https://github.com/johnthesmith/ai-cli/releases/download/$TAG/$FILE" -o "$BIN_DIR/ai"
chmod +x "$BIN_DIR/ai"

# Create symbolic link 1 -> ai
ln -sf "$BIN_DIR/ai" "$BIN_DIR/1"

info "Installation complete."
echo ""
echo "Use for shell completion"
echo "    ai --completion=bash >> ~/.bashrc"
echo "    ai --completion=zsh >> ~/.zshrc"
echo "    ai --completion=fish >> ~/.config/fish/config.fish"
echo "Set tokens: $CONFIG_DIR/tokens/<provider>.txt"
echo "Test: 1 --help"

exit 0
