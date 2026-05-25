# GitHub Token Setup

1. Go to: https://github.com/settings/personal-access-tokens
0. Click **Fine-grained token**
0. Click **Generate new token**
0. Fill:
   - **Token name**: `ai-cli`
   - **Expiration**: 90 days
   - **Resource owner**: your account
0. **Repository access**: `Public Repositories (read-only)`
0. Click **Permissions** → **Models** → `Read-only`
0. Click **Generate token**
0. **Copy token immediately** (shown only once)
8. Put your github token here `~/.config/local/ai/default/token.txt`
