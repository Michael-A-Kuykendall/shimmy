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

## What Is Shimmy?

Shimmy is a **single-binary** that provides **100% OpenAI-compatible endpoints** for GGUF models. Point your existing AI tools to Shimmy and they just work — locally, privately, and free.

Under the hood it runs on **Airframe**, a pure-Rust WebGPU (WGSL) transformer engine built from scratch. No C++ toolchain, no backend flags, no compilation required. Version history lives in the [CHANGELOG](CHANGELOG.md); see the [Airframe CHANGELOG](https://github.com/Michael-A-Kuykendall/airframe/blob/master/CHANGELOG.md) for engine release notes.

**Why this matters:**
- No C++ toolchain required — Rust only, top to bottom
- F32 precision throughout for deterministic, high-quality output
- WGSL compute shaders work on any GPU via WebGPU (NVIDIA, AMD, Intel, integrated)
- Model spec auto-derived from GGUF metadata — no hardcoded per-model constants
- YaRN RoPE scaling for extended context via `SHIMMY_MAX_CTX` (see [Extended Context](docs/EXTENDED_CONTEXT.md))

---

## 🎯 Supported Models

**11 model families · 25 certified model/quant combinations** — every model below passes Shimmy's 5-gate GPU math verification pipeline (dequant, structural peel, numerical, decode≡prefill, logits) against the certification ledger. GGUF files load as-is; no recompilation, no hardcoded per-model constants.

| Family | Model | Quants |
|---|---|---|
| **Llama** | [Llama-3.2-1B-Instruct](https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF) | Q4_K_M · Q6_K |
|  | [Llama-3.2-3B-Instruct](https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF) | Q4_K_M |
|  | [TinyLlama-1.1B-Chat](https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF) | Q4_0 · Q5_K_M · Q6_K |
| **Qwen3** | [Qwen3-0.6B](https://huggingface.co/Qwen/Qwen3-0.6B-GGUF) | Q4_K_M |
|  | [Qwen3-1.7B](https://huggingface.co/Qwen/Qwen3-1.7B-GGUF) | Q4_K_M |
|  | [Qwen3-4B](https://huggingface.co/Qwen/Qwen3-4B-GGUF) | Q4_K_M |
|  | [Qwen3-4B-Thinking](https://huggingface.co/Qwen/Qwen3-4B-Thinking-GGUF) | Q4_K_M |
|  | [Qwen3-8B](https://huggingface.co/Qwen/Qwen3-8B-GGUF) | Q4_K_M |
| **Qwen2** | [Qwen2-0.5B-Instruct](https://huggingface.co/Qwen/Qwen2-0.5B-Instruct-GGUF) | Q4_K_M |
|  | [Qwen2-1.5B-Instruct](https://huggingface.co/Qwen/Qwen2-1.5B-Instruct-GGUF) | Q4_K_M |
|  | [Qwen2-7B-Instruct](https://huggingface.co/Qwen/Qwen2-7B-Instruct-GGUF) | Q4_K_M |
| **Qwen3.5** | [Qwen3.5-9B](https://huggingface.co/Qwen/Qwen3.5-9B-GGUF) | Q4_K_M |
| **Phi-3** | [Phi-3.5-mini-Instruct](https://huggingface.co/microsoft/Phi-3.5-mini-instruct-gguf) | Q4_K_M |
|  | [Phi-3-mini-4k-Instruct](https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf) | Q4_0 |
| **Phi-2** | [Phi-2](https://huggingface.co/TheBloke/phi-2-GGUF) | Q4_K_M |
| **Gemma-2** | [Gemma-2-2B-it](https://huggingface.co/bartowski/gemma-2-2b-it-GGUF) | Q4_K_M |
|  | [Gemma-2-9B-it](https://huggingface.co/bartowski/gemma-2-9b-it-GGUF) | Q4_K_M |
| **Gemma-4** | [Gemma-4-12B-coder](https://huggingface.co/google/gemma-4-12B-coder-GGUF) | Q4_K_M |
|  | [Gemma-4-E4B](https://huggingface.co/google/gemma-4-E4B-it-GGUF) | Q4_K_M |
| **DeepSeek-R1** | [DeepSeek-R1-0528-Qwen3-8B](https://huggingface.co/deepseek-ai/DeepSeek-R1-Distill-Qwen-8B-GGUF) | Q4_K_M |
| **Ministral** | [Ministral-3-14B-Reasoning](https://huggingface.co/bartowski/Ministral-3-14B-Reasoning-GGUF) | Q4_K_M |
| **StarCoder2** | [StarCoder2-3B](https://huggingface.co/second-state/StarCoder2-3B-GGUF) | Q4_K_M |




## Features at a Glance

- **⚡ [TurboShimmy INT4 KV Cache](docs/turboshimmy.md)** — ~7× less KV VRAM with one flag (`--kv-quant int4`). Run Llama-3.2-3B on 4 GB GPUs.
- **🚀 [OpenAI SDK Compatibility](docs/INTEGRATION.md)** — drop-in replacement; VSCode Copilot, Cursor, Continue.dev, any OpenAI SDK.
- **🔧 [Extended Context](docs/EXTENDED_CONTEXT.md)** — YaRN RoPE scaling via `SHIMMY_MAX_CTX`.
- **📦 [Migrating from v1.x](docs/MIGRATION_v2.md)** — the llama.cpp backend was removed in v2.0; see the migration guide.
- **🧠 MOE support** — Mixture-of-Experts CPU offloading is on the [Airframe roadmap](docs/AIRFRAME_MOE_ROADMAP.md).
- **🏆 Certification** — Every model passes a 5-gate mathematical verification pipeline. See [docs/CERTIFICATION.md](docs/CERTIFICATION.md) for how it works.

---

## Quick Start (30 seconds)

```bash
# 1) Download pre-built binary (Windows example)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# 2) Point it at a GGUF model
set SHIMMY_BASE_GGUF=C:\path\to\model.gguf && ./shimmy.exe serve &

# 3) See registered models
./shimmy list

# 4) Smoke test the OpenAI API
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"tinyllama-1.1b","messages":[{"role":"user","content":"Say hi in 5 words."}],"max_tokens":32}' \
  | jq -r '.choices[0].message.content'
```

Full install, model acquisition, GPU, and VRAM sizing: **[docs/quickstart.md](docs/quickstart.md)**

---

## Documentation Hub

Full documentation lives in [docs/](docs/). Use this table to find what you need:

### Getting Started
| Document | Description |
|---|---|
| [quickstart.md](docs/quickstart.md) | Install, models, GPU, VRAM, extended context |
| [MIGRATION_v2.md](docs/MIGRATION_v2.md) | Migrating from Shimmy v1.x |
| [CONFIGURATION.md](docs/CONFIGURATION.md) | All environment variables and config options |
| [WINDOWS_GPU_BUILD_GUIDE.md](docs/WINDOWS_GPU_BUILD_GUIDE.md) | Windows-specific build instructions |

### Models & Performance
| Document | Description |
|---|---|
| [SUPPORTED_MODELS.md](docs/SUPPORTED_MODELS.md) | Certified models and quantization support |
| [turboshimmy.md](docs/turboshimmy.md) | INT4 KV cache compression |
| [EXTENDED_CONTEXT.md](docs/EXTENDED_CONTEXT.md) | YaRN RoPE scaling, VRAM math |
| [MODEL_EXPANSION.md](docs/MODEL_EXPANSION.md) | Model onboarding protocol and acceptance gates |
| [PERFORMANCE.md](docs/PERFORMANCE.md) | Performance tuning and token/sec benchmarks |

### API & Integration
| Document | Description |
|---|---|
| [API.md](docs/API.md) | Complete endpoint, CLI, and env-var reference |
| [OPENAI_COMPAT.md](docs/OPENAI_COMPAT.md) | OpenAI compatibility matrix — what's supported |
| [INTEGRATION.md](docs/INTEGRATION.md) | LangChain, OpenAI SDKs, VSCode, etc. |
| [EXAMPLES.md](docs/EXAMPLES.md) | Runnable code examples |
| [CROSS_COMPILATION.md](docs/CROSS_COMPILATION.md) | Building for other targets (ARM, Linux from Windows) |

### Engine Deep Dives
| Document | Description |
|---|---|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | System-level architecture and component map |
| [GPU_PIPELINE.md](docs/GPU_PIPELINE.md) | Bindless GPU architecture, WGSL shaders, dispatch patterns |
| [QUANTIZATION.md](docs/QUANTIZATION.md) | Q4_0, Q8_0, K-quant formats — bit-level internals |
| [CHAT_TEMPLATES.md](docs/CHAT_TEMPLATES.md) | Chat template auto-detection and format reference |

### FAQ & Troubleshooting
| Document | Description |
|---|---|
| [FAQ.md](docs/FAQ.md) | Frequently asked questions |
| [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | GPU errors, model failures, port conflicts |
| [FEATURES.md](docs/FEATURES.md) | Complete feature list |

### Certification & Methodology
| Document | Description |
|---|---|
| [CERTIFICATION.md](docs/CERTIFICATION.md) | How we mathematically prove every model is correct |
| [METHODOLOGY.md](docs/METHODOLOGY.md) | Engineering methodology and quality standards |
| [REGRESSION_TESTING.md](docs/REGRESSION_TESTING.md) | Regression testing approach |
| [ppt-invariant-testing.md](docs/ppt-invariant-testing.md) | Property-based and invariant testing details |
| [METRICS.md](docs/METRICS.md) | Observability and metrics reference |

---

## Development Testing

Shimmy maintains high code quality through comprehensive testing:

```bash
# Full test suite (default features = GPU engine)
cargo test --features airframe,huggingface

# Quick CPU-only tests (no GPU required)
cargo test --lib --no-default-features --features huggingface -- --test-threads=1
```

See [docs/ppt-invariant-testing.md](docs/ppt-invariant-testing.md) for technical details.

---

## Community & Support

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)
- **💝 Sponsorship**: [GitHub Sponsors](https://github.com/sponsors/Michael-A-Kuykendall)

### Star History

[![Star History Chart](https://star-history.dera.page/svg?repos=Michael-A-Kuykendall/shimmy&type=Timeline)](https://star-history.dera.page/#Michael-A-Kuykendall/shimmy&Timeline)

### 🚀 Momentum Snapshot

🌟 **![GitHub stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=flat&color=yellow) stars and climbing fast**
⏱ **<1s startup**
🦀 **100% Rust, no Python**

### 📰 As Featured On

🔥 [**Hacker News**](https://news.ycombinator.com/item?id=45130322) • [**Front Page Again**](https://news.ycombinator.com/item?id=45199898) • [**IPE Newsletter**](https://ipenewsletter.substack.com/p/the-strange-new-side-hustles-of-openai)

**Companies**: Need invoicing? Email [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)

---

## Performance Comparison

| Tool | Startup Time | Memory Usage | OpenAI API |
|------|--------------|--------------|------------|
| **Shimmy** | **<100ms** | **50MB** | **100%** |
| Ollama | 5-10s | 200MB+ | Partial |

---

## License & Philosophy

MIT License - forever and always.

**Philosophy**: Infrastructure should be invisible. Shimmy is infrastructure.

**Testing Philosophy**: Reliability through comprehensive validation and property-based testing.

---

**Forever maintainer**: Michael A. Kuykendall
**Promise**: This will never become a paid product
**Mission**: Making local model inference simple and reliable
