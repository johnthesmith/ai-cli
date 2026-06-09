# Dump models for DeepSeek

curl -L --socks5 "127.0.0.1:1080" \
-H "Authorization: Bearer `cat ~/.config/ai/app/cli/default/tokens/deepseek.txt`" \
https://api.deepseek.com/v1/models
