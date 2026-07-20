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

  **Languages:** [简体中文](docs/zh-CN/README.md) · [繁體中文](docs/zh-TW/README.md)
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

## Table of Contents

- [What Is Shimmy?](#drop-in-openai-api-replacement-for-local-llms)
- [🔥 Airframe Engine (v2.0)](#-airframe-engine)
- [⚡ TurboShimmy INT4 KV (v2.1)](#-turboshimmy-int4-kv)
- [🎯 Supported Models](#-supported-models)
- [📦 Migrating from v1.x](#-migrating-from-v1x)
- [⚡ Quick Start (30 seconds)](#quick-start-30-seconds)
- [🚀 OpenAI SDK Compatibility](#-compatible-with-openai-sdks-and-tools)
- [🔧 Extended Context](#-extended-context)
- [📥 Download & Install](#-download--install)
- [🔗 Integration Examples](#integration-examples)
- [📖 API Reference](#api-reference)
- [❓ FAQ](#-faq)
- [🏛️ Technical Architecture](#technical-architecture)
- [📚 Documentation Hub](#-documentation-hub)
- [🌍 Community & Support](#community--support)
- [⚡ Performance](#-performance-comparison)
- [License](#license--philosophy)

---

## Drop-in OpenAI API Replacement for Local LLMs

Shimmy is a **single-binary** that provides **100% OpenAI-compatible endpoints** for GGUF models. Point your existing AI tools to Shimmy and they just work — locally, privately, and free.

**🎉 NEW in v2.0.0**: Shimmy now runs on [Airframe](#-airframe-engine), a pure-Rust WGSL GPU engine. No C++ toolchain, no backend flags, no compilation required.

**⚡ NEW in v2.1.0**: [TurboShimmy INT4 KV](#-turboshimmy-int4-kv) — ~7× less KV cache VRAM with one flag. Run Llama-3.2-3B on 4 GB GPUs.

**⚡ NEW in v2.3.0**: Adapter selection fix (prefers discrete GPU over integrated), grammar control hooks, 357 passing tests.

## 🔥 Airframe Engine (v0.2.9)

Starting in v2.0.0, Shimmy's default inference engine is **Airframe** — a pure-Rust WebGPU (WGSL) transformer runtime built from scratch. **v0.2.9** brings the production batch_count fix, GPU adapter selection improvement, grammar hooks, and the PPT invariant cage.

**See [airframe CHANGELOG](https://github.com/Michael-A-Kuykendall/airframe/blob/master/CHANGELOG.md) for full release notes.**

**Why this matters:**
- No C++ toolchain required — Rust only, top to bottom
- F32 precision throughout for deterministic, high-quality output
- WGSL compute shaders work on any GPU via WebGPU (NVIDIA, AMD, Intel, integrated)
- Model spec auto-derived from GGUF metadata — no hardcoded per-model constants
- YaRN RoPE scaling for extended context via `SHIMMY_MAX_CTX` — engine allocates KV cache and sets RoPE scale automatically (see [Extended Context](#-extended-context) below)

**Quick start with Airframe (v2.0.0+):**
```bash
# Default: 2048-token context
SHIMMY_BASE_GGUF=/path/to/TinyLlama-1.1B-Chat-v1.0.Q4_0.gguf ./shimmy serve

# Extended context (4096 tokens — YaRN RoPE enabled automatically, KV cache resized)
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=4096 ./shimmy serve
```

## ⚡ TurboShimmy INT4 KV

**TurboShimmy** is Shimmy's on-GPU INT4 KV-cache compression system, shipping in v2.1.0. It squeezes the KV cache from 32-bit floats down to per-head-vector 4-bit integers — entirely in WGSL compute shaders with no CPU roundtrips — delivering ~7× less KV VRAM with no measurable quality loss at normal context lengths.

**One flag. ~7× less KV VRAM. Same output quality.**

```bash
# Enable TurboShimmy on any GGUF model
./shimmy serve --kv-quant int4

# Or via environment variable (docker-compose, systemd, etc.)
SHIMMY_KV_QUANT=int4 ./shimmy serve

# Windows GPU + long prompts: reduce per-dispatch work to prevent TDR resets
./shimmy serve --kv-quant int4 --prefill-chunk 8
```

**Why it matters** — TurboShimmy changes what fits on your GPU:

| GPU VRAM | Without TurboShimmy | With TurboShimmy (`--kv-quant int4`) |
|---|---|---|
| 3 GB | Llama-3.2-1B only | **Llama-3.2-3B fits ✅** |
| 4 GB | Llama-3.2-3B, ctx=2048 (tight) | **Llama-3.2-3B at ctx=8192 ✅** |
| 6 GB | 3B models, short context | **7B models with reasonable context ✅** |

**VRAM comparison (Llama-3.2-3B, ctx=2048):**

| Mode | KV cache | Total VRAM | Min GPU needed |
|---|---|---|---|
| Default (f32) | ~512 MB | ~2.4 GB | 3 GB (tight) |
| TurboShimmy (int4) | **~72 MB** | **~2.0 GB** | **2.5 GB ✅** |

**VRAM comparison (TinyLlama 1.1B, ctx=2048):**

| Mode | KV cache | Total VRAM |
|---|---|---|
| Default (f32) | 88 MB | ~700 MB |
| TurboShimmy (int4) | **~13 MB** | **~650 MB** |

**When to use TurboShimmy:**

| Situation | Recommendation |
|---|---|
| 3B model on a 4 GB GPU | `--kv-quant int4` — enables models that wouldn't fit otherwise |
| 7B model at ctx=4096+ | `--kv-quant int4` — cuts KV from ~512 MB → ~72 MB |
| Short chat sessions (ctx ≤ 2048) | `--kv-quant int4` — safe, no quality tradeoff |
| Long-form generation (ctx > 8192) | Default `f32` — keep maximum quality |
| Windows GPU + TDR crashes on long prompts | `--kv-quant int4 --prefill-chunk 8` |

**How it works:** Each K/V head vector is independently quantized to 4-bit integers with a per-vector F32 scale factor, encoded as packed nibbles by WGSL compute shaders. Dequantization happens on-the-fly when computing attention scores — also on GPU. The Airframe engine's helical context-shift operates directly on the packed INT4 representation. No CPU roundtrips at any step. Full architecture details in the [Airframe engine](https://github.com/Michael-A-Kuykendall/airframe).

> **Quality validation:** Needle-in-a-haystack benchmarks on Llama-3.2-3B show zero retrieval degradation vs F32 at ctx≤2048 across all tested depths (15%, 50%, 85%). Full benchmark data and setup guide: [TurboShimmy on the wiki](https://github.com/Michael-A-Kuykendall/shimmy/wiki/TurboShimmy).

> **Windows stability:** Airframe v0.2.1 ships a `device.on_uncaptured_error` handler so GPU validation errors surface as clean HTTP 500 responses instead of crashes. Use `--prefill-chunk 8` to prevent Windows TDR resets during long prefills on older GPUs (GTX 10xx/16xx series). **v0.2.7** adds TDR transport with GPU timestamp pools for accurate dispatch timing, fixing TDR watchdog crashes during long prefill sequences.

## 🎯 Supported Models

Airframe v2.0 ships with GPU-verified support across **7 model architectures** and **5 quantization types**, covering the models most commonly run on consumer hardware. Context window is read directly from each model's GGUF metadata — no hardcoded limits.

### ✅ Locally Validated (GPU Math Verified)

| Model | Architecture | Quant | Size | Context | Min VRAM |
|---|---|---|---|---|---|
| [TinyLlama-1.1B-Chat-v1.0](https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF) | Llama | Q4_0 | 638 MB | 2048 | ~800 MB |
| [Llama-3.2-1B-Instruct](https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF) | Llama | Q4_K_M | 770 MB | 131072* | ~1 GB |
| [Llama-3.2-3B-Instruct](https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF) | Llama | Q4_K_M | 1.9 GB | 131072* | ~2.5 GB |
| [phi-2](https://huggingface.co/TheBloke/phi-2-GGUF) | Phi-2 | Q4_K_M | 1.7 GB | 2048 | ~2.2 GB |
| [gemma-2-2b-it](https://huggingface.co/bartowski/gemma-2-2b-it-GGUF) | Gemma-2 | Q4_K_M | 1.6 GB | 8192 | ~2 GB |
| [starcoder2-3b](https://huggingface.co/second-state/StarCoder2-3B-GGUF) | StarCoder2 | Q4_K_M | 1.8 GB | 16384 | ~2.3 GB |
| [gpt2](https://huggingface.co/ggerganov/ggml/blob/main/gpt-2-117M-q4_0.bin) | GPT-2 | Q4_K_M | 107 MB | 1024 | ~200 MB |

> \* Llama-3.2's native context is 131072 tokens. Airframe reads this from GGUF and allocates KV cache accordingly. Use `SHIMMY_MAX_CTX=8192` for a practical 8K window on consumer hardware (~256 MB KV cache for the 1B model).

**GPU Math Verified** means the Airframe GPU dequantization shader produces results matching the CPU reference implementation, independently confirmed for every tensor type in each model. This is done via `quant_verify`, which tests 512 elements per quantization type per model.

### ⏳ Roadmap — Larger Models (Require ≥16 GB VRAM)

| Model | Architecture | Quant | Size | Status |
|---|---|---|---|---|
| deepseek-coder-6.7b-instruct | Llama | Q4_K_M | 3.9 GB | Pending remote GPU validation |
| deepseek-llm-7b-chat | Llama | Q4_K_M | 4.0 GB | Pending remote GPU validation |
| qwen2-7b-instruct | Qwen2 | Q4_K_M | 4.5 GB | Pending remote GPU validation |
| Phi-3.5-mini-instruct | Phi-3 | Q4_K_M | 2.3 GB | Requires fused QKV support (planned) |

### ✅ Supported Quantization Types

| Type | GGML ID | Notes |
|---|---|---|
| `F32` | 0 | Raw floats — maximum precision |
| `F16` | 1 | Half-precision floats |
| `Q4_0` | 2 | 4-bit, 32-element blocks |
| `Q8_0` | 8 | 8-bit, 32-element blocks |
| `Q4_K` | 12 | 4-bit K-quant superblocks (256 elements) — used in Q4_K_M GGUFs |
| `Q5_K` | 13 | 5-bit K-quant superblocks — used alongside Q4_K in mixed-precision models |
| `Q6_K` | 14 | 6-bit K-quant superblocks — typically used for output/embedding layers |

All types are implemented in both the GPU inference shader and a CPU reference implementation. GPU vs CPU agreement is validated for every type.

**Auto-discovery is enabled.** If Shimmy finds GGUF models in your HuggingFace cache, Ollama directory, LM Studio cache (`~/.cache/lm-studio/models`), or local `./models/` folder, it will register and serve them automatically. See [docs/MODEL_EXPANSION.md](docs/MODEL_EXPANSION.md) for the full compatibility matrix.

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

- **Automatically finds models** from Hugging Face cache, Ollama, LM Studio (`~/.cache/lm-studio/models`), and local dirs
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
# Install from crates.io
cargo install shimmy

# Build from source (huggingface engine, no GPU)
git clone https://github.com/Michael-A-Kuykendall/shimmy
cd shimmy
cargo build --release
```

> **Note**: The Airframe GPU engine is a public crate on [crates.io](https://crates.io/crates/airframe) and builds from source automatically. `cargo install shimmy` installs the huggingface engine variant from crates.io; for the full GPU build use the [pre-built release binaries](#-download-pre-built-binaries-recommended) or clone and build with `cargo build --release`.

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

`SHIMMY_MAX_CTX` overrides the context window at the engine level. When set above the model's native window, Airframe automatically engages YaRN RoPE scaling and resizes the KV cache accordingly.

```bash
# 4096-token context with YaRN (2x native window for TinyLlama)
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=4096 shimmy serve

# 8192 tokens (4x native, higher RoPE compression)
SHIMMY_BASE_GGUF=/path/to/model.gguf SHIMMY_MAX_CTX=8192 shimmy serve
```

> **Note:** Extended context beyond 4096 is functional but not yet as deeply validated as the native 2048-token window. Accepted range is 512–131072. Values outside that range are silently ignored and 2048 is used.

#### **💾 VRAM Sizing Reference**

Airframe allocates VRAM at load time: **weights** + **KV cache**. The KV cache is F32 and scales linearly with context length (`n_layers × n_kv_heads × head_dim × ctx × 2 × 4 bytes`).

**TinyLlama 1.1B Q4_0 — the v2.0 validated path:**

| Context (`SHIMMY_MAX_CTX`) | KV cache | Weights | Total | Min VRAM |
|---|---|---|---|---|
| 2048 (default) | ~88 MB | ~638 MB | ~726 MB | **~800 MB** |
| 4096 | ~176 MB | ~638 MB | ~814 MB | **~900 MB** |
| 8192 | ~352 MB | ~638 MB | ~990 MB | **~1.1 GB** |
| 16384 | ~704 MB | ~638 MB | ~1.3 GB | **~1.5 GB** |

> Integrated graphics (Intel Iris, Apple M-series unified memory, AMD Vega) running at 2048 context is ~800 MB — comfortably inside the 2 GB allocation most integrated GPUs share with system RAM.

**Scaling up to larger models** (architecture and quant support required — see [docs/MODEL_EXPANSION.md](docs/MODEL_EXPANSION.md)):

| Model | Quant | Weights | KV @ 2048 ctx | Min VRAM |
|---|---|---|---|---|
| Llama 3.2 1B | Q4_0 | ~636 MB | ~128 MB | ~900 MB |
| Llama 3.2 3B | Q4_0 | ~1.9 GB | ~448 MB | ~2.5 GB |
| Mistral 7B | Q4_K_M | ~4.1 GB | ~512 MB | ~5 GB |
| Llama 3 8B | Q4_K_M | ~4.7 GB | ~512 MB | ~5.5 GB |

The KV cache formula for any model: `n_layers × n_kv_heads × head_dim × ctx × 2 × 4 bytes`. Multiply the 2048 baseline by your `SHIMMY_MAX_CTX` multiplier to get the extended context allocation.

### Get Models

Shimmy auto-discovers models from:
- **Hugging Face cache**: `~/.cache/huggingface/hub/`
- **Ollama models**: `~/.ollama/models/`
- **Local directory**: `./models/`
- **Environment**: `SHIMMY_BASE_GGUF=path/to/model.gguf`

```bash
# Primary validated model for Airframe v2.0
huggingface-cli download TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF \
  --include "tinyllama-1.1b-chat-v1.0.Q4_0.gguf" --local-dir ./models/

# Alternative 1B — also fits in the same hardware envelope
huggingface-cli download bartowski/Llama-3.2-1B-Instruct-GGUF \
  --include "*Q4_K_M*" --local-dir ./models/
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
- `POST /v1/chat/completions` - OpenAI-compatible chat (streaming supported)
- `POST /v1/completions` - OpenAI-compatible text completions
- `GET /v1/models` - List available models
- `POST /api/generate` - Shimmy native API
- `GET /ws/generate` - WebSocket streaming

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `SHIMMY_BASE_GGUF` | *(auto-discover)* | Path to GGUF model file loaded as the default model |
| `SHIMMY_PORT` | `8080` | Port to listen on (Airframe server binary) |
| `SHIMMY_BIND_ADDRESS` | `0.0.0.0:8080` | Full bind address (overrides port) |
| `SHIMMY_MAX_CTX` | *(from GGUF)* | Override context window; activates YaRN RoPE scaling when above model native |
| `SHIMMY_MODEL_PATHS` | *(see Zero Config)* | Colon-separated extra model search paths |
| `SHIMMY_ENGINE_BACKEND` | `airframe` | `airframe` (default) or `llama` (legacy path) |
| `SHIMMY_ROPE_SCALE` | *(auto)* | Override computed YaRN scale factor |
| `SHIMMY_KV_QUANT` | `f32` | KV cache quantization: `f32` (default) or `int4` ([TurboShimmy](#-turboshimmy-int4-kv)) |
| `SHIMMY_PREFILL_CHUNK` | `64` | Tokens per prefill dispatch. Set to `8` on Windows if you see GPU TDR resets on long prompts |
| `RUST_BACKTRACE` | *(off)* | Set to `1` to print crash backtraces |

### CLI Commands
```bash
shimmy serve                              # Start server (auto port allocation)
shimmy serve --bind 127.0.0.1:8080        # Manual port binding
shimmy serve --gpu-backend auto           # WebGPU adapter auto-select (default)
shimmy serve --gpu-backend cpu            # Force CPU (disable GPU)
shimmy serve --kv-quant int4              # Enable TurboShimmy INT4 KV cache compression
shimmy serve --kv-quant int4 --prefill-chunk 8  # INT4 + Windows TDR prevention
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

---

## ❓ FAQ

**Does Shimmy work on my GPU?**
Shimmy uses WebGPU (via the Airframe engine) which runs on Vulkan, D3D12, and Metal — covering NVIDIA, AMD, Intel, and Apple Silicon. No CUDA required. See [GPU requirements in TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) if you hit adapter errors.

**What's the difference between Shimmy and llama.cpp / Ollama?**
Shimmy is written in pure Rust with no C++ toolchain dependency. The Airframe engine runs WGSL compute shaders compiled at startup — no pre-built binaries, no driver version pinning. The result is faster startup, lower memory overhead, and deterministic output. See the [GPU Pipeline doc](docs/GPU_PIPELINE.md) for internals.

**Why do I need `SHIMMY_BASE_GGUF` or `LIBSHIMMY_MODEL_PATH`?**
If you don't set these, Shimmy auto-discovers models in standard directories (`~/.cache/huggingface`, `~/.ollama`, `~/lm-studio/models`, `~/.cache/lm-studio/models`, `~/Library/Application Support/LMStudio`). Set `SHIMMY_BASE_GGUF` to override and point directly at a specific GGUF file.

**Can I run multiple models at once?**
Not currently — Shimmy loads one model per server instance. To serve multiple models, run multiple server instances on different ports. Hot-swapping models (reload without restart) is on the roadmap.

**Why does generation stop before `max_tokens`?**
The model reached a natural end-of-sequence token. For chat models this is expected behavior — the model signals it's done. If you want to force longer output, increase `max_tokens` and set `temperature > 0`. If generation stops on the wrong token, the model may be using the wrong chat template — see [CHAT_TEMPLATES.md](docs/CHAT_TEMPLATES.md).

**Is there streaming support?**
Set `"stream": true` in your request. Shimmy returns Server-Sent Events in the standard OpenAI streaming format.

**Q4_K_M vs Q4_0 — which should I use?**
`Q4_K_M` (K-quant) is consistently better quality than `Q4_0` for the same file size. Use `Q4_0` only when you need maximum compatibility or the model isn't available in K-quant. See [QUANTIZATION.md](docs/QUANTIZATION.md) for the full analysis.

**Can I extend the context window beyond what the model was trained on?**
Yes — set `SHIMMY_MAX_CTX` to any value. Airframe applies YaRN scaling automatically when the requested context exceeds the model's native context. Quality degrades gradually beyond 2× the native context. See [EXTENDED_CONTEXT.md](docs/EXTENDED_CONTEXT.md).

---

## 📚 Documentation Hub

Full documentation lives in [docs/](docs/). Use this table to find what you need:

### Getting Started
| Document | Description |
|---|---|
| [docs/quickstart.md](docs/quickstart.md) | 5-minute getting started guide |
| [docs/MIGRATION_v2.md](docs/MIGRATION_v2.md) | Migrating from Shimmy v1.x |
| [docs/CONFIGURATION.md](docs/CONFIGURATION.md) | All environment variables and config options |
| [docs/WINDOWS_GPU_BUILD_GUIDE.md](docs/WINDOWS_GPU_BUILD_GUIDE.md) | Windows-specific build instructions |

### API & Integration
| Document | Description |
|---|---|
| [docs/API.md](docs/API.md) | Complete API endpoint reference |
| [docs/OPENAI_COMPAT.md](docs/OPENAI_COMPAT.md) | OpenAI compatibility matrix — what's supported |
| [docs/INTEGRATION.md](docs/INTEGRATION.md) | Integrating with LangChain, OpenAI SDKs, etc. |
| [docs/EXAMPLES.md](docs/EXAMPLES.md) | Runnable code examples |
| [docs/CROSS_COMPILATION.md](docs/CROSS_COMPILATION.md) | Building for other targets (ARM, Linux from Windows) |

### Engine Deep Dives
| Document | Description |
|---|---|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System-level architecture and component map |
| [docs/GPU_PIPELINE.md](docs/GPU_PIPELINE.md) | Bindless GPU architecture, WGSL shaders, dispatch patterns |
| [docs/QUANTIZATION.md](docs/QUANTIZATION.md) | Q4_0, Q8_0, K-quant formats — bit-level internals |
| [docs/EXTENDED_CONTEXT.md](docs/EXTENDED_CONTEXT.md) | YaRN RoPE scaling, VRAM math, context extension |
| [docs/CHAT_TEMPLATES.md](docs/CHAT_TEMPLATES.md) | Chat template auto-detection and format reference |
| [docs/MODEL_EXPANSION.md](docs/MODEL_EXPANSION.md) | Model onboarding protocol and acceptance gates |

### Troubleshooting & Reference
| Document | Description |
|---|---|
| [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | Diagnostic guide for GPU errors, model failures, port conflicts |
| [docs/PERFORMANCE.md](docs/PERFORMANCE.md) | Performance tuning and token/sec benchmarks |
| [docs/FEATURES.md](docs/FEATURES.md) | Complete feature list |

### Development & Methodology
| Document | Description |
|---|---|
| [docs/METHODOLOGY.md](docs/METHODOLOGY.md) | Engineering methodology and quality standards |
| [docs/REGRESSION_TESTING.md](docs/REGRESSION_TESTING.md) | Regression testing approach |
| [docs/ppt-invariant-testing.md](docs/ppt-invariant-testing.md) | Property-based and invariant testing details |
| [docs/METRICS.md](docs/METRICS.md) | Observability and metrics reference |

---

## Community & Support

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)
- **📖 Documentation**: [Full Documentation Hub](#-documentation-hub) • [Migration Guide v1→v2](docs/MIGRATION_v2.md) • [Engineering Methodology](docs/METHODOLOGY.md) • [OpenAI Compatibility Matrix](docs/OPENAI_COMPAT.md) • [Architecture](docs/ARCHITECTURE.md) • [GPU Pipeline](docs/GPU_PIPELINE.md) • [Troubleshooting](docs/TROUBLESHOOTING.md)
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
