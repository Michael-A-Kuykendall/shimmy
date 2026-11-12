# API — Shimmy 2.6.7

Shimmy provides a bounded OpenAI-compatible surface, not universal API parity.
See [parameter compatibility](OPENAI_COMPAT.md), [access policy](CONFIGURATION.md)
and [actual fixture scope](SUPPORTED_MODELS.md).

| Route | Contract |
|---|---|
| GET /health | Health probe; does not certify model readiness |
| GET /v1/models | Discovered model IDs/capabilities; discovery is not numerical certification |
| POST /v1/chat/completions | JSON by default; SSE only with stream:true |
| POST /v1/completions | Text JSON; stream:true rejects with unsupported_streaming_mode |
| GET /api/tags | Ollama-style model listing, not full Ollama API parity |
| POST /v1/messages | Anthropic compatibility handler, not full Anthropic parity |
| GET /docs and /openapi.json | Running-server interactive and machine-readable documentation |
| GET /metrics and /diag | Local metrics/diagnostics; protect them as sensitive |

There is no GET /v1/models/:id route. Configured credentials protect every route
above except GET /health. Origin checks apply first, including health.
Anthropic x-api-key is an authentication credential when configured, not ignored.

## Chat request

```json
{"model":"MODEL_ID","messages":[{"role":"user","content":"The capital of France is"}],"max_tokens":8,"temperature":0,"stream":false}
```

Replace MODEL_ID with a returned ID. Successful JSON includes choices,
finish_reason and usage (prompt_tokens, completion_tokens, total_tokens).
A stop/EOS boundary reports stop; an exhausted output budget reports length.
Sampled terminal IDs contribute to token accounting without emitting stop text.
Do not equate string length with token count.

SSE uses data frames, content deltas, terminal finish information and [DONE].
Set stream_options.include_usage to request the final usage chunk. Generation
failures are errors, not successful empty completions. Client cancellation must
be followed by a fresh request state; selected recovery modes have executed
tests, not a universal transport/platform guarantee.

## Native and management surfaces

POST /api/generate accepts model and prompt or messages. Its JSON response uses
response plus usage/finish metadata, not the OpenAI choices shape. Raw prompt
mode is a separate explicit option. /ws/generate provides a distinct WebSocket
protocol. Use the implementation/installed OpenAPI for their exact schemas;
OpenAI examples are not interchangeable with them.

Native model listing/discovery routes exist. Model load/unload/status management
handlers contain placeholder acknowledgments, not a certified hot-swap or
multi-model eviction service. Tool/workflow routes do not imply implemented
external tool execution. The Airframe engine retains one model file and rejects
requests for a different resident file.

## Errors and capacity

Unknown models return 404; invalid inputs and unsupported/capacity requests
produce typed errors rather than silent fallback/truncation. Admission rejects
unsupported graph/kernel configurations. See [troubleshooting](TROUBLESHOOTING.md).
GPU device loss remains a platform risk; a numerical comparison does not fix it.
