/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

pub const DEFAULT: &str = r#"# AI util config file
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
    provider-file: ~/.config/ai/app/cli/%profile%/provider.txt

    # Chat file. This chat contains chat id
    chat-file: ~/.local/share/ai/app/cli/%profile%/chat.txt

    # History file (placeholders: %profile% %provider% %model% %chat%)
    history: ~/.local/share/ai/app/cli/%profile%/history/%chat%.txt

    # Pool path
    pool: ~/.local/share/ai/app/cli/%profile%/pool

    # Memory file for long store data
    memory: ~/.local/share/ai/app/cli/%profile%/memory.txt

    # File with current id prompt (placeholders: %profile% %provider% %model% %caht%)
    prompt-file-id: ~/.local/share/ai/app/cli/%profile%/prompt.txt

    # File with current original prompt (%profile% %provider% %model% %caht%)
    prompt-file: ~/.local/share/ai/app/cli/%profile%/prompts/%prompt-id%.txt

    # Think mode enable for llm deepseek
    think: false

    # Show mnemonic string after LLM answer when using CLI AI operations
    show-mnemonic: true

    # Token file (placeholder: %profile% %provider%)
    token: ~/.config/ai/app/cli/%profile%/tokens/%provider%.txt

    # File with current model
    model: ~/.local/share/ai/app/cli/%profile%/models/%provider%.txt

    # Maximum bytes count for chat prompt
    max-chat-prompt-size-byte: 80000

    # Socks5 proxy url socks5://host:port (optional)
    # proxy: socks5://127.0.0.1:1080

    # Request timeout in milliseconds (total time for the entire request)
    request_timeout_ms: 30000

    # Connection timeout in milliseconds (time to establish connection)
    connect_timeout_ms: 10000

    # Access control for AI operations
    # Each string consists of letters: c (create), r (read), u (update), d (delete)
    # Modes:
    #   normal      : "c"   - AI can only add, not modify or delete
    #   auto-mnemomorph : "cud" - AI can create, update, delete (full control)
    # For normal mode, keep only 'c' to prevent accidental deletions
    # For auto-mnemomorph mode, set "cud" to allow full memory/history editing
    access:
      history: "cud"
      memory: "cud"
      prompt: "c"

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
          - openai/gpt-4.1
          - openai/gpt-4.1-mini
          - openai/gpt-4.1-nano
          - openai/gpt-4o
          - openai/gpt-4o-mini
          - openai/gpt-5
          - openai/gpt-5-chat
          - openai/gpt-5-mini
          - openai/gpt-5-nano
          - openai/o1
          - openai/o1-mini
          - openai/o1-preview
          - openai/o3
          - openai/o3-mini
          - openai/o4-mini
          - openai/text-embedding-3-large
          - openai/text-embedding-3-small
          - cohere/cohere-command-a
          - deepseek/deepseek-r1
          - deepseek/deepseek-r1-0528
          - deepseek/deepseek-v3-0324
          - meta/llama-3.2-11b-vision-instruct
          - meta/llama-3.2-90b-vision-instruct
          - meta/llama-3.3-70b-instruct
          - meta/llama-4-maverick-17b-128e-instruct-fp8
          - meta/llama-4-scout-17b-16e-instruct
          - meta/meta-llama-3.1-405b-instruct
          - meta/meta-llama-3.1-8b-instruct
          - mistral-ai/codestral-2501
          - mistral-ai/ministral-3b
          - mistral-ai/mistral-medium-2505
          - mistral-ai/mistral-small-2503
          - microsoft/phi-4
          - microsoft/phi-4-mini-instruct
          - microsoft/phi-4-mini-reasoning
          - microsoft/phi-4-multimodal-instruct
          - microsoft/phi-4-reasoning
      openai:
        api: https://api.openai.com/v1/chat/completions
        available-models:
          - gpt-4.1
      deepseek:
        api: https://api.deepseek.com/v1/chat/completions
        think: false
        available-models:
          - deepseek-v4-flash
          - deepseek-v4-pro
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
