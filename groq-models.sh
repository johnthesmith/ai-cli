# Dump models for Groq

curl -L \
    -H "Authorization: Bearer `cat ~/.config/ai/default/tokens/groq.txt`" \
    https://api.groq.com/openai/v1/models | jq '.data[].id'
