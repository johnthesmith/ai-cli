# Утилита cli-ai

1. Утилита предназначена для встраивания ai в bash pipeline
2. Использование:
    1. ```echo "что думаешь про погоду" | cli-ai```
    2. ```echo "создай мне папку a" | cli-ai -e``` 

# Диаграмма

```mermaid
graph LR
system-prompt --> cli-ai
any --> stdout-any --> stdin-cli-ai--> cli-ai --> stdout-cli-ai --> user-confirm
```


# Ключи

1. `-e` | `--execute` - выполнить вывод утилиты;
2. `-p` | `--propmpt` - указать системный промпт.
