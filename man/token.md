# Tokens

1. Tokens use for LLM provider authorization.
2. Tokens spreded for all chats and collected in
`~/.config/ai/app/cli/<profile-id>/tokens/<provider>.txt` file.
3. Your need to put your token in the specific file manualy.



# Deepseek

1. Go to: https://platform.deepseek.com/api_keys



# Tokens and profiles

You can put the token for each profile, for example `deepseek` profile for 
`alice`:
```
~/.config/ai/app/cli/alice/tokens/deepseek.txt
```

or you can change `token` path in [config](./config.md) for example

```
~/.config/ai/app/cli/tokens/%provider%.txt
```

or other.
