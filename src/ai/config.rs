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
    # File with current chat id
    # Placeholders: %profile%
    chat-file: ~/.local/share/ai/app/cli/%profile%/chat.txt

    # File with current provider id
    # Placeholders: %profile%
    provider-file: ~/.local/share/ai/app/cli/%profile%/provider.txt

    # File with current model
    # Placeholders: %profile% %chat% %provider%
    model-file: ~/.local/share/ai/app/cli/%profile%/models/%provider%.txt

    # File with current prompt
    # Placeholders: %profile% %caht% %provider% %model%
    prompt-file: ~/.local/share/ai/app/cli/%profile%/prompts/%prompt%.txt

    # History file
    # Placeholders: %profile% %chat% %provider% %model%
    history: ~/.local/share/ai/app/cli/%profile%/history/%chat%.txt

    # Pool path
    # Placeholders: %profile% %chat% %provider% %model%
    pool: ~/.local/share/ai/app/cli/%profile%/pool

    # Memory file for long store data
    # Placeholders: %profile% %chat% %provider% %model%
    memory: ~/.local/share/ai/app/cli/%profile%/memory.txt

    # File with current id prompt
    # Placeholders: %profile% %chat% %provider% %model%
    prompt-file-id: ~/.local/share/ai/app/cli/%profile%/prompt.txt

    # Token file
    # Placeholders: %profile% %chat% %provider% %model%
    token: ~/.config/ai/app/cli/%profile%/tokens/%provider%.txt

    # Shell binary for command execution (default: /bin/bash)
    # Must support '-c' argument (POSIX-compatible).
    shell: /bin/bash

    # Think mode enable for llm deepseek
    think: false

    # Show mnemonic string after LLM answer when using CLI AI operations
    show-mnemonic: true

    # Maximum bytes count for chat prompt
    max-chat-prompt-size-byte: 80000

    # Socks5 proxy url socks5://host:port (optional)
    # proxy: socks5://127.0.0.1:1080

    # Request timeout in milliseconds (total time for the entire request)
    request_timeout_ms: 30000

    # Connection timeout in milliseconds (time to establish connection)
    connect_timeout_ms: 10000

    # Access control for AI operations
    # Each string consists of letters:
    #   insert, select, update, delete
    # Modes for promt:
    #   default: "i" - AI can only add, not modify or delete
    #   automnemomorph : "iud" - AI can create, update, delete (full control)
    access:
      history: "iud"
      memory: "iud"
      prompt: "i"

    # Output destinations
    destination:
      # For mac os:     cliclick t:"%data%"
      # For linux:      xdotool
      # For linux ssh:  ai --tiocsti
      #                 Requires `sudo sysctl -w dev.tty.legacy_tiocsti=1`
      command: "sleep 0.3 && xdotool type --clearmodifiers --delay 10 --file -"
      message: "cat && echo"
      pool: "ai --write-pool"
      clipboard: "xclip -selection clipboard"

    # AI providers configuration
    providers:
      github:
        api: https://models.github.ai/inference/chat/completions
        api-type: openai
        models:
          default: openai/gpt-4.1
          mini: openai/gpt-4.1-mini
          nano: openai/gpt-4.1-nano
          4o: openai/gpt-4o
          4o-mini: openai/gpt-4o-mini
          gpt5: openai/gpt-5
          gpt5chat: openai/gpt-5-chat
          gpt5mini: openai/gpt-5-mini
          gpt5nano: openai/gpt-5-nano
          o1: openai/o1
          o1mini: openai/o1-mini
          o1preview: openai/o1-preview
          o3: openai/o3
          o3mini: openai/o3-mini
          o4mini: openai/o4-mini
          3large: openai/text-embedding-3-large
          3small: openai/text-embedding-3-small
          "cohere/cohere-command-a": cohere/cohere-command-a
          "deepseek/deepseek-r1": deepseek/deepseek-r1
          "deepseek/deepseek-r1-0528": deepseek/deepseek-r1-0528
          "deepseek/deepseek-v3-0324": deepseek/deepseek-v3-0324
          "meta/llama-3.2-11b-vision-instruct": meta/llama-3.2-11b-vision-instruct
          "meta/llama-3.2-90b-vision-instruct": meta/llama-3.2-90b-vision-instruct
          "meta/llama-3.3-70b-instruct": meta/llama-3.3-70b-instruct
          "meta/llama-4-maverick-17b-128e-instruct-fp8": meta/llama-4-maverick-17b-128e-instruct-fp8
          "meta/llama-4-scout-17b-16e-instruct": meta/llama-4-scout-17b-16e-instruct
          "meta/meta-llama-3.1-405b-instruct": meta/meta-llama-3.1-405b-instruct
          "meta/meta-llama-3.1-8b-instruct": meta/meta-llama-3.1-8b-instruct
          "mistral-ai/codestral-2501": mistral-ai/codestral-2501
          "mistral-ai/ministral-3b": mistral-ai/ministral-3b
          "mistral-ai/mistral-medium-2505": mistral-ai/mistral-medium-2505
          "mistral-ai/mistral-small-2503": mistral-ai/mistral-small-2503
          "microsoft/phi-4": microsoft/phi-4
          "microsoft/phi-4-mini-instruct": microsoft/phi-4-mini-instruct
          "microsoft/phi-4-mini-reasoning": microsoft/phi-4-mini-reasoning
          "microsoft/phi-4-multimodal-instruct": microsoft/phi-4-multimodal-instruct
          "microsoft/phi-4-reasoning": microsoft/phi-4-reasoning

      openai:
        api: https://api.openai.com/v1/chat/completions
        api-type: openai
        models:
          default: gpt-4.1

      deepseek:
        api: https://api.deepseek.com/v1/chat/completions
        api-type: openai
        models:
          default: deepseek-v4-flash
          pro: deepseek-v4-pro

      groq:
        api: https://api.groq.com/openai/v1/chat/completions
        api-type: openai
        models:
          default: llama-3.3-70b-versatile
          instruct: llama-4-scout-17b-16e-instruct
          mixtral: mixtral-8x7b-32768

      together:
        api: https://api.together.xyz/v1/chat/completions
        models:
          default: meta-llama/Llama-3.3-70B-Instruct-Turbo

      ollama:
        api: http://localhost:11434/api/generate
        models:
          default: qwen3.5:9b
          llama: llama3.2

      anthropic:
        api: https://api.anthropic.com/v1/messages
        api-type: openai
        models:
          default: claude-3-5-sonnet-20241022

    # Specific request contract

    api-format:
    -
      # Deepseek rules
      provider: "deepseek"
      model: "*"

      request:
        model: "%model-name%"
        messages:
        -
          content: "%prompt%"
          role: user
        thinking:
          type: disabled
      answer: [ choices, 0, message, content ]

    -
      # ollama rules
      provider: ollama
      model: "*"

      request:
        model: "%model-name%"
        messages:
        -
          content: "%prompt%"
          role: user
        stream: false
      answer: [ message, content ]

    -
      # Default rules
      provider: "*"
      model: "*"

      request:
        model: "%model-name%"
        messages:
        -
          content: "%prompt%"
          role: user
      answer: [ choices, 0, message, content ]

"#;
