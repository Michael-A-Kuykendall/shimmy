# Shimmy: The 5MB Alternative to Ollama

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
[![Sponsor](https://img.shields.io/badge/❤️-Sponsor-ea4aaa?logo=github)](https://github.com/sponsors/Michael-A-Kuykendall)

**Shimmy will be free forever.** No asterisks. No "free for now." No pivot to paid.

> "Infrastructure should be invisible. Shimmy is infrastructure." — Michael A. Kuykendall

## What is Shimmy?

Shimmy is a **5.1MB single-binary** local inference server that provides OpenAI API-compatible endpoints for GGUF models. It's designed to be the **invisible infrastructure** that just works.

| Metric | Shimmy | Ollama | 
|--------|--------|--------|
| **Binary Size** | 5.1MB 🏆 | 680MB |
| **Startup Time** | <100ms 🏆 | 5-10s |
| **Memory Overhead** | <50MB 🏆 | 200MB+ |
| **OpenAI Compatibility** | 100% 🏆 | Partial |
| **Port Management** | Auto 🏆 | Manual |
| **Configuration** | Zero 🏆 | Manual |

## 🎯 Perfect for Developers

**Privacy**: Your code stays on your machine  
**Cost**: No per-token pricing, unlimited queries  
**Speed**: Local inference = sub-second responses  
**Integration**: Works with VSCode, Cursor, Continue.dev out of the box  

**BONUS:** First-class LoRA adapter support - from training to production API in 30 seconds.  

## Quick Start (30 seconds)

```bash
# Install from crates.io (Linux, macOS, Windows)
cargo install shimmy

# Or download pre-built binary (Windows only)
# https://github.com/Michael-A-Kuykendall/shimmy/releases/latest
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy.exe
```

> **⚠️ Windows Security Notice**: Windows Defender may flag the binary as a false positive. This is common with unsigned Rust executables. **Recommended**: Use `cargo install shimmy` instead, or add an exclusion for shimmy.exe in Windows Defender.

# Get any GGUF model - Shimmy auto-discovers from:
# • Hugging Face cache: ~/.cache/huggingface/hub/
# • Local directory: ./models/
# • Environment: SHIMMY_BASE_GGUF=path/to/model.gguf
# 
# Examples that work out of the box:
# huggingface-cli download microsoft/Phi-3-mini-4k-instruct-gguf --local-dir ./models/
# huggingface-cli download bartowski/Llama-3.2-1B-Instruct-GGUF --local-dir ./models/

# Start serving (auto-allocates port to avoid conflicts)
./shimmy serve

# Point your AI tools to the displayed port
# VSCode Copilot, Cursor, Continue.dev all work instantly
# OR use manual port: ./shimmy serve --bind 127.0.0.1:11435
```

[📖 Full quick start guide](docs/quickstart.md)

## 📦 Download & Install

### Package Managers
- **Rust**: [`cargo install shimmy`](https://crates.io/crates/shimmy)
- **VS Code**: [Shimmy Extension](https://marketplace.visualstudio.com/items?itemName=targetedwebresults.shimmy-vscode)
- **npm**: [`npm install -g shimmy-js`](https://www.npmjs.com/package/shimmy-js) *(coming soon)*
- **Python**: [`pip install shimmy`](https://pypi.org/project/shimmy/) *(coming soon)*

### Direct Downloads
- **GitHub Releases**: [Latest binaries for all platforms](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest)
- **Docker**: `docker pull shimmy/shimmy:latest` *(coming soon)*

---

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

**Companies**: Need invoicing? Email [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)

## Technical Architecture

- **Rust + Tokio**: Memory-safe, async performance
- **llama.cpp backend**: Industry-standard GGUF inference
- **OpenAI API compatibility**: Drop-in replacement
- **Dynamic port management**: Zero conflicts, auto-allocation
- **Zero-config auto-discovery**: Just works™

### API Endpoints
- `GET /health` - Health check
- `POST /v1/chat/completions` - OpenAI-compatible chat
- `GET /v1/models` - List available models
- `POST /api/generate` - Shimmy native API
- `GET /ws/generate` - WebSocket streaming

### CLI Commands
```bash
./shimmy serve                    # Start server (auto port allocation)
./shimmy serve --bind 127.0.0.1:8080  # Manual port binding
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
