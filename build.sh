#!/usr/bin/env bash
# ============================================
# Installation Script for ai-cli
# ============================================
#
# What this script does:
# ======================
#
# 1. Checks if running on Linux (required OS)
# 2. Checks for Rust/Cargo installation:
#    - If missing, installs Rust via rustup
#    - If present, skips installation
# 3. Clones the ai-cli repository to a temporary directory
# 4. Builds the release version using ./make-release.sh
# 5. Creates ~/.local/bin directory if it doesn't exist
# 6. Copies the compiled binary to ~/.local/bin/ai
# 7. Creates symlink ~/.local/bin/1 -> ai
# 8. Configures bash autocompletion in .bashrc:
#    - Adds export PATH=... if not already present
#    - Adds completion for 'ai' and '1' commands
#    - Uses marker '# AI settings' to avoid duplicates
# 9. Copies configuration files to ~/.config/ai/default/:
#    - Preserves existing files (does NOT overwrite)
#    - Only creates missing files (e.g., token.txt is kept)
# 10. Cleans up temporary directory
# 11. Displays next steps: GitHub token setup and testing
#
# Notes:
# - No sudo required - everything installs to $HOME
# - Existing user configurations are respected
# - Safe to run multiple times (idempotent)
#
# ============================================

set -euo pipefail

# ============================================
# Configuration constants
# ============================================
REPO_URL="https://github.com/johnthesmith/ai-cli.git"
CLONE_DIR="$HOME/tmp/ai-cli-install"
BIN_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/ai/default"
BASHRC="$HOME/.bashrc"

# Completion options (split to keep lines under 80 chars)
# Completion options for ai and 1 commands
COMPLETION_OPTIONS='--help'
COMPLETION_OPTIONS+=' --no-prompt'
COMPLETION_OPTIONS+=' --no-command'
COMPLETION_OPTIONS+=' --show-info'
COMPLETION_OPTIONS+=' --profile='
COMPLETION_OPTIONS+=' --switch-profile='
COMPLETION_OPTIONS+=' --switch-chat='
COMPLETION_OPTIONS+=' --pack-history'
COMPLETION_OPTIONS+=' --show-history'
COMPLETION_OPTIONS+=' --clear-history'
COMPLETION_OPTIONS+=' --profile='
COMPLETION_OPTIONS+=' --switch-profile='
COMPLETION_OPTIONS+=' --provider='
COMPLETION_OPTIONS+=' --switch-provider='
COMPLETION_OPTIONS+=' --clear-memory'
COMPLETION_OPTIONS+=' --show-memory'
COMPLETION_OPTIONS+=' --write-buffer'
COMPLETION_OPTIONS+=' --tiocsti'
COMPLETION_LINE="complete -W \"$COMPLETION_OPTIONS\" 1"
COMPLETION_LINE_AI="complete -W \"$COMPLETION_OPTIONS\" ai"



# ============================================
# Color codes for output
# ============================================
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color



# ============================================
# Helper functions
# ============================================
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}



# ============================================
# Main installation script
# ============================================

# Check OS
if [[ "$(uname)" != "Linux" ]]; then
    error "This script is intended for Linux only (Ubuntu 20.04+ / Debian 11+)"
    exit 1
fi



# Check for Rust and Cargo
if ! command -v cargo &> /dev/null; then
    info "Rust not found. Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    info "Rust is already installed"
fi



# Clone repository
if [[ -d "$CLONE_DIR" ]]; then
    warn "Directory $CLONE_DIR already exists. Removing..."
    rm -rf "$CLONE_DIR"
fi



info "Cloning repository $REPO_URL to $CLONE_DIR"
git clone "$REPO_URL" "$CLONE_DIR"
cd "$CLONE_DIR"



# Build release version
info "Building release version..."
./make-release.sh



# Create ~/.local/bin directory
mkdir -p "$BIN_DIR"



# Copy binary file
info "Copying ai to $BIN_DIR"
cp target/release/ai "$BIN_DIR/"



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



# Create configuration directory if it doesn't exist
mkdir -p "$CONFIG_DIR"



# Copy configuration files (only if missing)
info "Creating configuration in $CONFIG_DIR (preserving existing files)"

# Copy all files and directories from config template (preserving existing)
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


source ~/.bashrc


# GitHub token setup info
info "Installation complete!"
echo ""
echo "==================================================="
echo "Next steps:"
echo "1. Reload bash or run: source ~/.bashrc"
echo "2. Set up GitHub token (see README for instructions):"
echo "   - Go to https://github.com/settings/personal-access-tokens"
echo "   - Create a fine-grained token with access to public repositories"
echo "     and 'Models' -> 'Read-only' permission"
echo "   - Save the token to file: $CONFIG_DIR/token.txt"
echo "3. Test the installation: 1 --help"
echo "==================================================="

exit 0
