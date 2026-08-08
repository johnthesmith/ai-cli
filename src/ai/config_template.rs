/*
    SPDX-License-Identifier: MIT
    SPDX-FileCopyrightText: 2026 Still Swamp
*/

const CONFIG: &str = r#"
# ******************************************************************************
# Set keys are applied as CLI flags. Any key may be overridden by
# command-line arguments. Keys may also be refined via application.ai.rules
# matching the current provider and model.

# File with current chat id
# Placeholders: %profile%
chat-file: "%profile-path%/chat.txt"

# File with current provider id
# Placeholders: %profile-path% %profile% %chat%
provider-file: "%profile-path%/chats/%chat%/provider.txt"

# File with current model
# Placeholders: %profile-path% %profile% %chat% %provider%
model-file: "%profile-path%/chats/%chat%/models/%provider%.txt"

# File with current id prompt
# Placeholders: %chat-path% %profile-path% %profile% %chat% %provider% %model%
prompt-file-id: "%chat-path%/prompt.txt"

# File with current prompt
# Placeholders: %profile-path% %profile% %chat% %provider% %model%
prompt-file: "%profile-path%/chats/%chat%/prompts/%prompt%.txt"

# History file
# Placeholders: %profile-path% %profile% %chat% %provider% %model%
history: "%profile-path%/chats/%chat%/history.txt"

# Memory for current chat
# Placeholders: %profile-path% %profile% %chat% %provider% %model%
memory-of-chat-file: "%profile-path%/chats/%chat%/memory.txt"

# Memory file
# Placeholders: %profile-path% %profile% %chat% %provider% %model%
memory-file: "%profile-path%/memory/%memory-id%.txt"

# Token file
# Placeholders: %profile-path% %profile% %chat% %provider% %model%
token: ~/.config/ai/app/cli/%profile%/tokens/%provider%.txt

# Shell binary for command execution (default: /bin/bash)
# Must support '-c' argument (POSIX-compatible).
shell: /bin/bash

# Think mode enable for llm deepseek
think: false

# Show mnemonic status string after LLM answer when using CLI AI operations
out-status: true

# Maximum bytes count for chat prompt
max-prompt-bytes: 200000

# Socks5 proxy url socks5://host:port (optional)
# proxy: socks5://127.0.0.1:1080

# Request timeout in milliseconds (total time for the entire request)
request-timeout-ms: 1200000

# Connection timeout in milliseconds (time to establish connection)
connect-timeout-ms: 10000

# Color support or --color=true
color: true

# You can build your prompt from facts
# ai --build-prompt=default or other...
prompts:
  # This is standart prompt.
  default:
    facts:
      - protocol
      - answer-protocol
      - answer-rules
      - history-add
      - history-change
      - history-remove
      - history-pack
      - memory-add
      - shell-add
      - clipboard-add
      - read
      - write
      - file-context
      - domain-permissions
      - user-env
    # Access control for AI
    # Each string consists of letters: [i]nsert, [s]elect, [u]pdate, [d]elete
    # For empty rights use "."
    access:
      access: "s"
      history: "siud"
      memory: "si"
      prompt: "s"
      clipboard: "i"
      shell: "i"
      read: "s"
      write: "su"
  # This is experimental automnemomorph prompt.
  # It needed to advanced model
  amm:
    facts:
      - protocol
      - answer-protocol
      - answer-rules
      - automnemomorf
      - history-add
      - history-change
      - history-remove
      - history-pack
      - memory-add
      - memory-change
      - memory-remove
      - prompt-add
      - prompt-change
      - prompt-remove
      - shell-add
      - clipboard-add
      - read
      - write
      - file-context
      - domain-permissions
      - user-env
    access:
      access: "s"
      history: "siud"
      memory: "siud"
      prompt: "siud"
      clipboard: "i"
      shell: "i"
      read: "s"
      write: "su"
  # This is standart prompt with entity property link model
  epl:
    facts:
      - protocol
      - answer-protocol
      - answer-rules
      - history-add
      - history-pack
      - memory-add
      - shell-add
      - clipboard-add
      - read
      - write
      - file-context
      - epl
      - domain-permissions
      - user-env
    access:
      access: "s"
      history: "siud"
      memory: "siud"
      prompt: "siud"
      clipboard: "i"
      shell: "i"
      read: "s"
      write: "su"
  # This simple prompt like webchat on fact protocol
  simple:
    facts:
      - protocol
      - answer-protocol
      - history-add
      - memory-add
      - domain-permissions
      - user-env
    access:
      access: "s"
      history: "si"
      memory: "si"
      prompt: "."
      clipboard: "."
      shell: "."
      read: "."
      write: "."
  # This is the content extractor, return content only from html document
  extractor:
    facts:
      - extract
  # This is the experimental text game promp
  game:
    facts:
      - protocol
      - answer-protocol
      - history-change
      - history-remove
      - history-pack
      - memory-add
      - memory-change
      - memory-remove
      - game-master
      - game-instruction
      - game-memory
      - game-start
      - game-inventory
      - domain-permissions
    access:
      access: "s"
      history: "siud"
      memory: "siud"
      prompt: "siud"
      clipboard: "."
      shell: "."
      read: "."
      write: "."

