<div align="center">
  <img src="assets/shimmy-logo.png" alt="Shimmy Logo" width="300" height="auto" />

  # The Lightweight OpenAI API Server

  ### 🔒 Local Inference Without Dependencies 🚀

  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![Security](https://img.shields.io/badge/Security-Audited-green)](https://github.com/Michael-A-Kuykendall/shimmy/security)
  [![Crates.io](https://img.shields.io/crates/v/shimmy.svg)](https://crates.io/crates/shimmy)
  [![Downloads](https://img.shields.io/crates/d/shimmy.svg)](https://crates.io/crates/shimmy)
  [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
  [![GitHub Stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=social)](https://github.com/Michael-A-Kuykendall/shimmy/stargazers)

  [![💝 Sponsor this project](https://img.shields.io/badge/💝_Sponsor_this_project-ea4aaa?style=for-the-badge&logo=github&logoColor=white)](https://github.com/sponsors/Michael-A-Kuykendall)
</div>

**Shimmy will be free forever.** No asterisks. No "free for now." No pivot to paid.

### 💝 Support Shimmy's Growth

🚀 **If Shimmy helps you, consider [sponsoring](https://github.com/sponsors/Michael-A-Kuykendall) — 100% of support goes to keeping it free forever.**

- **$5/month**: Coffee tier ☕ - Eternal gratitude + sponsor badge
- **$25/month**: Bug prioritizer 🐛 - Priority support + name in [SPONSORS.md](SPONSORS.md)
- **$100/month**: Corporate backer 🏢 - Logo placement + monthly office hours
- **$500/month**: Infrastructure partner 🚀 - Direct support + roadmap input

[**🎯 Become a Sponsor**](https://github.com/sponsors/Michael-A-Kuykendall) | See our amazing [sponsors](SPONSORS.md) 🙏

---

## Drop-in OpenAI API Replacement for Local LLMs

Shimmy is a **single-binary** that provides **100% OpenAI-compatible endpoints** for GGUF models. Point your existing AI tools to Shimmy and they just work — locally, privately, and free.

**🎉 NEW in v2.0.0**: Shimmy now runs on [Airframe](#-airframe-engine), a pure-Rust WGSL GPU engine. No C++ toolchain, no backend flags, no compilation required.

## 🔥 Airframe Engine

Starting in v2.0.0, Shimmy's default inference engine is **Airframe** — a pure-Rust WebGPU (WGSL) transformer runtime built from scratch.

**Why this matters:**
- No C++ toolchain required — Rust only, top to bottom
- F32 precision throughout for deterministic, high-quality output
- WGSL compute shaders work on any GPU via WebGPU (NVIDIA, AMD, Intel, integrated)
- Model spec auto-derived from GGUF metadata — no hardcoded per-model constants
- YaRN RoPE scaling for extended context: set `SHIMMY_MAX_CTX=8192` and go

**Quick start with Airframe (v2.0.0+):**
```bash
# Set your model path and start serving
SHIMMY_BASE_GGUF=/path/to/TinyLlama-1.1B-Chat-v1.0.Q4_0.gguf ./shimmy serve

# Extended context (4096 tokens, YaRN RoPE enabled automatically)
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=4096 ./shimmy serve
```

## 📦 Migrating from v1.x

The llama.cpp backend is **removed in v2.0.0**. The Airframe engine is the only inference path.
See [docs/MIGRATION_v2.md](docs/MIGRATION_v2.md) for the step-by-step migration guide.

## Developer Tools

Whether you're forking Shimmy or integrating it as a service, we provide complete documentation and integration templates.

### Try it in 30 seconds

```bash
# 1) Download pre-built binary
# Windows:
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe
set SHIMMY_BASE_GGUF=C:\path\to\model.gguf && ./shimmy.exe serve &

# Linux / macOS:
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy && chmod +x shimmy
SHIMMY_BASE_GGUF=/path/to/model.gguf ./shimmy serve &

# 2) See registered models
./shimmy list

# 3) Smoke test the OpenAI API
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
        "model":"tinyllama-1.1b",
        "messages":[{"role":"user","content":"Say hi in 5 words."}],
        "max_tokens":32
      }' | jq -r '.choices[0].message.content'
```

## 🚀 Compatible with OpenAI SDKs and Tools

**No code changes needed** - just change the API endpoint:

- **Any OpenAI client**: Python, Node.js, curl, etc.
- **Development applications**: Compatible with standard SDKs
- **VSCode Extensions**: Point to `http://localhost:11435`
- **Cursor Editor**: Built-in OpenAI compatibility
- **Continue.dev**: Drop-in model provider

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

- **Automatically finds models** from Hugging Face cache, Ollama, local dirs
- **Auto-allocates ports** to avoid conflicts
- **Auto-detects LoRA adapters** for specialized models
- **Just works** - no config files, no setup wizards

## 🧠 Advanced MOE (Mixture of Experts) Support

> **Note**: MoE (Mixture of Experts) CPU offloading is on the Airframe roadmap. See [docs/AIRFRAME_MOE_ROADMAP.md](docs/AIRFRAME_MOE_ROADMAP.md) for the implementation plan.

**Run 70B+ models on consumer hardware** — coming to the Airframe engine. Track progress on the [roadmap](docs/ROADMAP.md).

**Perfect for**: Large models (70B+), limited VRAM systems, cost-effective inference

## 🎯 Perfect for Local Development

- **Privacy**: Your code never leaves your machine
- **Cost**: No API keys, no per-token billing
- **Speed**: Local inference, sub-second responses
- **Reliability**: No rate limits, no downtime

## Quick Start (30 seconds)

### Installation

**v2.0.0**: Download pre-built binaries with Airframe WebGPU engine included!

#### **📥 Pre-Built Binaries (Recommended — Zero Dependencies)**

Pick your platform and download — no compilation needed, GPU acceleration included:

```bash
# Windows x64 (Airframe WebGPU engine)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# Linux x86_64 (Airframe WebGPU engine)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy && chmod +x shimmy

# macOS ARM64 (Airframe with Metal backend via wgpu)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy && chmod +x shimmy

# macOS Intel
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-intel -o shimmy && chmod +x shimmy

# Linux ARM64 (huggingface engine; Airframe cross-compilation not yet supported)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-aarch64 -o shimmy && chmod +x shimmy
```

**That's it!** The Airframe WebGPU adapter is selected automatically at runtime.

#### **🛠️ Build from Source / cargo install**

```bash
# Install from crates.io (huggingface engine — works without GPU)
cargo install shimmy

# Build from source with Airframe GPU engine (requires airframe submodule)
git clone https://github.com/Michael-A-Kuykendall/shimmy --recurse-submodules
cd shimmy
cargo build --release --features airframe,huggingface
```

> **Note**: The GitHub Releases binaries already include the Airframe engine. Building from source with `--features airframe` is for contributors or custom builds.

### GPU Acceleration

**v2.0.0**: Airframe uses **WebGPU (wgpu)** for GPU acceleration. No backend flags, no driver installation beyond standard OS graphics drivers.

#### **📥 Download Pre-Built Binaries (Recommended)**

Release binaries include the Airframe engine with WebGPU support compiled in:

| Platform | Download | GPU Backend | Notes |
|----------|----------|-------------|-------|
| **Windows x64** | [shimmy-windows-x86_64.exe](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe) | WebGPU (wgpu) | NVIDIA, AMD, Intel |
| **Linux x86_64** | [shimmy-linux-x86_64](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64) | WebGPU (wgpu) | NVIDIA, AMD, Intel |
| **macOS ARM64** | [shimmy-macos-arm64](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64) | Metal (via wgpu) | Apple Silicon |
| **macOS Intel** | [shimmy-macos-intel](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-intel) | Metal (via wgpu) | Intel Mac |
| **Linux ARM64** | [shimmy-linux-aarch64](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-aarch64) | huggingface only | ARM cross-build |

#### **🎯 How GPU Selection Works**

Airframe uses wgpu's adapter enumeration. On first launch it selects the best available GPU adapter for your system — discrete GPU preferred over integrated, integrated over CPU fallback. No configuration needed.

```bash
# Check selected adapter
shimmy gpu-info

# Start serving (GPU adapter auto-selected)
shimmy serve
```

#### **🔧 Extended Context**

```bash
# YaRN RoPE scaling enables extended context automatically when SHIMMY_MAX_CTX is set
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=8192 shimmy serve
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

Point your development tools to the displayed port — VSCode Copilot, Cursor, Continue.dev all work instantly.

## 📦 Download & Install

### Package Managers
- **Rust**: [`cargo install shimmy`](https://crates.io/crates/shimmy) *(installs huggingface engine; for Airframe GPU, use GitHub Releases binaries)*
- **VS Code**: [Shimmy Extension](https://marketplace.visualstudio.com/items?itemName=targetedwebresults.shimmy-vscode)
- **npm**: `npm install -g shimmy-js` *(planned)*
- **Python**: `pip install shimmy` *(planned)*

### Direct Downloads
- **GitHub Releases**: [Latest binaries](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest)
- **Docker**: `docker pull shimmy/shimmy:latest` *(coming soon)*

### 🍎 macOS Support

**Full compatibility confirmed!** Shimmy works on macOS with Metal GPU acceleration via wgpu.

```bash
# Install from crates.io (huggingface engine)
cargo install shimmy

# For Airframe GPU engine, download the macOS binary from GitHub Releases:
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy && chmod +x shimmy
```

**✅ Verified working:**
- Intel and Apple Silicon Macs
- Metal GPU acceleration via wgpu (automatic on Apple Silicon)
- Xcode 17+ compatibility

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

I built Shimmy to retain privacy-first control on my AI development and keep things local and lean.

**This is my commitment**: Shimmy stays MIT licensed, forever. If you want to support development, [sponsor it](https://github.com/sponsors/Michael-A-Kuykendall). If you don't, just build something cool with it.

> 💡 **Shimmy saves you time and money. If it's useful, consider [sponsoring for $5/month](https://github.com/sponsors/Michael-A-Kuykendall) — less than your Netflix subscription, infinitely more useful for developers.**

## API Reference

### Endpoints
- `GET /health` - Health check
- `POST /v1/chat/completions` - OpenAI-compatible chat
- `GET /v1/models` - List available models
- `POST /api/generate` - Shimmy native API
- `GET /ws/generate` - WebSocket streaming

### CLI Commands
```bash
shimmy serve                              # Start server (auto port allocation)
shimmy serve --bind 127.0.0.1:8080        # Manual port binding
shimmy serve --gpu-backend auto           # WebGPU adapter auto-select (default)
shimmy serve --gpu-backend cpu            # Force CPU (disable GPU)
shimmy list                               # Show available models
shimmy discover                           # Refresh model discovery
shimmy generate --name X --prompt "Hi"   # Test generation
shimmy probe model-name                   # Verify model loads
shimmy gpu-info                           # Show selected WebGPU adapter
```

## Technical Architecture

- **Rust + Tokio**: Memory-safe, async performance
- **Airframe engine**: Pure-Rust WGSL GPU inference — no C++ toolchain, deterministic output, GGUF-native
- **OpenAI API compatibility**: Drop-in replacement
- **Dynamic port management**: Zero conflicts, auto-allocation
- **Zero-config auto-discovery**: Just works™

### 🚀 Advanced Features

- **🧠 MOE CPU Offloading**: Hybrid GPU/CPU processing for large models (70B+)
- **🎯 Smart Model Filtering**: Automatically excludes non-language models (Stable Diffusion, Whisper, CLIP)
- **🛡️ 6-Gate Release Validation**: Constitutional quality limits ensure reliability
- **⚡ Smart Model Preloading**: Background loading with usage tracking for instant model switching
- **💾 Response Caching**: LRU + TTL cache delivering 20-40% performance gains on repeat queries
- **🚀 Integration Templates**: One-command deployment for Docker, Kubernetes, Railway, Fly.io, FastAPI, Express
- **🔄 Request Routing**: Multi-instance support with health checking and load balancing
- **📊 Advanced Observability**: Real-time metrics with self-optimization and Prometheus integration
- **🔗 RustChain Integration**: Universal workflow transpilation with workflow orchestration

## Community & Support

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)
- **📖 Documentation**: [docs/](docs/) • [Migration Guide v1→v2](docs/MIGRATION_v2.md) • [Engineering Methodology](docs/METHODOLOGY.md) • [OpenAI Compatibility Matrix](docs/OPENAI_COMPAT.md) • [Benchmarks (Reproducible)](docs/BENCHMARKS.md)
- **💝 Sponsorship**: [GitHub Sponsors](https://github.com/sponsors/Michael-A-Kuykendall)

### Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Michael-A-Kuykendall/shimmy&type=Timeline)](https://www.star-history.com/#Michael-A-Kuykendall/shimmy&Timeline)

### 🚀 Momentum Snapshot

🌟 **![GitHub stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=flat&color=yellow) stars and climbing fast**
⏱ **<1s startup**
🦀 **100% Rust, no Python**

### 📰 As Featured On

🔥 [**Hacker News**](https://news.ycombinator.com/item?id=45130322) • [**Front Page Again**](https://news.ycombinator.com/item?id=45199898) • [**IPE Newsletter**](https://ipenewsletter.substack.com/p/the-strange-new-side-hustles-of-openai)

**Companies**: Need invoicing? Email [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)

## ⚡ Performance Comparison

| Tool | Startup Time | Memory Usage | OpenAI API |
|------|--------------|--------------|------------|
| **Shimmy** | **<100ms** | **50MB** | **100%** |
| Ollama | 5-10s | 200MB+ | Partial |

## Quality & Reliability

Shimmy maintains high code quality through comprehensive testing:

- **Comprehensive test suite** with property-based testing
- **Automated CI/CD pipeline** with quality gates
- **Runtime invariant checking** for critical operations
- **Cross-platform compatibility testing**
### Development Testing

Run the complete test suite:

```bash
# Using cargo aliases
cargo test-quick           # Quick development tests

# Using Makefile  
make test                  # Full test suite
make test-quick            # Quick development tests
```

See our [testing approach](docs/ppt-invariant-testing.md) for technical details.

---

## License & Philosophy

MIT License - forever and always.

**Philosophy**: Infrastructure should be invisible. Shimmy is infrastructure.

**Testing Philosophy**: Reliability through comprehensive validation and property-based testing.

---

**Forever maintainer**: Michael A. Kuykendall
**Promise**: This will never become a paid product
**Mission**: Making local model inference simple and reliable
