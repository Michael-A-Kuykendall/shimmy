# API Reference

Shimmy exposes three interface families:

- **OpenAI-compatible** (`/v1/*`) — drop-in replacement for OpenAI SDKs and tools
- **Shimmy native** (`/api/*`) — the original REST + WebSocket API
- **Ollama-compatible** (`/api/tags`) and **Anthropic-compatible** (`/v1/messages`) — for existing tooling

All endpoints serve the same inference engine. See
[docs/OPENAI_COMPAT.md](OPENAI_COMPAT.md) for the exact OpenAI parameter
compatibility matrix.

Interactive documentation is available from a running server at
`GET /docs`; the machine-readable contract is `GET /openapi.json`.
See [OpenAPI and Swagger UI](OPENAPI.md) for usage and implementation details.

## OpenAI-Compatible Endpoints

### `GET /v1/models`
Lists available models (also served as `GET /api/tags` for Ollama clients).

```json
{
  "object": "list",
  "data": [
    { "id": "tinyllama-1.1b", "object": "model", "owned_by": "shimmy" }
  ]
}
```

### `POST /v1/chat/completions`
OpenAI-compatible chat completions. Streaming supported via SSE when
`"stream": true`.

```json
{
  "model": "tinyllama-1.1b",
  "messages": [{"role": "user", "content": "Say hi in 5 words."}],
  "max_tokens": 32,
  "temperature": 0.7
}
```

### `POST /v1/completions`
Legacy text completions (non-chat). Same request shape minus `messages`.

### `POST /v1/messages`
Anthropic Messages API compatibility (`x-api-key` optional, ignored).

## Shimmy Native Endpoints

### `POST /api/generate`
Text generation (streaming + non-streaming).

**Request:**
```json
{
  "model": "string",
  "prompt": "string",
  "max_tokens": 100,
  "temperature": 0.7,
  "stream": false
}
```

**Non-streaming response:**
```json
{
  "choices": [{"text": "Generated text", "index": 0, "finish_reason": "length"}],
  "usage": {"prompt_tokens": 10, "completion_tokens": 20, "total_tokens": 30}
}
```

**Streaming response** (SSE):
```
data: {"choices":[{"text":"Hello","index":0}]}
data: {"choices":[{"text":" world","index":0}]}
data: [DONE]
```

### `GET /api/models`
List loaded models.

### `POST /api/models/discover`
Trigger model re-discovery.

### `POST /api/models/:name/load` / `POST /api/models/:name/unload`
Load or unload a specific model by name.

### `GET /api/models/:name/status`
Model status (loaded, VRAM, context).

### `GET /api/tags`
Ollama-compatible model list.

### `GET /health`
Health check.

```json
{"status": "healthy", "models_loaded": 1, "memory_usage": "2.1GB"}
```

### `GET /metrics`
Prometheus-style metrics endpoint.

### `GET /diag`
Diagnostics endpoint.

## WebSocket API

**Endpoint:** `ws://localhost:11435/ws/generate`

Send a single JSON `GenerateRequest` frame:
```json
{"model": "default", "prompt": "Hello world", "max_tokens": 50, "temperature": 0.7}
```

Receive token frames:
```json
{"token": "Hello"}
{"token": " world"}
{"done": true}
```

The WebSocket upgrade is documented in the OpenAPI contract as `GET
/ws/generate`, but its token frames are not REST JSON responses.

## CLI

```bash
shimmy serve                              # Start server (auto port allocation)
shimmy serve --bind 127.0.0.1:8080        # Manual port binding
shimmy serve --kv-quant int4              # Enable TurboShimmy INT4 KV
shimmy list                               # Show available models
shimmy discover                           # Refresh model discovery
shimmy generate --name X --prompt "Hi"    # Test generation
shimmy probe model-name                   # Verify model loads
shimmy gpu-info                           # Show selected WebGPU adapter
```

Global flags: `--verbose/-v`, `--help/-h`, `--version/-V`.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `SHIMMY_BASE_GGUF` | *(auto-discover)* | Path to GGUF model loaded as default |
| `SHIMMY_PORT` | `8080` | Listen port |
| `SHIMMY_BIND_ADDRESS` | `0.0.0.0:8080` | Full bind address (overrides port) |
| `SHIMMY_MAX_CTX` | *(from GGUF)* | Override context window; activates YaRN RoPE scaling when above native |
| `SHIMMY_MODEL_PATHS` | *(auto)* | Colon-separated extra model search paths |
| `SHIMMY_ENGINE_BACKEND` | `airframe` | Inference engine (airframe is the only engine) |
| `SHIMMY_ROPE_SCALE` | *(auto)* | Override computed YaRN scale factor |
| `SHIMMY_KV_QUANT` | `f32` | `f32` (default) or `int4` ([TurboShimmy](turboshimmy.md)) |
| `SHIMMY_PREFILL_CHUNK` | `64` | Tokens per prefill dispatch; set to `8` on Windows to avoid GPU TDR resets |
| `RUST_BACKTRACE` | *(off)* | Set to `1` for crash backtraces |

See [docs/CONFIGURATION.md](CONFIGURATION.md) for the complete configuration
reference.

## Error Responses

All endpoints return a consistent error format:

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
- `model_not_found` — requested model is not available
- `invalid_request` — request format is invalid
- `generation_failed` — text generation failed
- `server_error` — internal server error

## Rate Limiting

No rate limiting is currently implemented. For production use, place Shimmy
behind a reverse proxy with rate-limiting capabilities.
