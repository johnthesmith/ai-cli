# Dump models for DeepSeek

curl -L \
    -H "Authorization: Bearer `cat ~/.config/ai/default/tokens/deepseek.txt`" \
    https://api.deepseek.com/v1/models | jq '.data[].id'
