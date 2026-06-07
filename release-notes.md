# Release v1.0.10

1. Fixed `delete` operation (now properly accumulates multiple IDs)
0. Added block delimiter isolation for user prompt (prevents accidental injection)
0. Updated system prompt (block-based format, clarified role attribution)
0. **Important:** Remove old configs prompts memory and history before 
update (`rm -rf ~/.config/ai/app/cli ~/.local/share/ai/app/cli`).


# Release v1.0.9

1. Added CLI commands for storage operations (`--select`, `--delete`, `--update`, `--insert`)
0. Replaced JSON with block-based format for LLM communication (more reliable, no escaping issues)



# Release v1.0.8

1. Follows [AI Config Standard Proposal](https://github.com/johnthesmith/scraps/blob/main/en/proposal_ai_config_standard.md).
0. Access control (`c`/`u`/`d` permissions) and 
[auto-mnemomorph](./README.md#for-developers) mode (full AI control over history/memory)
0. Removed `--pack-history` command (history compaction now handled by AI via 
natural language)



# Release v1.0.7

1. Refactoring



# Release v1.0.4

1. Automatic configuration file creation on first run
0. Automatic prompt files creation (`chat.txt`, `summary.txt`) from embedded defaults
0. Automatic token file creation (empty) when missing
0. Configurable `request_timeout_ms` and `connect_timeout_ms`



# Release v1.0.3

1. Synchronous `message` output — fixes prompt/response order
0. ARMv6/ARMv7 support (Raspberry Pi)
0. Switched to `rustls` — no OpenSSL dependencies
0. Updated `install.sh` for ARM detection
0. Fixed message display before shell prompt



# Release v1.0.2

1. Version for `--info`.



# Release v1.0.1

1. Section statistics with size of history and memory for `--info`
0. `--info` for YAML output



# Release v0.1.14

1. `max-chat-prompt-size-byte` – prompt size limit (default: 100000 bytes)
0. `%model%` placeholder in all file paths
0. Model name sanitization for filesystem safety
0. **Hierarchical config** – parameters can be set at global, provider, model, 
or chat level with inheritance
0. **Unified config access** – single `get_config_val()` method replaces manual 
navigation
0. All paths now support `%profile%`, `%provider%`, `%model%`, `%chat%` 
placeholders
0. Double mutable borrow error on provider creation
0. Unsafe unwraps and type mismatches
0. Missing return values (stray semicolons)
