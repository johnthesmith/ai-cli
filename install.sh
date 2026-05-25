#!/usr/bin/env bash
# Installation Script for ai-cli

set -euo pipefail

# Configuration constants
REPO_URL="https://github.com/johnthesmith/ai-cli.git"
CLONE_DIR="$HOME/tmp/ai-cli-install"
BIN_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/ai/default"
BASHRC="$HOME/.bashrc"

# Completion options for ai and 1 commands
COMPLETION_OPTIONS='--help'
COMPLETION_OPTIONS+=' --no-prompt'
COMPLETION_OPTIONS+=' --no-command'
COMPLETION_OPTIONS+=' --show-info'
COMPLETION_OPTIONS+=' --profile='
COMPLETION_OPTIONS+=' --switch-profile='
COMPLETION_OPTIONS+=' --switch-chat='
COMPLETION_OPTIONS+=' --pack-history='
COMPLETION_OPTIONS+=' --show-history'
COMPLETION_OPTIONS+=' --clear-history'
COMPLETION_OPTIONS+=' --provider='
COMPLETION_OPTIONS+=' --switch-provider='
COMPLETION_OPTIONS+=' --clear-memory'
COMPLETION_OPTIONS+=' --show-memory'
COMPLETION_OPTIONS+=' --write-buffer'
COMPLETION_OPTIONS+=' --tiocsti'
COMPLETION_LINE="complete -W \"$COMPLETION_OPTIONS\" 1"
COMPLETION_LINE_AI="complete -W \"$COMPLETION_OPTIONS\" ai"

# ============================================
# Helper functions
# ============================================
info() {
    echo "[INFO] $1"
}

error() {
    echo "[ERROR] $1" >&2
}

warn() {
    echo "[WARN] $1"
}

# ============================================
# Main installation script
# ============================================

# Clone repository
if [[ -d "$CLONE_DIR" ]]; then
    warn "Directory $CLONE_DIR already exists. Removing..."
    rm -rf "$CLONE_DIR"
fi

info "Cloning repository $REPO_URL to $CLONE_DIR"
git clone "$REPO_URL" "$CLONE_DIR"
cd "$CLONE_DIR"

# Create ~/.local/bin directory
mkdir -p "$BIN_DIR"

# Download binary from GitHub Releases
info "Downloading ai binary..."
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$OS" in
    linux)
        case "$ARCH" in
            x86_64)  FILE="ai-linux" ;;
            aarch64) FILE="ai-linux-arm64" ;;
            *) error "Unsupported arch: $ARCH"; exit 1 ;;
        esac ;;
    darwin)
        case "$ARCH" in
            x86_64)  FILE="ai-macos-intel" ;;
            arm64)   FILE="ai-macos-apple-silicon" ;;
            *) error "Unsupported arch: $ARCH"; exit 1 ;;
        esac ;;
    *) error "Unsupported OS: $OS"; exit 1 ;;
esac

TAG=$(curl -s "https://api.github.com/repos/johnthesmith/ai-cli/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')
curl -L "https://github.com/johnthesmith/ai-cli/releases/download/$TAG/$FILE" -o "$BIN_DIR/ai"
chmod +x "$BIN_DIR/ai"

# Create symbolic link 1 -> ai
if [[ -L "$BIN_DIR/1" ]] || [[ -e "$BIN_DIR/1" ]]; then
    warn "File/link $BIN_DIR/1 already exists. Overwriting..."
    rm -f "$BIN_DIR/1"
fi
ln -s "$BIN_DIR/ai" "$BIN_DIR/1"

# Setup autocompletion in .bashrc
if grep -q "# AI settings" "$BASHRC" 2>/dev/null; then
    info "AI settings already configured in .bashrc"
else
    info "Adding AI settings to .bashrc"
    {
        echo ""
        echo "# AI settings"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo "$COMPLETION_LINE"
        echo "$COMPLETION_LINE_AI"
    } >> "$BASHRC"
fi

# Create configuration directory
mkdir -p "$CONFIG_DIR"

# Copy configuration files (only if missing)
info "Creating configuration in $CONFIG_DIR (preserving existing files)"

for file in "$CLONE_DIR/config/"*; do
    filename=$(basename "$file")
    target="$CONFIG_DIR/$filename"

    if [[ -d "$file" ]]; then
        if [[ ! -d "$target" ]]; then
            cp -r "$file" "$target"
            info "Created directory: $filename"
        else
            for subfile in "$file/"*; do
                subfilename=$(basename "$subfile")
                subtarget="$target/$subfilename"
                if [[ ! -e "$subtarget" ]]; then
                    cp "$subfile" "$subtarget"
                    info "Created: $filename/$subfilename"
                fi
            done
        fi
    else
        if [[ ! -f "$target" ]]; then
            cp "$file" "$target"
            info "Created: $filename"
        fi
    fi
done

# Clean up temporary directory
info "Cleaning up temporary build files..."
rm -rf "$CLONE_DIR"

info "Installation complete!"
echo ""
echo "==================================================="
echo "Next steps:"
echo "1. Reload bash or run: source ~/.bashrc"
echo "2. Test the installation: 1 --help"
echo "==================================================="

exit 0
