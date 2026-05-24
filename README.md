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

**🎉 NEW in v1.9.0**: One download, all GPU backends included! No compilation, no backend confusion - just download and run.

## Developer Tools

Whether you're forking Shimmy or integrating it as a service, we provide complete documentation and integration templates.

### Try it in 30 seconds

```bash
# 1) Download pre-built binary (includes all GPU backends)
# Windows:
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe
./shimmy.exe serve &

# Linux:
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy && chmod +x shimmy
./shimmy serve &

# macOS (Apple Silicon):
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy && chmod +x shimmy
./shimmy serve &

# 2) See models and pick one
./shimmy list

# 3) Smoke test the OpenAI API
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
        "model":"REPLACE_WITH_MODEL_FROM_list",
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

**Run 70B+ models on consumer hardware** with intelligent CPU/GPU hybrid processing:

- **🔄 CPU MOE Offloading**: Automatically distribute model layers across CPU and GPU
- **🧮 Intelligent Layer Placement**: Optimizes which layers run where for maximum performance
- **💾 Memory Efficiency**: Fit larger models in limited VRAM by using system RAM strategically
- **⚡ Hybrid Acceleration**: Get GPU speed where it matters most, CPU reliability everywhere else
- **🎛️ Configurable**: `--cpu-moe` and `--n-cpu-moe` flags for fine control

```bash
# Enable MOE CPU offloading during installation
cargo install shimmy --features moe

# Run with MOE hybrid processing
shimmy serve --cpu-moe --n-cpu-moe 8

# Automatically balances: GPU layers (fast) + CPU layers (memory-efficient)
```

**Perfect for**: Large models (70B+), limited VRAM systems, cost-effective inference

## 🎯 Perfect for Local Development

- **Privacy**: Your code never leaves your machine
- **Cost**: No API keys, no per-token billing
- **Speed**: Local inference, sub-second responses
- **Reliability**: No rate limits, no downtime

## Quick Start (30 seconds)

### Installation

**✨ v1.9.0 NEW**: Download pre-built binaries with ALL GPU backends included!

#### **📥 Pre-Built Binaries (Recommended - Zero Dependencies)**

Pick your platform and download - no compilation needed:

```bash
# Windows x64 (includes CUDA + Vulkan + OpenCL)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# Linux x86_64 (includes CUDA + Vulkan + OpenCL)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy && chmod +x shimmy

# macOS ARM64 (includes MLX for Apple Silicon)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy && chmod +x shimmy

# macOS Intel (CPU-only)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-intel -o shimmy && chmod +x shimmy

# Linux ARM64 (CPU-only)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-aarch64 -o shimmy && chmod +x shimmy
```

**That's it!** Your GPU will be detected automatically at runtime.

#### **🛠️ Build from Source (Advanced)**

Want to customize or contribute?

```bash
# Basic installation (CPU only)
cargo install shimmy --features huggingface

# Kitchen Sink builds (what pre-built binaries use):
# Windows/Linux x64:
cargo install shimmy --features huggingface,llama,llama-cuda,llama-vulkan,llama-opencl

# macOS ARM64:
cargo install shimmy --features huggingface,llama,mlx

# CPU-only (any platform):
cargo install shimmy --features huggingface,llama
```

> **⚠️ Build Notes**:
> - **Windows**: Install [LLVM](https://releases.llvm.org/download.html) first for libclang.dll
> - **Recommended**: Use pre-built binaries to avoid dependency issues
> - **Advanced users only**: Building from source requires C++ compiler + CUDA/Vulkan SDKs

### GPU Acceleration

**✨ NEW in v1.9.0**: One binary per platform with automatic GPU detection.

#### **📥 Download Pre-Built Binaries (Recommended)**

No compilation needed! Each binary includes ALL GPU backends for your platform:

| Platform | Download | GPU Support | Auto-Detects |
|----------|----------|-------------|--------------|
| **Windows x64** | [shimmy-windows-x86_64.exe](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe) | CUDA + Vulkan + OpenCL | ✅ |
| **Linux x86_64** | [shimmy-linux-x86_64](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64) | CUDA + Vulkan + OpenCL | ✅ |
| **macOS ARM64** | [shimmy-macos-arm64](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64) | MLX (Apple Silicon) | ✅ |
| **macOS Intel** | [shimmy-macos-intel](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-intel) | CPU only | N/A |
| **Linux ARM64** | [shimmy-linux-aarch64](https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-aarch64) | CPU only | N/A |

**How it works**: Download one file, run it. Shimmy automatically detects and uses your GPU!

```bash
# Windows example
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe
./shimmy.exe serve --gpu-backend auto  # Auto-detects CUDA/Vulkan/OpenCL

# Linux example  
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy
chmod +x shimmy
./shimmy serve --gpu-backend auto  # Auto-detects CUDA/Vulkan/OpenCL

# macOS ARM64 example
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy
chmod +x shimmy  
./shimmy serve  # Auto-detects MLX on Apple Silicon
```

#### **🎯 GPU Auto-Detection**

Shimmy uses intelligent GPU detection with this priority order:

1. **CUDA** (NVIDIA GPUs via nvidia-smi)
2. **Vulkan** (Cross-platform GPUs via vulkaninfo)
3. **OpenCL** (AMD/Intel GPUs via clinfo)
4. **MLX** (Apple Silicon via system detection)
5. **CPU** (Fallback if no GPU detected)

**No manual configuration needed!** Just run with `--gpu-backend auto` (default).

#### **🔧 Manual Backend Override**

Want to force a specific backend? Use the `--gpu-backend` flag:

```bash
# Auto-detect (default - recommended)
shimmy serve --gpu-backend auto

