# Automnemomorph

Automnemomorph (from Ancient Greek αὐτός — self, μνήμη — memory, μορφή — form)
is an agent possessing full arbitrary control over its own memory and capable,
by its own will and without a protocol, of performing SIUD operations on it:

* Select — selection and retrieval of desired facts (memories),
* Insert — creation of new facts (that never occurred),
* Update — modification of existing facts,
* Delete — deletion of existing facts.

[Look at](https://github.com/johnthesmith/scraps/blob/main/en/automnemomorph.md).



# For ai-cli

The capabilities of a mnemonomorph provide practical advantages, as they allow
focusing on the task at hand, shedding the burden of the past.

`ai-cli` is not an agent itself but enables the automnemomorph to function in
an interactive mode based on working with [facts](fact.md).

The automnemomorph protocol is included in the default prompt of the utility
and activates immediately upon creating a new chat or with the
`--bind-prompt=default` command.

You can view all automnemomorph directives by looking at the prompt content.
```
1 --out-prompt-content
```



# Downgrade

Using an automnemomorph imposes higher requirements on LLM.
You can downgrade automnemomorph for your chat to the simple prompt.

```
ai --bind-prompt=simple
```

and downgrade rights for LLM in the config for your [chat](chat.md):

```yaml
access-access: "s"
access-history: "s"
access-memory: "siud"
access-prompt: "s"
access-clipboard: "i"
access-shell: "i"
access-read: "s"
access-write: "su"
