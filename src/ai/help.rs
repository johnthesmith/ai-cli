/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

pub const CONTENT: &str = r#"%version%

Usage:
    ai                          Interactive keyboard input
    ai <question>               Ask a question
    echo <text> | ai            Read from stdin
    ai --help                   Show this help
Pattern:
    ai [hellow] [world] [--<action>=<key>][--<argument>=<value>]

Options:
    --help|-?|-h                Same as --show=help
    --info|-h                   Same as --show=info
    --version|-v                Same as --show=version

    --no-prompt                 Suppress input user prompt
    --no-command                Suppress command event

Session:
    --profile=<id>              Use profile for current session only
    --provider|-p=<id>          Use provider for current session only
    --model|-m=<id>             Use model for current session only
    --chat|-c=<id>              Use chat for current session only
    --prompt=<id>               Use prompt for current session only
    --switch-profile=<id>       Permanently switch profile
    --switch-provider=<id>      Permanently switch provider
    --switch-model=<id>         Permanently switch model
    --switch-chat=<id>          Permanently switch chat
    --switch-prompt=<id>        Permanently switch prompt:
                                default|automnemomorf|...
;
Access for LLM;
    --access-history=<mode>     Set history access rights
                                    c=create,
                                    u=update,
                                    d=delete
                                    Example: --access-history=cud
    --access-memory=<mode>      Set memory access rights
                                    c=create,
                                    u=update,
                                    d=delete
                                    Example: --access-memory=cud

Storage operations with target history|memory|prompt
    --out-history|-oh           Out history for chat
    --out-memory|-om            Out memory
    --out-prompt|-op            Out prompt
    --out-prompt-origin|-opo    Out original prompt
    --clear-history|-ch         Remove history content for current chat
    --clear-memory|-cm          Remove memory content for current chat
    --select-histroy=<id>       Show fact by id from history
    --select-memory=<id>        Show fact by id from memory
    --delete-history=<id>       Delete fact by id from history
    --delete-memory=<id>        Delete fact by id from memory
    --update-history=<id>       Update fact by id in history
    --update-memory=<id>        Update fact by id in memory
    --insert-history=<content>  Insert new fact into history
                                content can put from stdin
    --insert-memory=<id>        Insert new fact into memory
    --actor=<actor>             Actor for insert/update (default: assistant)
    --body=<text>               Body for insert/update (or from stdin)

Specific:
    --write-pool                Write stdin to pool file and forward to stdout
                                Example: echo 'data' | ai --write-pool
    --tiocsti                   Inject input directly into TTY input buffer for
                                keyboard.
                                Requires:
                                     `sudo sysctl -w dev.tty.legacy_tiocsti=1`
                                on modern kernels.
                                Only use in trusted environments.
                                Example: echo 'ls -la' | ai --tiocsti

    --completion=<shell>        Generate shell completion (bash|zsh|fish)
                                Example: ai --completion=bash >> ~/.bashrc

Recommendations:
    alias                       Set `alias 1=ai`

Author:
    Still Swamp (still@catlair.net) powered by deepseek
"#;
