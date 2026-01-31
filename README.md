# Утилита cliai

1. Утилита предназначена для встраивания ai в bash pipeline
2. Использование:
    1. ```echo "что думаешь про погоду" | cliai```
    2. ```echo "создай мне папку a" | cliai -e``` 

# Диаграмма

```mermaid
graph LR
system-prompt --> cliai
any --> stdout-any --> stdin-cliai--> cliai --> stdout-cliai --> user-confirm
```


# Ключи

1. `-e` | `--execute` - выполнить вывод утилиты;
2. `-p` | `--propmpt` - указать системный промпт.
