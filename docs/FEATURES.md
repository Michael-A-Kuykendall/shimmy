# Shimmy Features

## Auto-Discovery
- Automatically finds GGUF and SafeTensors models
- Scans common directories and environment variables
- Use `shimmy list` to see discovered models

## API Enhancements
- Proper HTTP status codes (404 for missing models, 502 for generation failures)
- `/metrics` endpoint for monitoring
- Enhanced error messages

## RustChain Integration
- Compatible as RustChain LLM provider
- See `docs/rustchain-provider.md` for configuration

## CLI Commands
- `serve` - Start HTTP server with all features
- `list` - Show discovered models
- `probe` - Test model loading
- `generate` - Quick CLI generation

## Environment Variables
- `SHIMMY_BASE_GGUF` - Primary model file
- `SHIMMY_LORA_GGUF` - Optional LoRA adapter
- Models also auto-discovered in:
  - `~/.cache/huggingface/`
  - `~/models/`
  - Parent directory of SHIMMY_BASE_GGUF

## Roadmap

### OpenAI Responses API (`POST /v1/responses`)
*Planned — near-term*

Support for OpenAI's Responses API, a newer alternative to Chat Completions.

Request shape:
```json
{
  "model": "local",
  "input": "Hello",
  "instructions": "You are helpful",
  "max_output_tokens": 512,
  "temperature": 0.7
}
```

Response access: `output[0].content[0].text`

Implementation is a new route + shape translation layer on top of the existing inference path — no engine changes required. Tool support (web search, code interpreter) is out of scope for v1.

Tracked in: https://github.com/Michael-A-Kuykendall/shimmy/issues/141
