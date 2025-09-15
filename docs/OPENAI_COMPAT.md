# OpenAI Compatibility Matrix

Shimmy exposes an OpenAI‑style API for easy drop‑in usage. This document spells out what is supported today, what is partially supported, and what is not (yet).

> **TL;DR**: Shimmy targets **Chat Completions** and **Models** first. Everything else is explicit below.

## Endpoints

| Endpoint | Status | Notes |
|---|---|---|
| `POST /v1/chat/completions` | **Supported** | Streaming via SSE (`stream: true`) supported. See examples below. |
| `GET /v1/models` | **Supported** | Lists locally available/aliased models. |
| `GET /v1/models/:id` | **Supported** | Metadata for a specific model, if present. |
| `POST /v1/completions` | *Optional/If present* | Legacy completion surface (document if enabled). |
| `POST /v1/embeddings` | **Not supported** | Planned/Out of scope for initial releases. |
| `POST /v1/images/*` | **Not supported** | N/A. |
| `POST /v1/audio/*` | **Not supported** | N/A. |
| `POST /v1/responses` | **Not supported** | Use chat completions. |
| Tool/Function Calling (chat) | *If implemented* | Document `tool_calls` schema + round‑trip example below, or mark as "Not supported". |

> Update the table to match the current binary; keep this honest to preempt "100% compatibility" nitpicks.

## Request/Response Compatibility (Chat Completions)

| Field | Status | Notes |
|---|---|---|
| `model` | **Required** | Accepts local model ID/alias. |
| `messages[]` | **Supported** | `role` in {`system`,`user`,`assistant`,`tool`} as supported. |
| `stream` | **Supported** | SSE with `data: { choices: [{ delta: { content } }] }`. |
| `temperature`, `top_p` | **Supported** | Standard float ranges. |
| `max_tokens` | **Supported** | Enforced cap; may differ by backend. |
| `tools`, `tool_choice` | *If supported* | Provide example or mark unsupported. |
| `logprobs`, `top_logprobs` | *Planned/If supported* | Document behavior. |
| `response_format` | **Ignored/If supported** | Note exact behavior. |

## Example: Chat (streaming)

```bash
curl -N http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "<YOUR_MODEL>",
    "stream": true,
    "messages": [
      {"role":"system","content":"You are a concise assistant."},
      {"role":"user","content":"Say hello in Rust style."}
    ]
  }'
```

## Example: List Models

```bash
curl http://127.0.0.1:11435/v1/models
```

## Differences from OpenAI

* Only documented fields above are honored; unknown fields are ignored with best‑effort defaults.
* Rate limiting and usage accounting may differ.
* Server returns local, deterministic errors for missing models or backend issues.

> If you add/remove features, update this matrix in the same PR.