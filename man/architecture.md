# Architecture

1. ai-cli interacts with the LLM based on the [fact](fact.md) protocol.



## Overview

1. ai-cli collects incoming text data and converts it into facts
2. facts are combined into a text request and sent to the LLM
3. the LLM response is parsed into facts
4. facts are distributed across domains
5. actions are performed on the resulting list of facts to save or apply them



## Diagram

```mermaid
flowchart LR

        subgraph filesystem read

            files_ro_in[("Files \n read")]
            files_rw_in[("Files \n read-write")]
            memory_in[("Share \n memory")]
            history_in[("Chat \n history")]
            prompt_in[("User \n prompt")]
        end
        subgraph filesystem write
            files_rw_out[("Files \n read-write")]
            memory_out[("Share \n memory")]
            history_out[("Chat \n history")]
            prompt_out[("User \n prompt")]
            log[("Log")]
        end

        clipboard["Clipboard"]
        stdin{{"User stdin"}}
        param{{"CLI params"}}
        command{{"bash"}}
        stdout{{"User stdout"}}

        merge{merge}
        split{split}

        req["Request"]
        resp["Response"]


    subgraph World["External"]
        llm["LLM API"]
    end

    split --> |all| log

    llm --> |HTTP \n response| resp
    req -->|HTTP \n request| llm

    resp --> |fact| split
    merge --> |fact| req

    split -->|txt| memory_out
    split -->|txt| history_out
    split -->|txt| prompt_out
    split -->|txt| stdout
    split -->|txt| files_rw_out
    split -->|command| command
    split -->|txt| clipboard

    files_ro_in --> |txt| merge
    files_rw_in --> |txt| merge
    memory_in --> |txt| merge
    prompt_in --> |txt| merge
    history_in --> |txt| merge
    stdin -->|txt| merge
    param -->|txt| merge
```

