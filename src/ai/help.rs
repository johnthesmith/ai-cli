/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

pub const CONTENT: &str = r#"%version%

Usage:
    ai                              Interactive keyboard input
    ai <text>                       Ask a question
    echo <text> | ai                Read from stdin
    ai --help                       Show this help

Pattern:
    ai [hellow] [world] [--<action>=<key>][--<argument>=<value>]

Options:
    --help                  -?|-h   Same as --show=help
    --info                  -i      Same as --show=info
    --version               -v      Same as --show=version
    --no-prompt                     Suppress input user prompt
    --no-shell                      Suppress shell event

Session:
    --profile=<id>                  Use profile for current session only
    --provider=<id>         -p      Use provider for current session only
    --model=<id>            -m      Use model by alias for current session only
    --chat|=<id>            -c      Use chat for current session only
    --prompt=<id>                   Use prompt for current session only
    --switch-profile=<id>           Permanently switch profile
    --switch-provider=<id>          Permanently switch provider
    --switch-model=<id>             Permanently switch model by alias
    --switch-chat=<id>              Permanently switch chat
    --switch-prompt=<id>            Permanently switch prompt:
                                    default|automnemomorf|...

Storage operations:
    --out-history           -oh     Out history
    --out-memory            -om     Out memory
    --out-prompt            -op     Out prompt
    --out-prompt-original   -opo    Out original prompt
    --reset-history         -rh     Reset history content for current chat
    --reset-memory          -rm     Reset memory content for current chat
    --select-histroy=<id>   -sh     Show fact by id from history
    --select-memory=<id>    -sm     Show fact by id from memory
    --delete-history=<id>   -dh     Delete fact by id from history
    --delete-memory=<id>    -dm     Delete fact by id from memory
    --update-history=<id>   -uh     Update fact by id in history
                                    content can put from stdin
    --update-memory=<id>    -u      Update fact by id in memory
                                    content can put from stdin
    --insert-history=<txt>  -ih     Insert new fact into history
                                    content can put from stdin
    --insert-memory=<txt>   -im     Insert new fact into memory
                                    content can put from stdin
    --actor=<actor>                 Actor for insert/update (default: assistant)
    --body=<text>                   Body for insert/update (or from stdin)

Specific:
    --write-pool                    Write stdin to pool file and forward to
                                    stdout
                                    Example: echo 'data' | ai --write-pool
    --tiocsti                       Inject input directly into TTY input buffer
                                    for keyboard.
                                    Requires on modern kernels:
                                    `sudo sysctl -w dev.tty.legacy_tiocsti=1`
                                    Only use in trusted environments.
                                    Example: echo 'ls -la' | ai --tiocsti
    --completion=<shell>            Generate shell completion (bash|zsh|fish)
                                    Example: ai --completion=bash >> ~/.bashrc

Recommendations:
    Set `alias 1=ai`

Author:
    Still Swamp (still@catlair.net) powered by deepseek
"#;
