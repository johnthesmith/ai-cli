[ai-cli](../README.md) | [man](./index.md)

---

# Providers

Providers give access to LLMs for ai-cli via the REST API. A provider is bound
to the current [chat](./chats.md) and can be changed. The list of available
provider identifiers is specified in the [config](./config.md) as keys of the
`application.ai.providers` section.

When working with `ai-cli`, the provider can be switched permanently for the
chat with the `--bind-provider=<id-provider>` command, or only for the current
call with `--provider=<id-provider>`.

# Default

DeepSeek is the default provider after the `--init` command.

# Provider models

Each provider can provide one or more [models](./models.md).

# Providers API

Providers have different APIs that are defined in the config key `api`.
Specific fields for sending data to various APIs are described in the key
`application.ai.rules`. This key contains a list of rules. A rule is selected
by the current `[provider, model]` pair. The `request` keys form the request
fields. The `answer` keys are used to extract data from the API response. You
can add your own specific settings following the examples from the config.
