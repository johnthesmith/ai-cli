/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

pub const DEFAULT: &str = r#"
# AI util config file
application:
  # Log settings
  log:
    file: ~/.local/share/ai/app/cli/%profile%/log.txt
    enabled: true

  # AI application
  ai:
    # Shell binary for command execution (default: /bin/bash)
    # Must support '-c' argument (POSIX-compatible).
    shell: /bin/bash

    # Providers file
    provider_file: ~/.config/ai/app/cli/%profile%/provider.txt

    # Chat file. This chat contains chat id
    chat-file: ~/.local/share/ai/app/cli/%profile%/chat.txt

    # History file (placeholders: %profile% %provider% %model% %chat%)
    history: ~/.local/share/ai/app/cli/%profile%/history/%chat%.txt

    # Pool path
    pool: ~/.local/share/ai/app/cli/%profile%/pool

    # Memory file for long store data
    memory: ~/.local/share/ai/app/cli/%profile%/memory/%chat%.txt

    # Token file (placeholder: %profile% %provider%)
    token: ~/.config/ai/app/cli/%profile%/tokens/%provider%.txt

    # File with current model
    model: ~/.local/share/ai/app/cli/%profile%/models/%provider%.txt

    # Maximum bytes count for chat prompt
    max-chat-prompt-size-byte: 100000

    # Socks5 proxy url socks5://host:port (optional)
    # proxy: socks5://127.0.0.1:1080

    # Request timeout in milliseconds (total time for the entire request)
    request_timeout_ms: 30000

    # Connection timeout in milliseconds (time to establish connection)
    connect_timeout_ms: 10000

    # AI prompts (placeholders: %profile% %provider% %model%)
    prompts:
      chat: ~/.config/ai/app/cli/%profile%/prompts/chat.txt
      summary: ~/.config/ai/app/cli/%profile%/prompts/summary.txt

    # Output destinations
    destination:
      command: "sleep 0.3 && xdotool type --clearmodifiers --delay 10 --file -"
      message: "cat && echo"
      pool: "ai --write-pool"
      clipboard: "xclip -selection clipboard"

    # AI providers configuration
    providers:

      github:
        api: https://models.github.ai/inference/chat/completions
        available-models:
          - openai/gpt-4o
          - openai/gpt-4.1
          - openai/gpt-4.1-mini

      openai:
        api: https://api.openai.com/v1/chat/completions
        available-models:
          - gpt-4.1

      deepseek:
        api: https://api.deepseek.com/v1/chat/completions
        available-models:
          - deepseek-chat

      groq:
        api: https://api.groq.com/openai/v1/chat/completions
        available-models:
          - llama-3.3-70b-versatile
          - llama-4-scout-17b-16e-instruct
          - mixtral-8x7b-32768

      together:
        api: https://api.together.xyz/v1/chat/completions
        available-models:
          - meta-llama/Llama-3.3-70B-Instruct-Turbo

      ollama:
        api: http://localhost:11434/api/generate
        available-models:
          - qwen3.5:9b
          - llama3.2

      anthropic:
        api: https://api.anthropic.com/v1/messages
        available-models:
          - claude-3-5-sonnet-20241022
"#;
