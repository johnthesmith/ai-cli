# Chats

Chats link [memory](memory.md), [prompt](prompts.md), and
[history](history.md) for interaction with LLM. You can switch between chats
using the `--bind-chat=<chat-id>` command or `--chat=<chat-id>` for the current
call. If the chat did not exist before, it will be created automatically.

By default, chats are stored in the folder:
```
./.ai-cli/profiles/<profile/>/chats/<chat>
```

After creating a chat, you may need to switch [provider](providers.md),
[model](model.md), [prompt](prompt.md).
