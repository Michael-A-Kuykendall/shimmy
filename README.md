<div align="center">
  <img src="assets/shimmy-logo.png" alt="Shimmy Logo" width="300" height="auto" />
  
  # The 5MB Alternative to Ollama

  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![Crates.io](https://img.shields.io/crates/v/shimmy.svg)](https://crates.io/crates/shimmy)
  [![Downloads](https://img.shields.io/crates/d/shimmy.svg)](https://crates.io/crates/shimmy)
  [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
  [![GitHub Stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=social)](https://github.com/Michael-A-Kuykendall/shimmy/stargazers)
  [![Sponsor](https://img.shields.io/badge/❤️-Sponsor-ea4aaa?logo=github)](https://github.com/sponsors/Michael-A-Kuykendall)
</div>

**Shimmy will be free forever.** No asterisks. No "free for now." No pivot to paid.

## Drop-in OpenAI API Replacement for Local LLMs

Shimmy is a **5.1MB single-binary** that provides **100% OpenAI-compatible endpoints** for GGUF models. Point your existing AI tools to Shimmy and they just work — locally, privately, and free.

```bash
# Install and run in 30 seconds
cargo install shimmy --features huggingface
shimmy serve
# → Running on http://localhost:11435
```

### Try it in 30 seconds

```bash
# 1) Install + run
cargo install shimmy --features huggingface
shimmy serve &

# 2) See models and pick one
shimmy list

# 3) Smoke test the OpenAI API
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
        "model":"REPLACE_WITH_MODEL_FROM_list",
        "messages":[{"role":"user","content":"Say hi in 5 words."}],
        "max_tokens":32
      }' | jq -r '.choices[0].message.content'
```

## 🚀 Works with Your Existing Tools

**No code changes needed** - just change the API endpoint:

- **VSCode Extensions**: Point to `http://localhost:11435`
- **Cursor Editor**: Built-in OpenAI compatibility  
- **Continue.dev**: Drop-in model provider
- **Any OpenAI client**: Python, Node.js, curl, etc.

### Use with OpenAI SDKs

- Node.js (openai v4)

```ts
import OpenAI from "openai";

const openai = new OpenAI({
  baseURL: "http://127.0.0.1:11435/v1",
  apiKey: "sk-local", // placeholder, Shimmy ignores it
});

const resp = await openai.chat.completions.create({
  model: "REPLACE_WITH_MODEL",
  messages: [{ role: "user", content: "Say hi in 5 words." }],
  max_tokens: 32,
});

console.log(resp.choices[0].message?.content);
```

- Python (openai>=1.0.0)

```python
from openai import OpenAI

client = OpenAI(base_url="http://127.0.0.1:11435/v1", api_key="sk-local")

resp = client.chat.completions.create(
    model="REPLACE_WITH_MODEL",
    messages=[{"role": "user", "content": "Say hi in 5 words."}],
    max_tokens=32,
)

print(resp.choices[0].message.content)
```

## ⚡ Zero Configuration Required

- **Auto-discovers models** from Hugging Face cache, Ollama, local dirs
- **Auto-allocates ports** to avoid conflicts
- **Auto-detects LoRA adapters** for specialized models
- **Just works** - no config files, no setup wizards

## 🎯 Perfect for Local Development

- **Privacy**: Your code never leaves your machine
- **Cost**: No API keys, no per-token billing  
- **Speed**: Local inference, sub-second responses
- **Reliability**: No rate limits, no downtime

## Quick Start (30 seconds)

### Installation

#### **🪟 Windows**
```bash
# RECOMMENDED: Use pre-built binary (no build dependencies required)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy.exe -o shimmy.exe

# OR: Install from source (requires LLVM/Clang)
# First install build dependencies:
winget install LLVM.LLVM
# Then install shimmy:
cargo install shimmy --features huggingface
```

> **⚠️ Windows Notes**: 
> - **Pre-built binary recommended** to avoid build dependency issues
> - If Windows Defender flags the binary, add an exclusion or use `cargo install`
> - For `cargo install`: Install [LLVM](https://releases.llvm.org/download.html) first to resolve `libclang.dll` errors

#### **🍎 macOS / 🐧 Linux**
```bash
# Install from crates.io
cargo install shimmy --features huggingface
```

### Get Models

Shimmy auto-discovers models from:
- **Hugging Face cache**: `~/.cache/huggingface/hub/`
- **Ollama models**: `~/.ollama/models/`
- **Local directory**: `./models/`
- **Environment**: `SHIMMY_BASE_GGUF=path/to/model.gguf`

```bash
# Download models that work out of the box
huggingface-cli download microsoft/Phi-3-mini-4k-instruct-gguf --local-dir ./models/
huggingface-cli download bartowski/Llama-3.2-1B-Instruct-GGUF --local-dir ./models/
```

### Start Server

```bash
# Auto-allocates port to avoid conflicts
shimmy serve

# Or use manual port
shimmy serve --bind 127.0.0.1:11435
```

Point your AI tools to the displayed port — VSCode Copilot, Cursor, Continue.dev all work instantly.

## 📦 Download & Install

### Package Managers
- **Rust**: [`cargo install shimmy`](https://crates.io/crates/shimmy)
- **VS Code**: [Shimmy Extension](https://marketplace.visualstudio.com/items?itemName=targetedwebresults.shimmy-vscode)
- **npm**: `npm install -g shimmy-js` *(coming soon)*
- **Python**: `pip install shimmy` *(coming soon)*

### Direct Downloads
- **GitHub Releases**: [Latest binaries](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest)
- **Docker**: `docker pull shimmy/shimmy:latest` *(coming soon)*

### 🍎 macOS Support

**Full compatibility confirmed!** Shimmy works flawlessly on macOS with Metal GPU acceleration.

```bash
# Install dependencies
brew install cmake rust

# Install shimmy
cargo install shimmy
```

**✅ Verified working:**
- Intel and Apple Silicon Macs
- Metal GPU acceleration (automatic)
- Xcode 17+ compatibility
- All LoRA adapter features

## Integration Examples

### VSCode Copilot
```json
{
  "github.copilot.advanced": {
    "serverUrl": "http://localhost:11435"
  }
}
```

### Continue.dev
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

### Cursor IDE
Works out of the box - just point to `http://localhost:11435/v1`

## Why Shimmy Will Always Be Free

I built Shimmy because I was tired of 680MB binaries to run a 4GB model.

**This is my commitment**: Shimmy stays MIT licensed, forever. If you want to support development, [sponsor it](https://github.com/sponsors/Michael-A-Kuykendall). If you don't, just build something cool with it.

> Shimmy saves you time and money. If it's useful, consider sponsoring for $5/month — less than your Netflix subscription, infinitely more useful.

## Performance Comparison

| Tool | Binary Size | Startup Time | Memory Usage | OpenAI API |
|------|-------------|--------------|--------------|------------|
| **Shimmy** | **5.1MB** | **<100ms** | **50MB** | **100%** |
| Ollama | 680MB | 5-10s | 200MB+ | Partial |
| llama.cpp | 89MB | 1-2s | 100MB | None |

## API Reference

### Endpoints
- `GET /health` - Health check
- `POST /v1/chat/completions` - OpenAI-compatible chat
- `GET /v1/models` - List available models
- `POST /api/generate` - Shimmy native API
- `GET /ws/generate` - WebSocket streaming

### CLI Commands
```bash
shimmy serve                    # Start server (auto port allocation)
shimmy serve --bind 127.0.0.1:8080  # Manual port binding
shimmy list                     # Show available models  
shimmy discover                 # Refresh model discovery
shimmy generate --name X --prompt "Hi"  # Test generation
shimmy probe model-name         # Verify model loads
```

## Technical Architecture

- **Rust + Tokio**: Memory-safe, async performance
- **llama.cpp backend**: Industry-standard GGUF inference
- **OpenAI API compatibility**: Drop-in replacement
- **Dynamic port management**: Zero conflicts, auto-allocation
- **Zero-config auto-discovery**: Just works™

## Community & Support

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)
- **📖 Documentation**: [docs/](docs/)
- **💝 Sponsorship**: [GitHub Sponsors](https://github.com/sponsors/Michael-A-Kuykendall)

### Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Michael-A-Kuykendall/shimmy&type=Date)](https://star-history.com/#Michael-A-Kuykendall/shimmy&Date)

### Sponsors

See our amazing [sponsors](SPONSORS.md) who make Shimmy possible! 🙏

**Sponsorship Tiers:**
- **$5/month**: Coffee tier - My eternal gratitude + sponsor badge
- **$25/month**: Bug prioritizer - Priority support + name in SPONSORS.md  
- **$100/month**: Corporate backer - Logo on README + monthly office hours
- **$500/month**: Infrastructure partner - Direct support + roadmap input

**Companies**: Need invoicing? Email [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)

## Quality & Reliability

Shimmy maintains high code quality through comprehensive testing:

- **Comprehensive test suite** with property-based testing
- **Automated CI/CD pipeline** with quality gates
- **Runtime invariant checking** for critical operations
- **Cross-platform compatibility testing**

See our [testing approach](docs/ppt-invariant-testing.md) for technical details.

---

## License & Philosophy

MIT License - forever and always.

**Philosophy**: Infrastructure should be invisible. Shimmy is infrastructure.

**Testing Philosophy**: Reliability through comprehensive validation and property-based testing.

---

**Forever maintainer**: Michael A. Kuykendall  
**Promise**: This will never become a paid product  
**Mission**: Making local AI development frictionless