# OpenAI compatibility — Shimmy 2.6.6

This is a selected protocol surface. It is not a promise of all OpenAI SDK,
endpoint, tool, image, audio or structured-output features.

| Capability | Disposition |
|---|---|
| GET /v1/models | Implemented listing |
| POST /v1/chat/completions | JSON default; chat SSE when explicitly requested |
| POST /v1/completions | JSON only; streaming rejects |
| GET /v1/models/:id | No route |
| Chat usage/finish reasons/stop boundaries | Executed selected server regressions |
| stream_options.include_usage | Implemented final chat usage chunk |
| Arbitrary model/quant/platform correctness | Not implied; see exact fixture matrix |
| Function/tool calling, vision, audio, embeddings | Not certified by this dense-text release |

Use model, messages, temperature, top_p, max_tokens, stop, stream and
stream_options according to the actual request types. Text completions use
prompt instead of messages. Unsupported or ignored extra fields do not become
implemented features simply because JSON deserialization succeeds.
Use explicit stream:false for JSON or omit it; no automatic SSE default.

Configure the client base URL as http://127.0.0.1:11435/v1 and use the configured
API key. A placeholder key is acceptable only for a local server without a
configured key. Non-loopback listeners require a real private key and remote
traffic should use TLS.

Prompt rendering is performed once, preserving conversation turns and exact
prepared IDs. The model must have an admitted template/tokenizer contract.
See [API](API.md), [integration](INTEGRATION.md),
[chat templates](CHAT_TEMPLATES.md) and [fixture matrix](SUPPORTED_MODELS.md).
