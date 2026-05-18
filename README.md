# AI Utility

1. CLI utility designed for embedding AI into bash pipelines
2. Usage:
    1. `echo "hello world" | ai` - pipeline;
    2. `ai create hello-world folder` - direct query to AI;
    3. `ai` - interactive query input;
    4. `ai --help` - information.



# Config

1. Files must be placed in the `~/.config/local/ai/default/` directory:
    1. `config.yaml` - main configuration file;
    2. `prompt.txt` - system prompt template;
    3. `token.txt` - GitHub token file;



# Usage Information

```
--help - information
--no-prompt - Suppress input prompt
--show-runtime - Show current runtime values (profile, chat, log, config)
--show-chat - Show current chat id
--switch-chat=<id> - Switch to chat <id>, default id is default
--show-history - Show history for current chat
--clear-history - Remove history for current chat
--profile=<name> - Use profile for current session only
--switch-profile=<name> - Switch and save profile
```



# Recommendations

1. Set alias `1=<path to ai>`
2. This allows you to simply use commands like `1 create hello world`



# Supported AI Providers

1. Currently `ai` works with the following providers:
    1. github



# Build Instructions

1. You can build and install `ai` manually.

## Prerequisites

1. Linux (Ubuntu 20.04+ or Debian 11+) — or newer
2. Install system dependencies
```bash
sudo apt install libxdo-dev pkg-config
```
3. Install Rust and Cargo:
 
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
cp target/release/ai ~/.local/bin/ai
alias 1='~/.local/bin/ai'
alias ai='~/.local/bin/ai'
```

## Config

1. Copy config for `default` profile.

```bash
cp -r ./config/* ~/.config/local/ai/default/
```

## GitHub Token Setup

1. Go to: https://github.com/settings/personal-access-tokens
2. Click **Generate new token** → **Fine-grained token**
3. Fill:
   - **Token name**: `ai-cli`
   - **Expiration**: 90 days
   - **Resource owner**: your account
4. **Repository access**: `Public Repositories (read-only)`
5. **Permissions** → **Models** → `Read-only`
6. Click **Generate token**
7. **Copy token immediately** (shown only once)
8. Put your github token here `~/.config/local/ai/default/token.txt`

## Run

```bash
1 your question
```


