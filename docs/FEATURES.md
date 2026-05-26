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

### Linux ARM64 Airframe GPU Enablement
*Planned — tracked for post-v2.0 stabilization*

Goal: ship a Linux ARM64 release binary with Airframe GPU engine enabled (instead of huggingface-only fallback).

Scope:
- Enable and validate Airframe engine build path for `aarch64-unknown-linux-gnu`
- Ensure runtime adapter selection works on real ARM64 hardware (including NVIDIA GB10 class devices)
- Keep current Linux ARM64 binary available until Airframe parity is validated

Acceptance criteria:
- `shimmy-linux-aarch64` release artifact reports Airframe enabled in `gpu-info`
- CI release workflow builds Linux ARM64 with Airframe without manual intervention
- Smoke test coverage includes Linux ARM64 Airframe serve + generation path

Tracked in: https://github.com/Michael-A-Kuykendall/shimmy/issues/131

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
