# Memory

Memory is a list of long-term stored [facts](fact.md) in the file:
```
./.ai-cli/profiles/<profile-id>/memory/<memory-id>.txt
```
You can freely edit memory following the [fact](fact.md) format.

Unlike [history](history.md), memory does not belong to a [chat](chats.md),
so it can be shared between chats.

Memory allows accumulating and using facts. The default
[prompt](./prompts.md) teaches the LLM to work with memory as an isolated
block separate from history.

Connecting memory to a chat is done with the command
```
--bind-memory=<memory-id>
```

If the memory file does not exist at connection time, it will be created.
For a one-time memory switch in the current request, use the command
```
--memory=<memory-id>
```

or

```
-m=<memory-id>
```

Memory clearing is done with the command
```
--remove-memory
```
or
```
-rm
```
or
```
-rmh
```


In the latter case, the current chat history will also be cleared.
Note that memory is cleared immediately for all chats where it is connected.

You can view the current memory with the commands
```
--out-memory
```

or

```
-om
```

You can operate with memory facts using commands like
```
--select-fact=<fact-id>
```