# Force CPU (for testing or compatibility)
shimmy serve --gpu-backend cpu

# Force CUDA (NVIDIA GPUs only)
shimmy serve --gpu-backend cuda

# Force Vulkan (AMD/Intel/Cross-platform)
shimmy serve --gpu-backend vulkan

# Force OpenCL (AMD/Intel alternative)
shimmy serve --gpu-backend opencl
```

**🛡️ Error Handling & Robustness**: If you force an unavailable backend (e.g., `--gpu-backend cuda` on AMD GPU), Shimmy will:
1. ✅ Display clear error message explaining the issue
2. ✅ Automatically fallback to next available backend in priority order
3. ✅ Log which backend was actually used (check with `--verbose`)
4. ✅ Continue serving requests (graceful degradation, no crashes)
5. ✅ Support environment variable override: `SHIMMY_GPU_BACKEND=cuda`

**Common scenarios**:
- `--gpu-backend cuda` on non-NVIDIA → Falls back to Vulkan or OpenCL
- `--gpu-backend vulkan` without drivers → Falls back to OpenCL or CPU
- `--gpu-backend invalid` → Clear error + fallback to auto-detection
- No GPU detected → Runs on CPU with performance warning

**Environment Variable**: Set `SHIMMY_GPU_BACKEND=cuda` to override default without CLI flags.

#### **🔍 Check GPU Support**
```bash
# Show detected GPU backends
shimmy gpu-info

# Check which backend is being used
shimmy serve --gpu-backend auto --verbose
```

#### **⚡ Binary Sizes**

- **GPU-enabled binaries** (Windows/Linux x64, macOS ARM64): ~40-50MB
- **CPU-only binaries** (macOS Intel, Linux ARM64): ~20-30MB

Trade-off: Slightly larger binaries for zero compilation and automatic GPU detection.

#### **🛠️ Build from Source (Advanced)**

Want to customize or contribute? Build from source:
- Multiple backends can be compiled in, best one selected automatically
- Use `--gpu-backend <backend>` to force specific backend

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
- **Rust**: [`cargo install shimmy --features moe`](https://crates.io/crates/shimmy) *(recommended)*
- **Rust (basic)**: [`cargo install shimmy`](https://crates.io/crates/shimmy)
- **VS Code**: [Shimmy Extension](https://marketplace.visualstudio.com/items?itemName=targetedwebresults.shimmy-vscode)
- **Windows MSVC**: Uses `shimmy-llama-cpp-2` packages for better compatibility
- **npm**: `npm install -g shimmy-js` *(planned)*
- **Python**: `pip install shimmy` *(planned)*

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
- MLX native acceleration for Apple Silicon
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
shimmy serve                    # Start server (auto port allocation)
shimmy serve --bind 127.0.0.1:8080  # Manual port binding
shimmy serve --cpu-moe --n-cpu-moe 8  # Enable MOE CPU offloading
shimmy list                     # Show available models (LLM-filtered)
shimmy discover                 # Refresh model discovery
shimmy generate --name X --prompt "Hi"  # Test generation
shimmy probe model-name         # Verify model loads
shimmy gpu-info                 # Show GPU backend status
```

## Technical Architecture

- **Rust + Tokio**: Memory-safe, async performance
- **llama.cpp backend**: Industry-standard GGUF inference
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
- **📖 Documentation**: [docs/](docs/) • [Engineering Methodology](docs/METHODOLOGY.md) • [OpenAI Compatibility Matrix](docs/OPENAI_COMPAT.md) • [Benchmarks (Reproducible)](docs/BENCHMARKS.md)
- **💝 Sponsorship**: [GitHub Sponsors](https://github.com/sponsors/Michael-A-Kuykendall)

### Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Michael-A-Kuykendall/shimmy&type=Timeline)](https://www.star-history.com/#Michael-A-Kuykendall/shimmy&Timeline)

### 🚀 Momentum Snapshot

📦 **Sub-5MB single binary** (142x smaller than Ollama)
🌟 **![GitHub stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=flat&color=yellow) stars and climbing fast**
⏱ **<1s startup**
🦀 **100% Rust, no Python**

### 📰 As Featured On

🔥 [**Hacker News**](https://news.ycombinator.com/item?id=45130322) • [**Front Page Again**](https://news.ycombinator.com/item?id=45199898) • [**IPE Newsletter**](https://ipenewsletter.substack.com/p/the-strange-new-side-hustles-of-openai)

**Companies**: Need invoicing? Email [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)

## ⚡ Performance Comparison

| Tool | Binary Size | Startup Time | Memory Usage | OpenAI API |
|------|-------------|--------------|--------------|------------|
| **Shimmy** | **4.8MB** | **<100ms** | **50MB** | **100%** |
| Ollama | 680MB | 5-10s | 200MB+ | Partial |
| llama.cpp | 89MB | 1-2s | 100MB | Via llama-server |

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
