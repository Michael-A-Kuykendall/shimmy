# API Reference

Shimmy provides multiple API interfaces for local LLM inference.

## HTTP REST API

### Generate Text

**Endpoint:** `POST /api/generate`

**Request Body:**
```json
{
  "model": "string",           // Model name (required)
  "prompt": "string",          // Input prompt (required)
  "max_tokens": 100,          // Maximum tokens to generate (optional, default: 100)
  "temperature": 0.7,         // Sampling temperature (optional, default: 0.7)
  "stream": false             // Enable streaming response (optional, default: false)
}
```

**Non-Streaming Response:**
```json
{
  "choices": [
    {
      "text": "Generated text response",
      "index": 0,
      "finish_reason": "length"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  }
}
```

**Streaming Response:**
Server-Sent Events with data chunks:
```
data: {"choices":[{"text":"Hello","index":0}]}

data: {"choices":[{"text":" world","index":0}]}

data: [DONE]
```

### List Models

**Endpoint:** `GET /api/models`

**Response:**
```json
{
  "models": [
    {
      "id": "default",
      "name": "Default Model",
      "description": "Base GGUF model"
    }
  ]
}
```

### Health Check

**Endpoint:** `GET /api/health`

**Response:**
```json
{
  "status": "healthy",
  "models_loaded": 1,
  "memory_usage": "2.1GB"
}
```

## WebSocket API

**Endpoint:** `ws://localhost:11435/ws/generate`

### Connect and Send
```json
{
  "model": "default",
  "prompt": "Hello world",
  "max_tokens": 50,
  "temperature": 0.7
}
```

### Receive Tokens
```json
{"token": "Hello"}
{"token": " world"}
{"done": true}
```

## CLI Interface

### Commands

```bash
# Start server
shimmy serve --bind 127.0.0.1:11435 --port 11435

# Generate text
shimmy generate --prompt "Hello" --max-tokens 50 --temperature 0.7

# List available models
shimmy list

# Probe model loading
shimmy probe [model-name]

# Show diagnostics
shimmy diag
```

### Global Options

- `--verbose, -v`: Enable verbose logging
- `--help, -h`: Show help information
- `--version, -V`: Show version information

## Error Responses

All endpoints return consistent error formats:

```json
{
  "error": {
    "code": "model_not_found",
    "message": "The specified model was not found",
    "details": "Model 'invalid-model' is not available"
  }
}
```

Common error codes:
- `model_not_found`: Requested model is not available
- `invalid_request`: Request format is invalid
- `generation_failed`: Text generation failed
- `server_error`: Internal server error

## Rate Limiting

Currently no rate limiting is implemented. For production use, consider placing shimmy behind a reverse proxy with rate limiting capabilities.
