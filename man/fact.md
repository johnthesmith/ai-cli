# Facts

Facts are the foundation of the ai-cli and prompts operation. A fact is a
record in a text file.

Each fact begins with a single-line header: `#FACT|<domain>|<actor>|<id>`.
The first field is the delimiter #FACT.
The second field is the fact domain:
    prompt      – directives for LLM
    history     – temporary chat history
    memory      – long-term information
    shell       – shell commands or pipelines
    clipboard   – clipboard information
    read        – input file data
    write       – write file data
The third field is the fact owner:
    assistant
    user
The fourth field is the unique fact id or NEW for new facts.
Content is included until the next fact header.
Facts represent the current state of dialog shared between user and assistant.

Facts are used to build [prompts](prompts.md), [history](history.md),
[memory](memory.md).

You can view any used fact with the command `--select-fact=<fact-id>`.
For example:
```bash
ai --select-fact=protocol
```

You are free to add, modify, and delete facts following the specified format.
The [automnemomorph](automnemomor.md) can also perform all the listed
operations based on the permissions granted to it. You can find the
automnemomorph instructions in the `automnemomorf` fact.
