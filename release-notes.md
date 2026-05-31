# Release v1.0.1

## Added

1. Section statistics with size of history and memory for `--info`

## Changed

1. `--info` for YAML output



# Release v0.1.14

## Added

1. `max-chat-prompt-size-byte` – prompt size limit (default: 100000 bytes)
0. `%model%` placeholder in all file paths
0. Model name sanitization for filesystem safety

## Changed

1. **Hierarchical config** – parameters can be set at global, provider, model, or chat level with inheritance
0. **Unified config access** – single `get_config_val()` method replaces manual navigation
0. All paths now support `%profile%`, `%provider%`, `%model%`, `%chat%` placeholders

## Fixed

1. Double mutable borrow error on provider creation
0. Unsafe unwraps and type mismatches
0. Missing return values (stray semicolons)
