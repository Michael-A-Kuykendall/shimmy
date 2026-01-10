# Shimmy Features

## Auto-Discovery
- Automatically finds GGUF and SafeTensors models
- Scans common directories and environment variables
- Use `cargo run --features llama -- list` to see discovered models

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
