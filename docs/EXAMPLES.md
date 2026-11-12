# Examples — Shimmy 2.6.7

Install and select a verified fixture using the [quickstart](quickstart.md).
Use returned/discovered model IDs rather than assuming filenames are aliases.

```bash
shimmy --version
shimmy list
shimmy --model-dirs /absolute/path/to/models discover
shimmy generate MODEL_NAME --prompt "The capital of France is" --max-tokens 8
shimmy --ctx 512 --kv-quant f32 serve --model-path /path/to/model.gguf --bind 127.0.0.1:11435
```

Replace MODEL_NAME and paths before running. A direct serve path registers its
filename stem; listing/discovery is not proof that execution is supported.

```bash
curl -f http://127.0.0.1:11435/v1/models
curl -f http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"MODEL_ID","messages":[{"role":"user","content":"Hello"}],"max_tokens":8,"stream":false}'
```

Replace MODEL_ID with a listed ID. Add credentials when configured. For chat
SSE use stream:true and curl -N. Text-completion SSE is unsupported.
See [API](API.md), [network access](quickstart.md#network-access-and-api-keys)
and [fixture limits](SUPPORTED_MODELS.md).
