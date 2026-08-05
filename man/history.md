# History

The chat history is a file with [facts](fact.md) for the current [chat](chat.md).
The history is located in the file
```
./.ai-cli/profiles/<profile/>/chats/<chat>/history.txt
```

You are free to edit the history, but follow the fact format.

The history is updated simultaneously by you and the tool at the initiative of
the LLM when processing your requests. When the history changes at the
initiative of the llm, you can see the mnemonic `+h|-h|^h`.

To view the history, use
```
--out-history
```
or
```
-oh
```

You can operate with history facts using commands like
```
--select-fact=<fact-id>
```

The history can be deleted as a file or with the command
```
--remove-history
```
or
```
-rh
```
or
```
-rmh
```

In the latter case, the [memory](memory.md) connected to the chat will also be
deleted.

