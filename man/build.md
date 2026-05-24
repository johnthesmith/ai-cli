# Build

1. You can build and install `ai` manually.

## Prerequisites

1. Linux (Ubuntu 20.04+ or Debian 11+) — or newer
2. Install Rust and Cargo:
 
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## Clone project

```bash
git clone https://github.com/johnthesmith/ai-cli.git
cd ai-cli
```

## Build

1. Debug build (with debug symbols):

```bush
./make-debug.sh
```

2. Release build (optimized, stripped):

```bash
./make-release.sh
```

## Setup

1. Put `ai` in to `~/.local/bin`.
```bash
mkdir -p ~/.local/bin
cp target/release/ai ~/.local/bin/
```
2. Add to your `~/.bashrc`
```
# AI settings
export PATH="$HOME/.local/bin:$PATH"
ln -sf ~/.local/bin/ai ~/.local/bin/1
complete -W "--help --show-info --show-chat --switch-chat --show-history --clear-history --profile --switch-profile=" 1
complete -W "--help --show-info --show-chat --switch-chat --show-history --clear-history --profile --switch-profile=" ai
```
3. Restart your bash.



## Config

1. Copy config for `default` profile.

```bash
mkdir -p ~/.config/ai/default/
cp -r ./config/* ~/.config/ai/default/
```
2. You can:
    1. use proxy;
    0. send bash commands to keyboard, tty, stdout or file;
    0. change prompt;
    0. switch profile.
3. For all options see `~/.config/ai/default/`.
