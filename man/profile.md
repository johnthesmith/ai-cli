# Profile

AI CLI supports many profiles for each instance. The default profile is
created after the `--init` command in the `./.ai-cli/profiles/<profile>`
folder. It contains the following entities:
- [chats](chats.md)
- [history](history.md)
- [memory](memory.md)
- [prompts](prompts.md)

You can create or bind other profiles with `--bind-profile=<profile-id>` for
example:

```
1 --bind-profile=default
1 --bind-profile=home
1 --bind-profile=work
1 --bind-profile=alice
```

Read specific information about
[tokens settings for profiles](./token.md#tokens-and-profiles).

The current profile is stored in `./.ai-cli/profile.txt`.
To remove a profile, use the `rm -rf` command.