# Output destinations
destination:
  # For mac os:     cliclick t:"%data%"
  # For linux:      xdotool
  # For linux ssh:  ai --tiocsti
  #                 Requires `sudo sysctl -w dev.tty.legacy_tiocsti=1`
  command: "sleep 0.3 && xdotool type --clearmodifiers --delay 10 --file -"
  message: "cat && echo"
  clipboard: "xclip -selection clipboard"

application:

  # Log settings
  log:
    file: "%profile-path%/log.txt"
    enabled: true

  # Sets are presets that expand into CLI flags.
  # When --set=<name> is used, all keys under sets.<name> are applied as if
  # they were passed as command-line arguments. User-provided flags override
  # the preset.
  sets:
    extractor:
      provider: deepseek
      model: default
      prompt: extractor
      out-status: false
      no-history: true
      no-memory: true
      no-prompt: true
      max-prompt-bytes: 3000000

  # AI application
  ai:
    # AI providers configuration
    providers:

      deepseek:
        api: https://api.deepseek.com/v1/chat/completions
        models:
          default: deepseek-v4-flash
          pro: deepseek-v4-pro

      local:
        api: http://localhost:11434/api/generate
        proxy: ""
        models:
          default: llama3
          gemma: gemma3:1b1

      openai:
        api: https://api.openai.com/v1/chat/completions
        models:
          default: gpt-4.1

      groq:
        api: https://api.groq.com/openai/v1/chat/completions
        models:
          default: llama-3.3-70b-versatile
          instruct: llama-4-scout-17b-16e-instruct
          mixtral: mixtral-8x7b-32768

      together:
        api: https://api.together.xyz/v1/chat/completions
        models:
          default: meta-llama/Llama-3.3-70B-Instruct-Turbo

      ollama:
        api: https://ollama.com/api/chat
        models:
          default: deepseek-v4-flash
          gpt-oss:120b: gpt-oss:120b

      anthropic:
        api: https://api.anthropic.com/v1/messages
        models:
          default: claude-3-5-sonnet-20241022

    # Specific request contract
    rules:
    -
      # Deepseek rules for each model
      provider: "deepseek"
      model: "*"
      # Request scheme
      request:
        model: "%model-name%"
        messages:
        -
          content: "%prompt%"
          role: user
        thinking:
          type: disabled
      # Answer scheme
      answer: [ choices, 0, message, content ]
      tokens_in: [ usage, prompt_tokens ]
      tokens_out: [ usage, completion_tokens ]
      tokens_billed: [ usage, total_tokens ]
      tokens_cached: [ usage, prompt_tokens_details, cached_tokens ]

    -
      # local rules
      provider: local
      model: "*"

      request:
        model: "%model-name%"
        prompt: "%prompt%"
        stream: false
      answer: [ response, content ]

    -
      provider: ollama
      model: "*"

      answer: [ message, content ]
      request:
        model: "%model-name%"
        messages:
        -
          role: user
          content: "%prompt%"
        stream: false
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
