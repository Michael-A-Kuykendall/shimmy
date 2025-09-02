# Shimmy: Instant LoRA Inference

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
[![Sponsor](https://img.shields.io/badge/❤️-Sponsor-ea4aaa?logo=github)](https://github.com/sponsors/Michael-A-Kuykendall)

**From LoRA training to production API in under 30 seconds.**

Shimmy solves the developer pain point: You just trained a useful LoRA adapter with Unsloth/PEFT/Axolotl, now you want to serve it immediately without conversion hassles.

> "No more fighting with conversion scripts. Train your LoRA, point Shimmy at it, done." — Real developer workflow

## 🎯 The Problem Shimmy Solves

```bash
# You have this after training:
my-awesome-coding-lora/
├── adapter_model.safetensors  ← Your trained LoRA
├── adapter_config.json
└── training_results.json

# You want this:
curl -X POST http://localhost:11435/api/generate \
  -d '{"model":"phi3-lora","prompt":"def fibonacci(n):"}'
```

**Before Shimmy:** Convert SafeTensors → GGUF → Configure llama.cpp → Debug APIs  
**With Shimmy:** Point and serve. Done.

## 🚀 Zero-Friction LoRA Serving

| Step | Before Shimmy | With Shimmy |
|------|---------------|-------------|
| **Convert** | Find llama.cpp scripts, debug formats | Auto-handled |
| **Configure** | Manual server setup, ports, contexts | Zero-config |
| **Serve** | Complex llama.cpp command lines | `shimmy serve` |
| **Test** | Figure out API format | Standard OpenAI API |
| **Time** | 15-30 minutes, error-prone | **30 seconds** |

## Why Shimmy?

**Privacy**: Your code stays on your machine  
**Cost**: No per-token pricing, unlimited queries  
**Speed**: Local inference = sub-second responses  
**Integration**: Works with VSCode, Cursor, Continue.dev out of the box  

## Quick Start (30 seconds)

```bash
# Install from crates.io
cargo install shimmy --features llama

# Or download pre-built binary
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy.exe

# Get any GGUF model (example: Phi-3 Mini)
# Place in ./models/ or set SHIMMY_BASE_GGUF=path/to/model.gguf

# Start serving  
./shimmy serve

# Point your AI tools to http://localhost:11435
# VSCode Copilot, Cursor, Continue.dev all work instantly
```

[📖 Full quick start guide](docs/quickstart.md)

## Integration Examples

**VSCode Copilot**:
```json
// settings.json
{
  "github.copilot.advanced": {
    "serverUrl": "http://localhost:11435"
  }
}
```

**Continue.dev**:
```json
{
  "models": [{
    "title": "Local Shimmy",
    "provider": "openai", 
    "model": "your-model-name",
    "apiBase": "http://localhost:11435/v1"
  }]
}
```

[🔗 See all integrations](docs/integrations.md)

## Why Shimmy Will Always Be Free

I built Shimmy because I was tired of 680MB binaries to run a 4GB model.

**This is my commitment**: Shimmy stays MIT licensed, forever. If you want to support development, [sponsor it](https://github.com/sponsors/Michael-A-Kuykendall). If you don't, just build something cool with it.

> Shimmy saves you time and money. If it's useful, consider sponsoring for $5/month — less than your Netflix subscription, infinitely more useful.

## Performance vs Competition

[📊 See detailed benchmarks](docs/benchmarks.md)

| Tool | Binary | Startup | Memory | OpenAI API |
|------|--------|---------|--------|------------|
| **Shimmy** | **5.1MB** | **<100ms** | **50MB** | **100%** |
| Ollama | 680MB | 5-10s | 200MB+ | Partial |
| llama.cpp | 89MB | 1-2s | 100MB | None |

## Community & Support

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)
- **📖 Documentation**: [docs/](docs/)
- **💝 Sponsorship**: [GitHub Sponsors](https://github.com/sponsors/Michael-A-Kuykendall)

### Weekly Showcase

**What did you build with Shimmy this week?** Share in [Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions) and get featured!

## Sponsors

See our amazing [sponsors](SPONSORS.md) who make Shimmy possible! 🙏

### Sponsorship Tiers

- **$5/month**: Coffee tier - My eternal gratitude + sponsor badge
- **$25/month**: Bug prioritizer - Priority support + name in SPONSORS.md  
- **$100/month**: Corporate backer - Logo on README + monthly office hours
- **$500/month**: Infrastructure partner - Direct support + roadmap input

**Companies**: Need invoicing? Email [sponsors@shimmy.dev](mailto:sponsors@shimmy.dev)

## Technical Architecture

- **Rust + Tokio**: Memory-safe, async performance
- **llama.cpp backend**: Industry-standard GGUF inference
- **OpenAI API compatibility**: Drop-in replacement
- **Zero-config auto-discovery**: Just works™

### API Endpoints
- `GET /health` - Health check
- `POST /v1/chat/completions` - OpenAI-compatible chat
- `GET /v1/models` - List available models
- `POST /api/generate` - Shimmy native API
- `GET /ws/generate` - WebSocket streaming

### CLI Commands
```bash
./shimmy serve                    # Start server
./shimmy list                     # Show available models  
./shimmy discover                 # Refresh model discovery
./shimmy generate --name X --prompt "Hi"  # Test generation
./shimmy probe model-name         # Verify model loads
```

## License & Philosophy

MIT License - forever and always.

**Philosophy**: Infrastructure should be invisible. Shimmy is infrastructure.

---

**Forever maintainer**: Michael A. Kuykendall  
**Promise**: This will never become a paid product  
**Mission**: Making local AI development frictionless

*"The best code is code you don't have to think about."*
