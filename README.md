<div align="center">
  <img src="https://raw.githubusercontent.com/Michael-A-Kuykendall/shimmy/main/assets/shimmy-logo.png" alt="Shimmy Logo" width="300" height="auto" />

  # Shimmy — Local Inference, OpenAI-Compatible

  ### 🔒 The 5MB alternative to Ollama — 100% Rust, zero dependencies 🚀

  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
  [![Security](https://img.shields.io/badge/Security-Audited-green)](https://github.com/Michael-A-Kuykendall/shimmy/security)
  [![CI](https://github.com/Michael-A-Kuykendall/shimmy/actions/workflows/ci.yml/badge.svg)](https://github.com/Michael-A-Kuykendall/shimmy/actions/workflows/ci.yml)
  [![Crates.io](https://img.shields.io/crates/v/shimmy.svg)](https://crates.io/crates/shimmy)
  [![Downloads](https://img.shields.io/crates/d/shimmy.svg)](https://crates.io/crates/shimmy)
  [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
  [![GitHub Stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=social)](https://github.com/Michael-A-Kuykendall/shimmy/stargazers)
  [![💝 Sponsor this project](https://img.shields.io/badge/💝_Sponsor_this_project-ea4aaa?style=for-the-badge&logo=github&logoColor=white)](https://github.com/sponsors/Michael-A-Kuykendall)
  [![Trans rights](https://pride-badges.pony.workers.dev/static/v1?label=trans%20rights&stripeWidth=6&stripeColors=5BCEFA,F5A9B8,FFFFFF,F5A9B8,5BCEFA)](https://translifeline.org/)
  [![LGBTQ+ friendly](https://pride-badges.pony.workers.dev/static/v1?label=lgbtq%2B%20friendly&stripeWidth=6&stripeColors=E40303,FF8C00,FFED00,008026,24408E,732982)](https://www.thetrevorproject.org/)

  **Languages:** [简体中文](docs/zh-CN/README.md) · [繁體中文](docs/zh-TW/README.md)
</div>

Shimmy is independently maintained and free forever. [Sponsorship](https://github.com/sponsors/Michael-A-Kuykendall) funds certification, compatibility work, and releases.

**Shimmy will be free forever.** No asterisks. No "free for now." No pivot to paid.

---

## What Is Shimmy?

Shimmy is a **single-binary** OpenAI-compatible inference server for GGUF models. Point your existing AI tools at Shimmy and they just work — locally, privately, and free.

**Shimmy is the server. Airframe is the engine.** Under the hood, Shimmy runs on [**Airframe**](https://github.com/Michael-A-Kuykendall/airframe) (v0.4.0), a pure-Rust WebGPU (WGSL) transformer engine. No C++ toolchain, no Python runtime, no backend flags. 26 models certified across 12 families. Version history: [CHANGELOG](CHANGELOG.md) · [Airframe CHANGELOG](https://github.com/Michael-A-Kuykendall/airframe/blob/main/CHANGELOG.md).

**Why this matters:**
- No Python runtime or C++ toolchain — Rust only, top to bottom
- F32 accumulation precision with deterministic output (same model + seed + params → same output)
- WGSL compute shaders via WebGPU — NVIDIA, AMD, Intel, integrated GPUs, Apple Silicon
- Model spec auto-derived from GGUF metadata — no hardcoded per-model constants
- YaRN RoPE scaling for extended context via `SHIMMY_MAX_CTX` (see [Extended Context](docs/EXTENDED_CONTEXT.md))

---

## 🎯 Supported Models

**12 model families · 26 certified model/quant combinations** — every model below passes Shimmy's 3-box certification regimen (MATH + INFERENCE + DETERMINISM) against the certification ledger. Certification applies to the named model/quant combination; architecture recognition does not automatically mean certification. GGUF files load as-is; no recompilation, no hardcoded per-model constants.

| Family | Model | Quants |
|---|---|---|
| **Llama** | [Llama-3.2-1B-Instruct](https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF) | Q4_K_M · Q6_K |
|  | [Llama-3.2-3B-Instruct](https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF) | Q4_K_M |
|  | [Llama-3.1-8B-Instruct](https://huggingface.co/bartowski/Llama-3.1-8B-Instruct-GGUF) | Q4_K_M |
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
|  | [Gemma-2-9B-it](https://huggingface.co/bartowski/gemma-2-9b-it-GGUF) | Q4_K_M (supported; cert: see [v2-roadmap](docs/v2-roadmap.md)) |
| **Gemma-4** | [Gemma-4-12B-coder](https://huggingface.co/google/gemma-4-12B-coder-GGUF) | Q4_K_M |
|  | [Gemma-4-E4B](https://huggingface.co/google/gemma-4-E4B-it-GGUF) | Q4_K_M |
| **DeepSeek-R1** | [DeepSeek-R1-0528-Qwen3-8B](https://huggingface.co/deepseek-ai/DeepSeek-R1-Distill-Qwen-8B-GGUF) | Q4_K_M |
| **Ministral** | [Ministral-3-14B-Reasoning](https://huggingface.co/bartowski/Ministral-3-14B-Reasoning-GGUF) | Q4_K_M |
| **StarCoder2** | [StarCoder2-3B](https://huggingface.co/second-state/StarCoder2-3B-GGUF) | Q4_K_M |

**SafeTensors format** (`.safetensors`) is supported for model loading via `safetensors_native`. Full Airframe-native inference for SafeTensors remains roadmap work; see [docs/v2-roadmap.md](docs/v2-roadmap.md).



## Features

- **⚡ [TurboShimmy INT4 KV Cache](docs/turboshimmy.md)** — About 7× lower KV-cache memory in tested configurations. Run Llama-3.2-3B on 4 GB GPUs.
- **🚀 [OpenAI SDK Compatibility](docs/INTEGRATION.md)** — Chat completions, text completions, streaming, and model endpoints. Works with OpenAI SDKs and tools using that surface.
- **🔧 [Extended Context](docs/EXTENDED_CONTEXT.md)** — YaRN RoPE scaling via `SHIMMY_MAX_CTX`.
- **📦 [Migrating from v1.x](docs/MIGRATION_v2.md)** — llama.cpp, MLX, HuggingFace, and RustChain backends removed in v2.0+. Shimmy is now a pure Airframe product.
- **🏆 Certification** — Every model passes a 3-box certification regimen (MATH + INFERENCE + DETERMINISM). See [docs/CERTIFICATION.md](docs/CERTIFICATION.md).
- **🧠 MOE support** — Mixture-of-Experts CPU offloading is on the Airframe roadmap.

---

## Quick Start

```bash
cargo install shimmy
shimmy serve --model-path /absolute/path/to/model.gguf --bind 127.0.0.1:11435
```

Then in another terminal:

```bash
shimmy list --short
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"tinyllama-1.1b","messages":[{"role":"user","content":"Say hi in 5 words."}],"max_tokens":32}'
```

Full install, model acquisition, GPU, VRAM sizing, platform-specific builds: **[docs/quickstart.md](docs/quickstart.md)**

---

## Documentation

| Start here | What you need |
|---|---|
| [Quick Start](docs/quickstart.md) | Install, models, GPU, VRAM |
| [Supported Models](docs/SUPPORTED_MODELS.md) | Certified models and quantization |
| [API Compatibility](docs/OPENAI_COMPAT.md) | Endpoints, SDKs, integration |
| [Configuration](docs/CONFIGURATION.md) | Env vars and config options |
| [Troubleshooting](docs/TROUBLESHOOTING.md) | GPU errors, model failures |

<details>
<summary>Complete documentation index</summary>

| Section | Documents |
|---|---|
| **Models & Performance** | [TurboShimmy](docs/turboshimmy.md) — INT4 KV cache compression · [Extended Context](docs/EXTENDED_CONTEXT.md) — YaRN RoPE scaling, VRAM math · [Performance](docs/PERFORMANCE.md) — Tuning and token/sec · [Model Expansion](docs/MODEL_EXPANSION.md) — Onboarding protocol |
| **API & Integration** | [API Reference](docs/API.md) · [Integration Guides](docs/INTEGRATION.md) · [Examples](docs/EXAMPLES.md) · [Cross-Compilation](docs/CROSS_COMPILATION.md) |
| **Engine** | [Architecture](docs/ARCHITECTURE.md) · [GPU Pipeline](docs/GPU_PIPELINE.md) · [Quantization](docs/QUANTIZATION.md) · [Chat Templates](docs/CHAT_TEMPLATES.md) |
| **Certification** | [Certification](docs/CERTIFICATION.md) · [Methodology](docs/METHODOLOGY.md) · [Regression Testing](docs/REGRESSION_TESTING.md) · [PPT Testing](docs/ppt-invariant-testing.md) · [Metrics](docs/METRICS.md) |
| **FAQ** | [FAQ](docs/FAQ.md) · [Features](docs/FEATURES.md) · [Migration](docs/MIGRATION_v2.md) · [Windows GPU](docs/WINDOWS_GPU_BUILD_GUIDE.md) |

</details>

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
- **📖 Security**: [Security Policy](https://github.com/Michael-A-Kuykendall/shimmy/security)

### 🚀 Momentum Snapshot

🌟 **![GitHub stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=flat&color=yellow) stars and climbing fast**
⏱ **<1s startup**
🦀 **100% Rust, no Python**

### 📰 As Featured On

🔥 [**Hacker News**](https://news.ycombinator.com/item?id=45130322) · [**Front Page Again**](https://news.ycombinator.com/item?id=45199898) · [**IPE Newsletter**](https://ipenewsletter.substack.com/p/the-strange-new-side-hustles-of-openai)

**Companies**: Need invoicing? Email [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)

---

## Performance

| Tool | Startup | Memory | API |
|------|---------|--------|-----|
| **Shimmy** | **<1s** | **~50MB** | Chat, completions, streaming, models |
| Ollama | 5-10s | 200MB+ | Partial |

_Measured on RTX 3060, Shimmy v2.6.0, TinyLlama-1.1B. Your results vary by hardware._

---

## Sponsor Shimmy

Shimmy is independently maintained. Sponsorship funds certification, compatibility work, and releases.

- **$5/month**: Coffee tier ☕ — Sponsor badge + name in [SPONSORS.md](SPONSORS.md)
- **$25/month**: Supporter 🐛 — Priority support + name in [SPONSORS.md](SPONSORS.md)
- **$100/month**: Corporate backer 🏢 — Logo placement + release recognition
- **$500/month**: Infrastructure partner 🚀 — Office hours + roadmap consultation

**Current sponsors:** [ZephyrCloudIO](https://github.com/ZephyrCloudIO) · [gqf2008](https://github.com/gqf2008) · [alistairheath](https://github.com/alistairheath)

[**🎯 Become a Sponsor**](https://github.com/sponsors/Michael-A-Kuykendall) · [Invoicing](mailto:michaelallenkuykendall@gmail.com)

---

## License & Philosophy

MIT License — see [LICENSE](LICENSE). **Shimmy will be free forever.**

**Promise**: This will never become a paid product.

Shimmy is infrastructure: it should be invisible. Reliability through comprehensive validation and property-based testing.

---

**Maintainer**: Michael A. Kuykendall · **Mission**: Making local model inference simple and reliable

---

## Support

This project is a safe space. Trans rights are human rights.

If you or someone you love needs support:

- [The Trevor Project](https://www.thetrevorproject.org/) — 24/7 for LGBTQ+ young people. Call 1-866-488-7386 or text START to 678-678
- [Trans Lifeline](https://translifeline.org/) — peer support run by and for trans people. US: 877-565-8860
- [988 Suicide & Crisis Lifeline](https://988lifeline.org/) — call or text 988
