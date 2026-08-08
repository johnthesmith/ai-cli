# Completion (Shell Completion)

To enable CLI autocompletion, run the following steps:

Create the autocompletion directory:

```bash
mkdir -p ~/.local/share/bash-completion/completions
```

Create the autocompletion file:

```bash
cat > ~/.local/share/bash-completion/completions/ai << 'EOF'
_ai_completion() {
    mapfile -t COMPREPLY < <(ai --comp-line="$COMP_LINE" --comp-point="$COMP_POINT")
}
complete -o nospace -F _ai_completion ai
complete -o nospace -F _ai_completion 1
EOF
```

Copy the file for `1`:

```bash
cp ~/.local/share/bash-completion/completions/ai \
~/.local/share/bash-completion/completions/1
```

After this, restart the terminal.

### Requirements

- The `bash-completion` package must be installed (check: `type _init_completion`)

### How the keys work

- `--comp-line` — the current input line (what has been typed up to the cursor position)
- `--comp-point` — the cursor position in the input line

`ai-cli` returns a list of possible completion candidates, which bash inserts into the command line.
