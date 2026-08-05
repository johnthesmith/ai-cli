# Models

Each chat works with a specific model. The model depends on the current
[provider](providers.md).

The list of available models is contained in the
[configuration file](config.md) in the `application.ai.providers.<provider>.models` section.
You can add models as you see fit. The main thing is that the
[provider](providers.md) supports them.

The current model for a chat is stored in the file
`./.ai-cli/profiles/<profile>/chats/<chat>/model.txt`. Thus, each
[chat](chats.md) has its own model. You can switch the model within each chat
using the `--bind-model=<id>` command for permanent use and `--model=<id>` for
a one-time request.
