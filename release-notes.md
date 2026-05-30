# Release v0.1.14

## Added

- `max-chat-prompt-size-byte` – prompt size limit (default: 100000 bytes)
- `%model%` placeholder in all file paths
- Model name sanitization for filesystem safety

## Changed

- **Hierarchical config** – parameters can be set at global, provider, model, or chat level with inheritance
- **Unified config access** – single `get_config_val()` method replaces manual navigation
- All paths now support `%profile%`, `%provider%`, `%model%`, `%chat%` placeholders

## Fixed

- Double mutable borrow error on provider creation
- Unsafe unwraps and type mismatches
- Missing return values (stray semicolons)
