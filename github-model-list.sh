#  Dump models for github

curl -L \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer `cat ~/.config/ai/default/tokens/github.txt`" \
    https://models.github.ai/catalog/models | jq '.[] | .id'
