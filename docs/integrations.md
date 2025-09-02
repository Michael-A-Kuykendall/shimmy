# Shimmy Integrations

Shimmy works with any tool that supports the OpenAI API. Here are the most popular integrations:

## Code Editors

### VSCode + Copilot
```json
// settings.json
{
  "github.copilot.advanced": {
    "serverUrl": "http://localhost:11435"
  }
}
```

### Cursor
1. Open Settings (Ctrl/Cmd + ,)
2. Go to "AI" settings
3. Set custom endpoint: `http://localhost:11435`

### Continue.dev
```json
// ~/.continue/config.json
{
  "models": [{
    "title": "Local Shimmy",
    "provider": "openai",
    "model": "your-model-name",
    "apiBase": "http://localhost:11435/v1"
  }],
  "tabAutocompleteModel": {
    "title": "Local Shimmy Tab",
    "provider": "openai", 
    "model": "your-model-name",
    "apiBase": "http://localhost:11435/v1"
  }
}
```

### Neovim + Copilot.lua
```lua
-- copilot.lua config
require('copilot').setup({
  server_opts_overrides = {
    settings = {
      ["*"] = {
        ["*"] = {
          editorConfiguration = {
            enableAutoCompletions = true,
          },
          advanced = {
            serverUrl = "http://localhost:11435"
          }
        }
      }
    }
  }
})
```

## Programming Languages

### Python
```python
import openai

client = openai.OpenAI(
    base_url="http://localhost:11435/v1",
    api_key="not-needed"
)

response = client.chat.completions.create(
    model="your-model-name",
    messages=[{"role": "user", "content": "Hello!"}]
)
print(response.choices[0].message.content)
```

### JavaScript/TypeScript
```javascript
import OpenAI from 'openai';

const openai = new OpenAI({
  baseURL: 'http://localhost:11435/v1',
  apiKey: 'not-needed'
});

const completion = await openai.chat.completions.create({
  model: 'your-model-name',
  messages: [{ role: 'user', content: 'Hello!' }],
});

console.log(completion.choices[0].message.content);
```

### Rust
```rust
// Using reqwest
use serde_json::json;

let client = reqwest::Client::new();
let response = client
    .post("http://localhost:11435/v1/chat/completions")
    .json(&json!({
        "model": "your-model-name",
        "messages": [{"role": "user", "content": "Hello!"}]
    }))
    .send()
    .await?;
```

### Go
```go
package main

import (
    "context"
    "fmt"
    "github.com/sashabaranov/go-openai"
)

func main() {
    config := openai.DefaultConfig("not-needed")
    config.BaseURL = "http://localhost:11435/v1"
    
    client := openai.NewClientWithConfig(config)
    
    resp, err := client.CreateChatCompletion(
        context.Background(),
        openai.ChatCompletionRequest{
            Model: "your-model-name",
            Messages: []openai.ChatCompletionMessage{
                {Role: "user", Content: "Hello!"},
            },
        },
    )
    
    if err != nil {
        panic(err)
    }
    
    fmt.Println(resp.Choices[0].Message.Content)
}
```

## CLI Tools

### LLM (Simon Willison's tool)
```bash
# Install: pipx install llm
llm install llm-openai-compatible

# Configure
llm keys set openai-compatible
# API Key: not-needed
# Base URL: http://localhost:11435/v1

# Use
llm chat -m your-model-name "Hello!"
```

### Aider
```bash
# Install: pipx install aider-chat
aider --openai-api-base http://localhost:11435/v1 --model your-model-name
```

## Docker

### Run Shimmy in Docker
```dockerfile
FROM ubuntu:22.04
COPY shimmy /usr/local/bin/
COPY models/ /models/
ENV SHIMMY_BASE_GGUF=/models/your-model.gguf
EXPOSE 11435
CMD ["shimmy", "serve", "--bind", "0.0.0.0:11435"]
```

```bash
docker build -t shimmy .
docker run -p 11435:11435 shimmy
```

## Testing Tools

### Simple curl test
```bash
curl -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-model-name",
    "messages": [{"role": "user", "content": "Say hello!"}],
    "max_tokens": 10
  }'
```

### Streaming test
```bash
curl -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-model-name", 
    "messages": [{"role": "user", "content": "Count to 5"}],
    "max_tokens": 20,
    "stream": true
  }'
```

## Common Issues

**Model name not found**: Use `./shimmy list` to see available model names

**Wrong port**: Shimmy defaults to 11435, but you can change with `--bind`

**Performance issues**: Try a smaller/faster model like Phi-3-mini

**Tool not working**: Ensure the tool supports custom OpenAI base URLs

---

**Missing an integration?** [Open an issue](https://github.com/Michael-A-Kuykendall/shimmy/issues) and we'll add it!
