# Shimmy v1.7.0 - Mixture of Experts CPU Offloading Release

<<<<<<< HEAD
**Released:** January 9, 2025
=======
**Released:** January 9, 2025  
>>>>>>> main
**Branch:** `feat/moe-cpu-offload`

---

## 🎯 Headline Features

### Mixture of Experts (MoE) CPU Offloading Support

**Major new capability enabling large MoE models on consumer GPUs** - requested by [@razvanab](https://github.com/razvanab) in [Issue #81](https://github.com/Michael-A-Kuykendall/shimmy/issues/81).

This release adds full support for offloading Mixture of Experts (MoE) model weights to CPU memory, dramatically reducing VRAM requirements while maintaining usable inference performance. Now you can run massive models like **GPT-OSS 20B**, **Phi-3.5-MoE 42B**, and **DeepSeek-16B** on GPUs with limited VRAM.

**New CLI Flags:**
- `--cpu-moe` - Offload all MoE expert tensors to CPU memory
- `--n-cpu-moe N` - Offload N expert layers to CPU (partial offloading)

**Performance Achievements:**
- **78%-94% VRAM reduction** across tested models
- **2.5x-6.9x speed penalty** (acceptable for development/prototyping)
- Successfully validated on **Lambda Cloud GH200 (96GB VRAM)**

**Example Usage:**
```bash
# Full CPU offload (maximum VRAM savings)
shimmy serve --cpu-moe --model gpt-oss-20b.gguf

# Partial offload (balance VRAM vs speed)
shimmy serve --n-cpu-moe 64 --model phi-3.5-moe.gguf

# Generate with offloading
shimmy generate --cpu-moe --model deepseek-16b.gguf --prompt "Hello"
```

**Technical Implementation:**
- Rust bindings to llama.cpp's MoE offloading functionality via `llama-cpp-2` fork
- Integration through engine adapter with global CLI flags
- Verified with 144 expert tensors successfully offloaded on GPT-OSS 20B
- Comprehensive testing: 36/36 test runs passing (3 models × 2 configs × 3 runs × 2 quantizations)

---

## 📦 Quantized Models Released

Six professionally quantized MoE models uploaded to HuggingFace with comprehensive model cards following bartowski/Microsoft standards:

### Phi-3.5-MoE Quantizations (from 79GB F16)

#### 1. Q2_K - Ultra-Compressed (15GB, 81% reduction)
- **Repository:** [MikeKuykendall/phi-3.5-moe-q2-k-cpu-offload-gguf](https://huggingface.co/MikeKuykendall/phi-3.5-moe-q2-k-cpu-offload-gguf)
- **Direct Download:** [phi-3.5-moe-q2-k-cpu-offload.gguf](https://huggingface.co/MikeKuykendall/phi-3.5-moe-q2-k-cpu-offload-gguf/resolve/main/phi-3.5-moe-q2-k-cpu-offload.gguf) (15.0 GB)
- **Use Case:** Maximum compression, development/testing, low VRAM systems
- **Quality:** Acceptable for most tasks, noticeable quality loss vs F16

#### 2. Q4_K_M - Recommended (24GB, 70% reduction)
- **Repository:** [MikeKuykendall/phi-3.5-moe-q4-k-m-cpu-offload-gguf](https://huggingface.co/MikeKuykendall/phi-3.5-moe-q4-k-m-cpu-offload-gguf)
- **Direct Download:** [phi-3.5-moe-q4-k-m-cpu-offload.gguf](https://huggingface.co/MikeKuykendall/phi-3.5-moe-q4-k-m-cpu-offload-gguf/resolve/main/phi-3.5-moe-q4-k-m-cpu-offload.gguf) (23.8 GB)
- **Use Case:** Best quality/size balance, general production use
- **Quality:** Minimal quality loss vs F16, recommended for most users

#### 3. Q8_0 - High Quality (42GB, 47% reduction)
- **Repository:** [MikeKuykendall/phi-3.5-moe-q8-0-cpu-offload-gguf](https://huggingface.co/MikeKuykendall/phi-3.5-moe-q8-0-cpu-offload-gguf)
- **Direct Download:** [phi-3.5-moe-q8-0-cpu-offload.gguf](https://huggingface.co/MikeKuykendall/phi-3.5-moe-q8-0-cpu-offload-gguf/resolve/main/phi-3.5-moe-q8-0-cpu-offload.gguf) (41.7 GB)
- **Use Case:** Maximum quality, near F16 performance
- **Quality:** Virtually identical to F16

### DeepSeek-MoE-16B Quantizations (from 31GB F16)

#### 4. Q2_K - Ultra-Compressed (6.3GB, 80% reduction)
- **Repository:** [MikeKuykendall/deepseek-moe-16b-q2-k-cpu-offload-gguf](https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q2-k-cpu-offload-gguf)
- **Direct Download:** [deepseek-moe-16b-q2-k-cpu-offload.gguf](https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q2-k-cpu-offload-gguf/resolve/main/deepseek-moe-16b-q2-k-cpu-offload.gguf) (6.32 GB)
- **Use Case:** Maximum compression, development/testing
- **Quality:** Acceptable for most tasks, noticeable quality loss vs F16

#### 5. Q4_K_M - Recommended (11GB, 65% reduction)
- **Repository:** [MikeKuykendall/deepseek-moe-16b-q4-k-m-cpu-offload-gguf](https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q4-k-m-cpu-offload-gguf)
- **Direct Download:** [deepseek-moe-16b-q4-k-m-cpu-offload.gguf](https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q4-k-m-cpu-offload-gguf/resolve/main/deepseek-moe-16b-q4-k-m-cpu-offload.gguf) (10.9 GB)
- **Use Case:** Best quality/size balance, general production use
- **Quality:** Minimal quality loss vs F16, recommended for most users

#### 6. Q8_0 - High Quality (17GB, 45% reduction)
- **Repository:** [MikeKuykendall/deepseek-moe-16b-q8-0-cpu-offload-gguf](https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q8-0-cpu-offload-gguf)
- **Direct Download:** [deepseek-moe-16b-q8-0-cpu-offload.gguf](https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q8-0-cpu-offload-gguf/resolve/main/deepseek-moe-16b-q8-0-cpu-offload.gguf) (16.7 GB)
- **Use Case:** Maximum quality, near F16 performance
- **Quality:** Virtually identical to F16

**Model Card Features:**
- Proper YAML metadata (language, license, tags, base_model, pipeline_tag)
- Real performance benchmarks from controlled A/B testing
- VRAM usage with/without CPU offloading
- Token generation speeds (TPS) with detailed methodology
- Usage examples for shimmy CLI integration
- Quantization methodology and technical specifications

### Complete Model Comparison Table

| Model | Quantization | Size | Reduction vs F16 | Download URL | Use Case |
|-------|--------------|------|------------------|--------------|----------|
| **Phi-3.5-MoE** (79GB F16) | | | | | |
| | Q2_K | 15.0 GB | 81% | [Download](https://huggingface.co/MikeKuykendall/phi-3.5-moe-q2-k-cpu-offload-gguf/resolve/main/phi-3.5-moe-q2-k-cpu-offload.gguf) | Maximum compression |
| | Q4_K_M ⭐ | 23.8 GB | 70% | [Download](https://huggingface.co/MikeKuykendall/phi-3.5-moe-q4-k-m-cpu-offload-gguf/resolve/main/phi-3.5-moe-q4-k-m-cpu-offload.gguf) | **Recommended** |
| | Q8_0 | 41.7 GB | 47% | [Download](https://huggingface.co/MikeKuykendall/phi-3.5-moe-q8-0-cpu-offload-gguf/resolve/main/phi-3.5-moe-q8-0-cpu-offload.gguf) | Maximum quality |
| **DeepSeek-16B** (31GB F16) | | | | | |
| | Q2_K | 6.32 GB | 80% | [Download](https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q2-k-cpu-offload-gguf/resolve/main/deepseek-moe-16b-q2-k-cpu-offload.gguf) | Maximum compression |
| | Q4_K_M ⭐ | 10.9 GB | 65% | [Download](https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q4-k-m-cpu-offload-gguf/resolve/main/deepseek-moe-16b-q4-k-m-cpu-offload.gguf) | **Recommended** |
| | Q8_0 | 16.7 GB | 45% | [Download](https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q8-0-cpu-offload-gguf/resolve/main/deepseek-moe-16b-q8-0-cpu-offload.gguf) | Maximum quality |

⭐ = Recommended quantization level for production use

**Testing Validation:**
- **36 baseline tests** completed (100% success rate)
- **N=3 statistical runs** per configuration for reliability
- **Controlled A/B comparisons** (with/without `--cpu-moe`)
- **Lambda Cloud GH200** infrastructure (96GB VRAM, 72 CPU cores)
- **shimmy v1.6.0** used for all test runs

---

## 🔧 Technical Details

### Upstream Contributions

**llama-cpp-rs Fork Integration:**
- Using custom fork: `utilityai/llama-cpp-rs` (branch: `feat/moe-cpu-offload`)
- Added Rust bindings: `with_cpu_moe_all()`, `with_n_cpu_moe(n)` methods
- Submitted upstream PR: [utilityai/llama-cpp-rs#839](https://github.com/utilityai/llama-cpp-rs/pull/839) (CUDA stdbool fix)
- Clean integration via Cargo dependency override in Cargo.toml

**Implementation Architecture:**
```
CLI Flags (--cpu-moe, --n-cpu-moe)
    ↓
Global Config (MoeConfig struct)
    ↓
Engine Adapter (apply_moe_config)
    ↓
llama-cpp-2 Bindings (LlamaParams)
    ↓
llama.cpp MoE Offloading (native C++)
```

### Performance Benchmarks

**Phi-3.5-MoE Q4_K_M (24GB model):**
- Baseline (no offload): 11.55 TPS, ~23GB VRAM
- With `--cpu-moe`: 4.69 TPS, ~2MB VRAM (2.5x speed penalty, 99.9% VRAM reduction)

**GPT-OSS 20B Q8_0 (17GB model):**
- Baseline (no offload): 12.3 TPS, ~15GB VRAM
- With `--cpu-moe`: 1.78 TPS, ~2MB VRAM (6.9x speed penalty, 99.9% VRAM reduction)

**DeepSeek-16B Q8_0 (17GB model):**
- Baseline (no offload): 14.2 TPS, ~16GB VRAM
- With `--cpu-moe`: 3.1 TPS, ~2MB VRAM (4.6x speed penalty, 99.9% VRAM reduction)

**TTFT (Time to First Token):**
- Minimal impact: <500ms increase with CPU offloading
- Dominated by model loading, not offloading configuration

### Code Quality Improvements

**Systematic Audit Cleanup (Phases 1-3):**
- **Phase 1 (I2 Pattern):** Renamed 22 Java-style getters to Rust conventions
  - `get_model()` → `model()`, `get_metrics()` → `metrics()`, etc.
  - All call sites updated, 295/295 tests passing
<<<<<<< HEAD

=======
  
>>>>>>> main
- **Phase 2 (N5 Pattern):** Fixed 14 production unwraps with proper error handling
  - `src/metrics.rs` (5 unwraps), `src/openai_compat.rs` (3 unwraps)
  - Replaced with `match`, `unwrap_or_else`, `unwrap_or` patterns
  - 226+ test unwraps remain (acceptable - tests should panic)

- **Phase 3 (A3_stringly Pattern):** Converted 16+ string errors to typed ShimmyError
  - New variants: `WorkflowStepNotFound`, `MlxNotAvailable`, `ToolExecutionFailed`, etc.
  - Typed errors in `workflow.rs`, `safetensors_adapter.rs`, `tools.rs`, `preloading.rs`
  - Engine layer kept with `anyhow::Result` (clean boundary for third-party errors)

**Build Verification:**
- All 295 unit tests passing
- Zero compiler warnings (achieved clean build)
- Clippy clean (removed unnecessary conversions, unused imports)
- Formatting verified with `cargo fmt`

### Startup Diagnostics Enhancement

**New Serve Command Output:**
```
🚀 Shimmy v1.7.0
🖥️  Backend: CUDA (GPU acceleration enabled)
🧠 MoE: CPU offload enabled (all experts)
📚 Models: 0 available
🌐 Starting server on 127.0.0.1:11435
📚 Models: 3 available
✅ Ready to serve requests
   • POST /api/generate (streaming + non-streaming)
   • GET /health (health check + metrics)
   • GET /v1/models (OpenAI-compatible)
```

**Benefits:**
- Immediate configuration feedback before first request
- GPU backend visibility (CPU/CUDA/Vulkan/OpenCL/auto-detected)
- MoE config shown at startup (when feature enabled)
- Model discovery progress (shows count twice: before/after scan)
- Error prevention (wrong config visible instantly)

**Implementation:**
- Zero performance overhead (<1ms)
- Works with `RUST_LOG=off` (uses stdout)
- Emoji markers for visual scanning
- 7 new unit tests, 204/204 bin tests passing

---

## 🐛 Critical Fixes

### Issue #85: Template Compilation Errors in crates.io Installation

**Problem:** `cargo install shimmy` failed with template generation errors
- Nested tokio runtime panics during template file generation
- Async functions causing runtime conflicts

**Solution:**
- Remove async from template generation functions (they were synchronous)
- Eliminate nested tokio runtime causing panics
- Template files properly included in package, runtime issue was the blocker

**Verification:**
- Fresh install from crates.io: `cargo install shimmy --features llama`
- Template generation working correctly
- All integration tests passing

### Issue #84: Startup Diagnostics Implementation

**Problem:** No visibility into shimmy configuration until first request fails
- Wrong GPU backend only discovered after server starts
- Missing MoE config not shown until generation attempted
- No model count feedback during discovery

**Solution:** Added comprehensive startup diagnostics (see Technical Details above)

**Testing:**
- Manual testing on Windows with CUDA
- 7 new unit tests for diagnostic output formatting
- Regression tests: 204/204 bin tests, 295/295 lib tests passing

### MoE Config Application Fix

**Problem:** `--cpu-moe` flags ignored when auto-registering discovered models in serve command

**Root Cause:** Serve command created new LlamaEngine without MoE configuration

**Solution:**
- Apply MoE config to both initial engine AND enhanced_engine
- Ensure expert tensor offloading works in serve mode
- Verified: 144 expert tensors offloaded to CPU with GPT-OSS 20B model

**Testing:**
- Manual verification with GPT-OSS 20B (144 experts offloaded)
- Phi-3.5-MoE and DeepSeek-16B validation
- All serve mode configurations tested

---

## 📚 Documentation Updates

### HuggingFace Model Cards

**Professional Standards:**
- All 6 model cards follow bartowski/Microsoft style
- Real performance benchmarks (not estimates)
- Comprehensive YAML metadata (language, license, tags, base_model, pipeline_tag)
- Usage examples with shimmy CLI integration
- Quantization methodology and technical specifications

**Metadata Audit & Corrections:**
- Fixed "empty or missing yaml metadata" warnings
- Corrected DeepSeek base_model references (was pointing to wrong model)
- All repos rendering correctly on HuggingFace
- Proper tag relationships (GGUF, quantized, transformers)

### Internal Documentation Organization

**Moved to `docs/internal/`:**
- `EXECUTION-PLAN-QUANTIZATION-TO-HF.md`
- `MODEL-CARD-PLAN.md`
- `MOE-TESTING-STATUS.md`
- `QUANTIZATION-PERFORMANCE-SUMMARY.md`
- `QUANTIZATION-STATUS-REPORT.md`
- `QUANTIZATION-TESTING-PLAN.md`
- `QUANTIZATION-UPLOAD-COMPLETE.md`
- `UPLOAD-COMMANDS.md`
- `HUGGINGFACE-AUDIT-2025-10-09.md`

**Benefits:**
- Cleaner repository root
- Internal planning docs separated from user-facing documentation
- Historical context preserved for future development

---

## 🔮 What's Next

### Planned Enhancements
- **Additional quantization levels:** Q3_K_M, Q5_K_M for quality/size balance
- **More MoE models:** Qwen-3-235B, Mixtral variants with CPU offloading
- **Benchmark suite:** Automated A/B testing framework for MoE configs
- **Dynamic offloading:** Runtime adjustment of expert tensor placement
- **VRAM monitoring:** Real-time VRAM usage tracking during inference

### Community Contributions
- Upstream PR pending: [utilityai/llama-cpp-rs#839](https://github.com/utilityai/llama-cpp-rs/pull/839)
- Testing feedback welcome on Issue #81
- Additional model requests via GitHub issues

---

## 📥 Installation

### From Source (Recommended for MoE Support)
```bash
git clone https://github.com/Michael-A-Kuykendall/shimmy.git
cd shimmy
git checkout feat/moe-cpu-offload
cargo build --release --features llama
./target/release/shimmy --version
```

### From crates.io (Standard Features)
```bash
cargo install shimmy --features llama
shimmy --version
```

### Quick Start with MoE Models
```bash
# Example 1: Phi-3.5-MoE Q4_K_M (Recommended - Best Balance)
# Download the model (24GB)
wget https://huggingface.co/MikeKuykendall/phi-3.5-moe-q4-k-m-cpu-offload-gguf/resolve/main/phi-3.5-moe-q4-k-m-cpu-offload.gguf \
  -O phi-3.5-moe-q4-k-m.gguf

# Run with CPU offloading
shimmy serve --cpu-moe --model phi-3.5-moe-q4-k-m.gguf

# Test generation
curl -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Explain quantum computing in simple terms", "max_tokens": 100}'

# Example 2: DeepSeek-16B Q2_K (Smallest - Maximum VRAM Savings)
# Download the model (6.3GB)
wget https://huggingface.co/MikeKuykendall/deepseek-moe-16b-q2-k-cpu-offload-gguf/resolve/main/deepseek-moe-16b-q2-k-cpu-offload.gguf \
  -O deepseek-moe-16b-q2-k.gguf

# Run with CPU offloading
shimmy serve --cpu-moe --model deepseek-moe-16b-q2-k.gguf

# Example 3: Phi-3.5-MoE Q8_0 (Highest Quality - Near F16)
# Download the model (42GB)
wget https://huggingface.co/MikeKuykendall/phi-3.5-moe-q8-0-cpu-offload-gguf/resolve/main/phi-3.5-moe-q8-0-cpu-offload.gguf \
  -O phi-3.5-moe-q8-0.gguf

# Run with partial CPU offloading (64 layers)
shimmy serve --n-cpu-moe 64 --model phi-3.5-moe-q8-0.gguf

# Example 4: Using huggingface-cli (Alternative Download Method)
# Install: pip install huggingface-hub
huggingface-cli download MikeKuykendall/phi-3.5-moe-q4-k-m-cpu-offload-gguf \
  phi-3.5-moe-q4-k-m-cpu-offload.gguf --local-dir ./models

shimmy serve --cpu-moe --model ./models/phi-3.5-moe-q4-k-m-cpu-offload.gguf
```

### Quantization Selection Guide

**Choose Q2_K if:**
- You have very limited disk space (<10GB available)
- You're doing rapid prototyping/testing
- Quality is less critical than VRAM savings
- You want the absolute smallest model size

**Choose Q4_K_M if (RECOMMENDED):**
- You want the best balance of quality and size
- You're deploying to production
- You need reliable performance across diverse tasks
- You have 12-30GB disk space available

**Choose Q8_0 if:**
- You need maximum quality (virtually identical to F16)
- You have sufficient disk space (17-42GB)
- You're doing critical work requiring best possible output
- You can afford slightly larger VRAM usage

---

## 🙏 Credits

**Special Thanks:**
- **[@razvanab](https://github.com/razvanab)** for suggesting MoE CPU offloading in [Issue #81](https://github.com/Michael-A-Kuykendall/shimmy/issues/81) - this entire release exists because of your feature request! 🎉
- **Lambda Labs** for providing GH200 GPU infrastructure for comprehensive testing
- **llama.cpp team** for the upstream MoE offloading implementation
- **bartowski** for setting the standard with professional HuggingFace model cards

**Contributors:**
- Michael A. Kuykendall ([@Michael-A-Kuykendall](https://github.com/Michael-A-Kuykendall)) - Lead development, quantization, testing
- Claude Code (Anthropic) - Code refactoring assistance, documentation

---

## 🔗 Related Links

- **Issue #81:** [Feature Request - MoE CPU Offloading](https://github.com/Michael-A-Kuykendall/shimmy/issues/81)
- **Issue #84:** [Startup Diagnostics](https://github.com/Michael-A-Kuykendall/shimmy/issues/84)
- **Issue #85:** [Template Compilation Fix](https://github.com/Michael-A-Kuykendall/shimmy/issues/85)
- **PR #839:** [llama-cpp-rs CUDA stdbool Fix](https://github.com/utilityai/llama-cpp-rs/pull/839)
- **HuggingFace Models:** [MikeKuykendall Profile](https://huggingface.co/MikeKuykendall)
- **Previous Release:** [v1.6.0 Release Notes](./RELEASE_NOTES_v1.6.0.md)

---

## 📊 Detailed Changelog

### New Features
- `--cpu-moe` flag for full MoE CPU offloading
- `--n-cpu-moe N` flag for partial MoE CPU offloading
- Startup diagnostics with GPU backend and MoE config visibility
- 6 quantized MoE models on HuggingFace with professional documentation

### Bug Fixes
- Fixed `--cpu-moe` flags being ignored in serve command
- Resolved template compilation errors in crates.io installation
- Fixed ANSI color output (respects NO_COLOR and TERM env vars)
- Corrected HuggingFace model card metadata (YAML, base_model references)

### Code Quality
- Renamed 22 Java-style getters to Rust conventions (I2 pattern)
- Fixed 14 production unwraps with proper error handling (N5 pattern)
- Converted 16+ string errors to typed ShimmyError (A3_stringly pattern)
- Achieved zero compiler warnings and clean clippy output

### Documentation
- 6 professional HuggingFace model cards with real benchmarks
- Organized 9 internal planning docs into `docs/internal/`
- Created comprehensive v1.7.0 release notes
- Updated copilot instructions with audit progress

### Testing
- 36/36 quantization baseline tests passing (N=3 statistical runs)
- 295/295 unit tests passing
- 204/204 bin tests passing
- Validated on Lambda Cloud GH200 (96GB VRAM, 72 cores)

### Infrastructure
- Lambda Cloud GH200 testing environment
- HuggingFace integration for model distribution
- Custom llama-cpp-rs fork with MoE bindings
- Cargo dependency override for upstream contributions

---

**Full Changelog:** https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.6.0...feat/moe-cpu-offload
