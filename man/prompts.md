# Prompts

Prompts are LLM request texts with a description of request processing rules.
Prompts are augmented with [history](history.md) and [memory](memory.md).

By default, prompts are stored in files:
`./.ai-cli/profiles/<profile/>/chats/<chat>/prompts/<prompt>.txt` Thus, each
[chat](chats.md) can have its own prompts between which you can switch within
each chat using the `--bind-prompt=<id>` command for permanent use and
`--prompt=<id>` for a single request. The identifier of the current chat is
stored in the file `./.ai-cli/profiles/<profile>/chats/<chat>/prompt.txt`.

Prompts are created by default from the fact template. For the initial
prompt creation use the `--build-prompt=<template>` command, where the
template specifies the [configuration](config.md) key in the `prompts.<template>`
argument. A prompt will be created with the current name <id> based on the
<template> from the listed facts. You can define your own templates. After
creation, you can manually edit any prompt in the file.

To view the current prompt, use the `--out-prompt` or `-op` command.
The original prompt file can be viewed with the `--out-prompt-content` or
`-opc` command.

Do not be afraid to ruin the prompt or that automnemomorph will do it. You can
always restore any prompt with `--build-prompt=<template>`.



# Examples

You can switch any prompt by name for current chat:
```
1 --bind-prompt=my-prompt
```

or you can use the prompt for the current request only:

```
1 --prompt=<name>
```


If you see "template not found", build the prompt from a template once:

```
1 --build-prompt=default
```

It takes facts from the config and builds the prompt for your `<name>`.
You can build any prompt this way.


Once the prompt is ready, you can work with its facts:

```
1 --select-fact=<id>
1 --delete-fact=<id>
1 --update-fact=<id> --actor=<actor>
1 --insert-fact=<id> --actor=<actor>
```
