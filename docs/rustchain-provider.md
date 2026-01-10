# Using Shimmy as RustChain LLM Provider

Shimmy can serve as a local LLM provider for RustChain missions.

## Provider Configuration

Add to your RustChain config:

```yaml
llm_providers:
  shimmy:
    type: "http"
    base_url: "http://127.0.0.1:11435"
    endpoint: "/api/generate"
    model: "phi3-lora"  # or your loaded model name
    request_format: "shimmy"
```

## Request Format

Shimmy expects requests in this format:
```json
{
  "model": "phi3-lora",
  "prompt": "Your prompt here",
  "max_tokens": 512,
  "temperature": 0.7,
  "stream": false
}
```

## Usage in Missions

Reference in mission YAML:
```yaml
steps:
  - step_type: "llm"
    provider: "shimmy"
    prompt: "Analyze this code..."
```
