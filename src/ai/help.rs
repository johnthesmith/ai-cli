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

Start:
    --init                          Initialise ai in the current directory

Sets:
    --set=<name>                    Apply preset from config keys:
                                    application.sets.<name>
Options:
    --help                  -?|-h   Same as --show=help
    --info                  -i      Same as --show=info
    --version               -v      Same as --show=version
    --no-user-prompt                Suppress input user prompt
    --no-shell                      Suppress shell event
    --no-clipboard                  Suppress clipboard operation
    --no-histroy                    Suppress histroy operations
    --no-memory                     Suppress memory operations
    --no-prompt                     Suppress prompt operations
    --out-status                    Out status indicator after answer (true)
    --read=<file1>[,<file2>,... ]   Send readable file (read only for llm)
    --write=<file1>[,<file2>,... ]  Send writeable file (read and write for llm)
    --compile-prompt=<id>           Compile prompt file from config `prompts`
                                    New prompt will be placed to current prompt
                                    --bind-prompt or --prompt

Session:
    --profile=<id>                  Use profile for current session only
    --chat=<id>             -c      Use chat for current session only
    --provider=<id>         -p      Use provider for current session only
    --model=<id>            -m      Use model by alias for current session only
    --memory=<id>                   Use memory by alias for current session only
    --prompt=<id>                   Use prompt for current session only

    --bind-profile=<id>             Permanently switch profile for chat
    --bind-chat=<id>                Permanently switch chat
    --bind-provider=<id>            Permanently switch provider for chat
    --bind-model=<id>               Permanently switch model by alias for chat
    --bind-memory=<id>              Permanently switch memory for chat
    --bind-prompt=<id>              Permanently switch prompt

Storage operations:
    --out-history           -oh     Out the history
    --out-memory            -om     Out the memory
    --out-prompt            -op     Out the prompt
    --out-prompt-content    -opc    Out content of the prompt
    --reset-history         -rh     Reset history content for current chat
    --reset-memory          -rm     Reset memory content for current chat
                            -rmh    Reset memory&history for current chat
    --select-histroy=<id>   -sh     Show fact by id from history
    --select-memory=<id>    -sm     Show fact by id from memory
    --delete-history=<id>   -dh     Delete fact by id from history
    --delete-memory=<id>    -dm     Delete fact by id from memory
    --update-history=<id>   -uh     Update fact by id in history from stdin
    --update-memory=<id>    -u      Update fact by id in memory from stdin
    --insert-history=<txt>  -ih     Insert new fact into history from stdin
    --insert-memory=<txt>   -im     Insert new fact into memory from stdin
    --actor=<actor>                 Actor for insert/update (default: assistant)
    --body=<text>                   Body for insert/update (or from stdin)

Specific:
    --tiocsti                       Inject input directly into TTY input buffer
                                    for keyboard.
                                    Requires on modern kernels:
                                    `sudo sysctl -w dev.tty.legacy_tiocsti=1`
                                    Only use in trusted environments.
                                    Example: echo 'ls -la' | ai --tiocsti

Recommendations:
    Set `alias 1=ai`

Author:
    Still Swamp (still@catlair.net) powered by deepseek
"#;
