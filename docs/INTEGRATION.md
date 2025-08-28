# Integration Guide

This guide shows how to integrate Shimmy with various tools and platforms.

## RustChain Integration

Shimmy works seamlessly with RustChain for AI agent mission execution.

### Setup

1. Start Shimmy server:
```bash
export SHIMMY_BASE_GGUF=/path/to/model.gguf
shimmy serve --bind 127.0.0.1:11435
```

2. Configure RustChain to use Shimmy:
```toml
# rustchain.toml
[llm]
provider = "shimmy"
base_url = "http://localhost:11435"
model = "default"
```

3. Create RustChain missions that use LLM steps:
```yaml
version: "1.0"
name: "code_analysis"
steps:
  - id: "analyze"
    step_type: "llm"
    parameters:
      prompt: "Analyze this code: {{code_input}}"
      max_tokens: 500
```

## Punch Discovery Integration

Use Shimmy with punch-discovery for enhanced code analysis.

### Setup

1. Install punch-discovery alongside Shimmy
2. Configure punch to use Shimmy for AI analysis:

```yaml
# punch.yaml
llm:
  provider: shimmy
  endpoint: http://localhost:11435/api/generate
  model: default
```

### Workflow Example

```bash
# Analyze codebase with punch
punch discover /path/to/project

# Use results with Shimmy for further analysis
shimmy generate --prompt "$(cat analysis.json)" --max-tokens 1000
```

## VSCode Extensions

Shimmy can serve as a backend for VSCode AI extensions.

### Continue.dev Integration

Configure Continue.dev to use Shimmy:

```json
{
  "models": [
    {
      "title": "Shimmy Local",
      "provider": "openai",
      "model": "default",
      "apiBase": "http://localhost:11435/v1",
      "apiKey": "none"
    }
  ]
}
```

### Custom Extension Integration

Create a VSCode extension that uses Shimmy:

```typescript
import * as vscode from 'vscode';

async function generateWithShimmy(prompt: string): Promise<string> {
    const response = await fetch('http://localhost:11435/api/generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            model: 'default',
            prompt: prompt,
            max_tokens: 500,
            stream: false
        })
    });
    
    const data = await response.json();
    return data.choices[0].text;
}
```

## Cursor IDE Integration

Configure Cursor to use Shimmy as a local provider:

1. Open Cursor settings
2. Navigate to AI settings
3. Add custom provider:
   - URL: `http://localhost:11435/api/generate`
   - Model: `default`
   - API Key: (leave empty)

## CLI Tools Integration

### Shell Scripts

```bash
#!/bin/bash
# generate_commit_message.sh

# Get git diff
diff=$(git diff --cached)

# Generate commit message with Shimmy
commit_msg=$(curl -s -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"default\",
    \"prompt\": \"Generate a commit message for this diff:\n$diff\",
    \"max_tokens\": 50
  }" | jq -r '.choices[0].text')

echo "Suggested commit message: $commit_msg"
```

### Python Integration

```python
import requests
import json

class ShimmyClient:
    def __init__(self, base_url="http://localhost:11435"):
        self.base_url = base_url
    
    def generate(self, prompt, max_tokens=100, temperature=0.7):
        response = requests.post(
            f"{self.base_url}/api/generate",
            json={
                "model": "default",
                "prompt": prompt,
                "max_tokens": max_tokens,
                "temperature": temperature,
                "stream": False
            }
        )
        return response.json()["choices"][0]["text"]
    
    def stream_generate(self, prompt, max_tokens=100):
        response = requests.post(
            f"{self.base_url}/api/generate",
            json={
                "model": "default",
                "prompt": prompt,
                "max_tokens": max_tokens,
                "stream": True
            },
            stream=True
        )
        
        for line in response.iter_lines():
            if line.startswith(b"data: "):
                data = line[6:].decode()
                if data == "[DONE]":
                    break
                yield json.loads(data)["choices"][0]["text"]

# Usage
client = ShimmyClient()
result = client.generate("Hello, world!")
print(result)
```

### Node.js Integration

```javascript
const axios = require('axios');

class ShimmyClient {
    constructor(baseUrl = 'http://localhost:11435') {
        this.baseUrl = baseUrl;
    }
    
    async generate(prompt, options = {}) {
        const response = await axios.post(`${this.baseUrl}/api/generate`, {
            model: 'default',
            prompt: prompt,
            max_tokens: options.maxTokens || 100,
            temperature: options.temperature || 0.7,
            stream: false
        });
        
        return response.data.choices[0].text;
    }
}

// Usage
const client = new ShimmyClient();
client.generate('Hello, world!').then(result => {
    console.log(result);
});
```

## Docker Integration

Use Shimmy in containerized environments:

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --features llama

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/shimmy /usr/local/bin/shimmy
COPY models/ /models/

ENV SHIMMY_BASE_GGUF=/models/model.gguf

EXPOSE 11435

CMD ["shimmy", "serve", "--bind", "0.0.0.0:11435"]
```

## Reverse Proxy Setup

### Nginx Configuration

```nginx
server {
    listen 80;
    server_name shimmy.example.com;
    
    location / {
        proxy_pass http://localhost:11435;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # For WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

### Caddy Configuration

```caddyfile
shimmy.example.com {
    reverse_proxy localhost:11435
}
```

## Load Balancing

For high-availability setups, run multiple Shimmy instances:

```bash
# Instance 1
SHIMMY_BASE_GGUF=/models/model.gguf shimmy serve --bind 127.0.0.1:11435

# Instance 2  
SHIMMY_BASE_GGUF=/models/model.gguf shimmy serve --bind 127.0.0.1:11436

# Instance 3
SHIMMY_BASE_GGUF=/models/model.gguf shimmy serve --bind 127.0.0.1:11437
```

Configure your load balancer to distribute requests across instances.

## Monitoring Integration

### Prometheus Metrics

Shimmy exposes basic metrics at `/metrics`:

```
# Request count
shimmy_requests_total{method="POST",endpoint="/api/generate"} 150

# Response time
shimmy_request_duration_seconds_bucket{le="0.1"} 45

# Model status
shimmy_model_loaded{model="default"} 1
```

### Health Checks

Use the health endpoint for monitoring:

```bash
# Health check script
#!/bin/bash
response=$(curl -s http://localhost:11435/api/health)
status=$(echo $response | jq -r '.status')

if [ "$status" = "healthy" ]; then
    exit 0
else
    exit 1
fi
```
