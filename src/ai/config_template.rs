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
# Placeholders: %profile-path% %profile% %caht% %provider% %model%
prompt-file: "%profile-path%/chats/%chat%/prompts/%prompt%.txt"

# History file
# Placeholders: %profile-path% %profile% %caht% %provider% %model%
history: "%profile-path%/chats/%chat%/history.txt"

# Memory for current chat
# Placeholders: %profile-path% %profile% %caht% %provider% %model%
memory-of-chat-file: "%profile-path%/chats/%chat%/memory.txt"

# Memory file
# Placeholders: %profile-path% %profile% %caht% %provider% %model%
memory-file: "%profile-path%/memory/%memory-id%.txt"

# Token file
# Placeholders: %profile-path% %profile% %caht% %provider% %model%
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

# Access control for AI operations
# Each string consists of letters:
#   insert, select, update, delete
# Modes for promt:
#   default: "i" - AI can only add, not modify or delete
#   automnemomorph : "iud" - AI can create, update, delete (full control)
access-access: "s"
access-history: "siud"
access-memory: "siud"
access-prompt: "siud"
access-clipboard: "i"
access-shell: "i"
access-read: "s"
access-write: "su"

# You can compile your prompt from facts
# ai --compile-prompt=default|amm|extractor or other
prompts:
  default:
  - protocol
  - answer
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
  - epl
  - domain-permissions
  - user-env
  simple:
  - protocol
  - answer
  - history-add
  - read
  - write
  - clipboard-add
  - shell-add
  - domain-permissions
  - user-env
  extractor:
  - extract
  game:
  - protocol
  - answer
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


# Output destinations
destination:
  # For mac os:     cliclick t:"%data%"
  # For linux:      xdotool
  # For linux ssh:  ai --tiocsti
  #                 Requires `sudo sysctl -w dev.tty.legacy_tiocsti=1`
  command: "sleep 0.3 && xdotool type --clearmodifiers --delay 10 --file -"
  message: "cat && echo"
  clipboard: "xclip -selection clipboard"

# AI util config file
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

      local:
        api: http://localhost:11434/api/generate
        proxy: ""
        models:
          default: llama3
          gemma: gemma3:1b1

      github:
        api: https://models.github.ai/inference/chat/completions
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
          cohere-a: cohere/cohere-command-a
          deepseek-r1: deepseek/deepseek-r1
          deepseek-r1-0528: deepseek/deepseek-r1-0528
          deepseek-v3-0324: deepseek/deepseek-v3-0324
          llama-3.2-11b-vision-instruct: meta/llama-3.2-11b-vision-instruct
          llama-3.2-90b-vision-instruct: meta/llama-3.2-90b-vision-instruct
          llama-3.3-70b-instruct: meta/llama-3.3-70b-instruct
          llama-4-maverick-17b-128e-instruct-fp8: meta/llama-4-maverick-17b-128e-instruct-fp8
          llama-4-scout-17b-16e-instruct: meta/llama-4-scout-17b-16e-instruct
          llama-3.1-405b-instruct: meta/meta-llama-3.1-405b-instruct
          llama-3.1-8b-instruct: meta/meta-llama-3.1-8b-instruct
          mistral-ai-codestral-2501: mistral-ai/codestral-2501
          mistral-ai-ministral-3b: mistral-ai/ministral-3b
          mistral-ai-mistral-medium-2505: mistral-ai/mistral-medium-2505
          mistral-ai-mistral-small-2503: mistral-ai/mistral-small-2503
          phi-4: microsoft/phi-4
          phi-4-mini-instruct: microsoft/phi-4-mini-instruct
          phi-4-mini-reasoning: microsoft/phi-4-mini-reasoning
          phi-4-multimodal-instruct: microsoft/phi-4-multimodal-instruct
          phi-4-reasoning: microsoft/phi-4-reasoning

      openai:
        api: https://api.openai.com/v1/chat/completions
        models:
          default: gpt-4.1

      deepseek:
        api: https://api.deepseek.com/v1/chat/completions
        models:
          default: deepseek-v4-flash
          pro: deepseek-v4-pro

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

      sh:
        api: https://ollama-platform-ai.docdoc.pro/api/chat
        models:
          default: "qwen3:30b-a3b-instruct-2507-q8_0"

    # Specific request contract
    rules:
    -
      # Deepseek rules for each model
      provider: "deepseek"
      model: "*"
      # Request pathes
      request:
        model: "%model-name%"
        messages:
        -
          content: "%prompt%"
          role: user
        thinking:
          type: disabled
      # answer pathes
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
