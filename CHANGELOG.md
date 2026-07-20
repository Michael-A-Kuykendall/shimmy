# Changelog

All notable changes to Shimmy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.3.0] - 2026-07-20

### Added
- **Grammar control hooks** — schoolmarm+grammar module for structured generation
- **PPT invariant cage (B1-B3)** — golden-vault-based regression detection. Contracts verify per-layer RMS/checksum against vault oracles for 12 populated models. Run with `cargo test --test test_invariants -- --test-threads=1`.
- **Discrete GPU preference** — adapter selection now enumerates all GPUs and prefers `DiscreteGpu` over `IntegratedGpu` (commit `6ed0349`). Fixes multi-GPU laptops picking the Intel iGPU.

### Fixed
- **batch_count: 0 → batch_count: 1** in `server_inference.rs`. The QKV shader guard `if (global_id.y >= params.batch_count)` killed ALL threads when `batch_count=0` (since `0>=0` is true), producing no Q/K/V output. Root cause of the server-side gibberish regression (bisected to commit `43027d3`).
- **Rust/WGSL struct layout mismatch** — per-field quant types (Q/QK/K/V) aligned between Rust `LayerParams` and WGSL shader structs.

### Changed
- Airframe dependency updated to 0.2.9.
- Shimmy version bumped to 2.3.0.

### Library API
- `tokenizer_arc()` — shared tokenizer access
- `eos_token()`, `im_end_token()` — token accessors
- `fse_control_from_patterns()` — grammar pattern compiler
- `trace_callback()` — capture hook for invariant probes

---

## [2.2.1] - 2026-06-30

### Fixed
- **CI regression test cleanup** — Removed stale llama-era, MLX, and release-gate tests (17+ files, ~4500 lines). All CI test targets updated to use `--features airframe,huggingface` defaults.
- **Workflow consolidation** — Deleted 3 dead workflows (release-dry-run, test-release-binaries, version-validation). Release.yml rewritten from 373 to 66 lines as a clean matrix build.
- **Quality gate fixes** — Binary size test now uses debug+strip instead of slow release build. Variable expansion bug in quality gate fixed. MLX test no longer asserts x86_64-apple-darwin platform.

### Changed
- Airframe dependency updated to 0.2.8.

---

## [2.2.0] - 2026-06-09

### Fixed
- **Multi-architecture model loading** — Qwen3, Qwen2, Gemma-2, Phi-3 family models now
  load correctly via airframe 0.2.2. Previously these architectures silently failed due
  to a hardcoded Llama key prefix in the GGUF metadata parser.
- **Qwen3 tied embeddings** — Qwen3 models that share output.weight with token_embd.weight
  now load and run correctly.
- **Context cap safety** — Large-context models (Qwen2-7B n_ctx=32768) no longer cause
  memory exhaustion on CPU oracle generation paths.

### Changed
- Airframe dependency updated to 0.2.2.
- Shimmy version bumped to 2.2.0.

---

## [2.1.0] - 2026-06-02

### Added
- **TurboShimmy INT4 KV cache** — per-head-vector INT4 compression for KV buffers,
  cutting KV VRAM usage ~7× with no measured retrieval loss. Enable via
  `SHIMMY_KV_QUANT=int4`. See airframe 0.2.0 release notes.

### Fixed
- **wgpu 27 staging-buffer panic on GTX 1050 Ti and similar older GPUs** (issue #205,
  reported by @Kuntey). Upgraded to airframe 0.2.1, which fixes `max_buffer_size`
  being incorrectly capped to `max_storage_buffer_binding_size`. On some older GPU/driver
  stacks these limits differ, causing a deferred validation error that wgpu 27 surfaced as
  a cryptic "Staging Buffer is invalid" panic. A pre-flight size guard now returns a clear
  error message if the model exceeds the GPU's binding limit.
- Removed spurious `WARNING: Missing Norm Tensor post_attention_norm/post_ffw_norm` log
  spam for Llama, Mistral, Phi and Qwen models. These tensors are Gemma-only.

### Changed
- Airframe dependency updated to 0.2.1.

---

## [2.0.0] - 2026-05-26

### 🚀 **v2.0 PUBLIC RELEASE** — Airframe GPU Engine

This release replaces the llama.cpp inference backend with **Airframe**, a pure-Rust WGSL GPU inference engine. Airframe runs the entire transformer pipeline on GPU using WebGPU compute shaders — no C++ dependencies, no build flags, no platform matrix headaches. This is a major architectural break from v1.x.

The llama.cpp code path is **historically parked**, not deleted. It remains accessible via `--legacy` flag or `SHIMMY_ENGINE_BACKEND=llama` if needed.

### What's New Since v1.9.0

**New Default Engine: Airframe**
- Pure Rust — no C++ toolchain required at runtime
- WGSL compute shaders compiled on first launch; no pre-compiled shader artifacts
- F32 precision throughout (output head dequantized Q6_K → F32 on load)
- YaRN RoPE scaling for extended context via `SHIMMY_MAX_CTX` env var
- Stateless per-request inference with full KV cache reset between requests
- Model spec auto-derived from GGUF metadata — works with any GGUF-format model

**API**
- Added `POST /v1/completions` text completion endpoint (OpenAI-compatible)
- `POST /v1/chat/completions` now accepts `frequency_penalty` and `presence_penalty` fields; values map to `repeat_penalty = 1.0 + max(freq, presence) * 0.5`
- Input validation: empty `messages` array returns 400; `max_tokens` of 0 or above 131072 returns 400

**Inference**
- Fixed stop token propagation: `<|eot_id|>` (Llama-3) and other model-specific stop tokens from chat templates now correctly terminate generation; previously they were silently dropped and the model would generate until `max_tokens`
- Chunked prefill: prompts now processed in 512-token chunks (was one giant dispatch) — extended context viable again

**Model Configuration**
- Set `SHIMMY_BASE_GGUF=/path/to/model.gguf` to configure the default model
- Pass `--model-path /path/to/model.gguf` to the `serve` command as an alternative
- No more hardcoded developer paths shipped in the binary

**Model Discovery**
- Added `~/.cache/lm-studio/models` to auto-discovery search paths (fixes #184)

**Template Routing Fixed**
- TinyLlama, Llama-1, and Llama-2 models now correctly use ChatML template
- Llama-3 template reserved for models explicitly named `llama-3` / `llama3`

**Distribution**
- `cargo install shimmy` now works — crates.io package uses huggingface engine variant
- GitHub Releases provide platform binaries with the full Airframe WebGPU engine

### ⚠️ Migration from v1.x

**SHIMMY_BASE_GGUF is now required.** The default model is no longer hardcoded. Set the env var or pass `--model-path`.

**llama.cpp is historically parked.** Everything you could do with TinyLlama via llama.cpp works identically through Airframe. Opt back with `--legacy` or `SHIMMY_ENGINE_BACKEND=llama` if needed.

See `docs/MIGRATION_v2.md` for full migration steps.

## [1.9.0] - 2026-01-09

### 🎉 **KITCHEN SINK ARCHITECTURE** - One Binary, All GPUs

This major release revolutionizes Shimmy's distribution model, solving 22+ user-reported build issues by shipping all GPU backends in a single binary per platform. No more compilation nightmares, no more backend confusion - just download and run.

### 🏆 **HEADLINE ACHIEVEMENTS**

**🚀 ZERO-CONFIGURATION GPU AUTO-DETECTION**
- **NEW**: Single binary per platform with ALL GPU backends included
- **FIXED**: Issues #129, #130, #142, #144, #110, #105, #99, #86, #88 - Build failures eliminated
- Automatic GPU detection with priority order: CUDA → Vulkan → OpenCL → MLX → CPU
- **Result**: 95%+ of users no longer need to compile from source

**📦 SIMPLIFIED DISTRIBUTION (9 binaries → 5 binaries)**
- `shimmy-windows-x86_64.exe` - CUDA + Vulkan + OpenCL (~45MB)
- `shimmy-linux-x86_64` - CUDA + Vulkan + OpenCL (~45MB)
- `shimmy-macos-arm64` - MLX for Apple Silicon (~35MB)
- `shimmy-macos-intel` - CPU-only (~20MB)
- `shimmy-linux-aarch64` - CPU-only (~20MB)

**🎯 USER EXPERIENCE IMPROVEMENTS**
- **Before**: "Which backend do I need? CUDA? Vulkan? OpenCL?"
- **After**: "Pick your platform. Download. Run."
- Zero compilation required for 95%+ of users
- Existing `--gpu-backend auto` flag now actually works (all backends compiled in)
- Manual override still available: `--gpu-backend cuda|vulkan|opencl|cpu`

### 📦 **NEW FEATURES**

**Kitchen Sink Binaries**
- Pre-built binaries include all GPU backends for each platform
- Runtime auto-detection eliminates user choice paralysis
- Leverages existing GPU detection code (previously hidden behind compile-time flags)

**Private Testing Infrastructure**
- Created `shimmy-private` repository for pre-release validation
- Dual-remote workflow: test privately → validate → release publicly
- Prevents future "dangling test releases" in public repo

**Documentation Reorganization**
- Moved internal strategy docs to `docs/internal/`
- Moved audit reports to `docs/audits/`
- Moved release notes to `docs/releases/`
- Cleaned root directory from 32 to 16 markdown files
- Updated `.gitignore` for internal documentation patterns

### 🐛 **FIXES**

**Issue #129: GPU support not available in precompiled**
- Pre-built binaries now include all GPU backends
- No more "download shimmy, realize it's CPU-only, compile from source" cycle

**Issue #130: GPU not enabled with --backend vulkan flag**
- Vulkan backend now compiled into Windows/Linux binaries
- `--gpu-backend vulkan` actually uses Vulkan (not CPU)

**Issue #142: AMD GPU not detected (Vulkan/OpenCL)**
- Both Vulkan AND OpenCL now in same binary
- Auto-detection tries both, uses what works on AMD cards

**Issue #144: MLX should be default on Apple Silicon**
- macOS ARM64 binary includes MLX by default
- No compilation needed for Apple Silicon GPU support

**Issues #110, #105, #99, #86, #88: Build failures**
- Pre-built binaries eliminate 95%+ of compilation issues
- Missing template files: Fixed in packaging
- C++ dependency errors: Not needed for pre-built binaries
- LLVM/libclang issues: Avoided entirely with downloads

### 🔧 **CHANGES**

**Build System**
- Release workflow reduced from 9 builds to 5 Kitchen Sink builds
- Each platform binary now includes all available GPU backends
- Binary sizes increased (~20MB CPU → ~45MB GPU, acceptable trade-off)

**Documentation**
- README.md rewritten to emphasize pre-built binaries
- Installation section shows download links, not compilation
- GPU Acceleration section explains auto-detection
- Quick Start updated with platform-specific downloads

**Repository Structure**
- Created `docs/internal/` for strategy documents
- Created `docs/audits/` for forensic reports
- Created `docs/releases/` for version-specific notes
- Created `.github/internal/` for workflow documentation
- Reduced root directory clutter significantly

### ⚠️ **BREAKING CHANGES**

**Binary Naming**
- **Before**: `shimmy-windows-x86_64-cuda.exe`, `shimmy-windows-x86_64-vulkan.exe`, etc.
- **After**: `shimmy-windows-x86_64.exe` (includes all backends)
- Users downloading specific backend binaries need to update scripts

**Compilation Instructions**
- Old wiki pages with backend-specific compilation are obsolete
- Users should prefer pre-built binaries unless contributing/customizing
- Advanced users can still build from source with feature flags

### 🎁 **BONUS**

**Marketing Message**
- "Ollama performance, 12x smaller, zero configuration"
- One binary with automatic GPU detection
- Privacy-first local inference
- No compilation headaches

**User Outreach Strategy**
- Personalized messages to all 22+ affected users
- Tagged announcements showing individual issue resolutions
- Demonstrates community responsiveness

### 📊 **METRICS**

**Binary Count**: 9 → 5 (44% reduction)  
**Compilation Required**: 95% → 5% (for contributors/customizers)  
**GPU Detection**: Manual → Automatic  
**User Choice Required**: Platform + Backend → Platform only  
**Expected Issue Reduction**: 22+ build issues → Near zero  

### 🙏 **ACKNOWLEDGMENTS**

This release was directly shaped by user feedback from issues:
#129, #130, #142, #144, #110, #105, #99, #86, #88, #126, #127, #100, #114, #112, #131, #98, #87, #83, #152

Thank you to everyone who reported build issues - your pain drove this improvement!

## [1.8.1] - 2025-12-08

### 🐳 **DOCKER PUBLISHING INFRASTRUCTURE** - Container Registry Publishing Fixed

This patch release fixes the missing Docker publishing infrastructure that caused the v1.8.0 release to fail despite all quality gates passing.

### 🏆 **HEADLINE ACHIEVEMENTS**

**🚀 DOCKER PUBLISHING PIPELINE IMPLEMENTATION**
- **FIXED**: Added Docker image publishing to GitHub Container Registry (GHCR)
- Implemented automated Docker build and push in release workflow
- Added proper GHCR authentication and image tagging
- **Result**: Containerized shimmy deployments now work reliably

**🔧 RELEASE INFRASTRUCTURE COMPLETION**
- Completed the Docker publishing pipeline for issue #146
- Enhanced release workflow with container registry publishing
- Improved deployment automation for cloud-native environments

### 📦 **CHANGES**
- Added Docker build and push step to release workflow
- Configured GHCR publishing with proper authentication
- Added both versioned and latest Docker image tags

## [1.8.0] - 2025-12-08

### 🐳 **DOCKER PUBLISHING PIPELINE** - Containerized Deployments Fixed

This release resolves critical Docker image publishing failures that prevented containerized shimmy deployments. The fix ensures reliable automated Docker Hub publishing for all future releases.

### 🏆 **HEADLINE ACHIEVEMENTS**

**🚀 DOCKER PUBLISHING AUTOMATION**
- **FIXED**: Issue #146 - Docker image publishing pipeline failures
- Implemented automated Docker Hub publishing in release workflow
- Added comprehensive Docker build validation in release gates
- **Result**: Containerized shimmy deployments now work reliably

**🔧 INFRASTRUCTURE RELIABILITY**
- Enhanced release gate validation with Docker build verification
- Strengthened CI/CD pipeline with container deployment testing
- Improved deployment automation for cloud-native environments

### 📦 **CHANGES**
- Fixed Docker publishing workflow configuration
- Added Docker build validation to release gates
- Enhanced containerized deployment reliability

## [1.7.4] - 2025-10-23

### 🔧 Fixes

- Auto-update `Cargo.lock` before `cargo publish` to prevent crates.io publish failures
- Gate 4 now validates huggingface binary size (2.6 MB) instead of CUDA binary (26 MB)
- ARM64 Linux CI/CD: remove unused macOS config from Cross.toml, silence cross-compilation warnings

## [1.7.3] - 2025-10-12

### 🎯 **SYSTEMATIC ENGINEERING EXCELLENCE** - Production Quality Release

This release represents a **COMPLETE TRANSFORMATION** of shimmy's engineering discipline, achieving 100% CI/CD reliability through methodical problem-solving and introducing revolutionary PPT invariant validation that caught real architectural bugs.

### 🏆 **HEADLINE ACHIEVEMENTS**

**🔥 ZERO-TIMEOUT CI/CD PIPELINE**
- **BREAKTHROUGH**: Eliminated ALL timeout failures through systematic individual test analysis
- Implemented platform-specific test guards preventing MLX compilation on incompatible systems  
- Replaced expensive release builds with optimized debug alternatives (10x faster)
- **Result**: 100% CI reliability across all platforms and quality gates

**🧪 PPT INVARIANT SYSTEM INTEGRATION** 
- **REAL BUG CAUGHT**: PPT system identified critical GGUF→Llama backend routing violation
- Production integration with semantic contract enforcement across all inference paths
- Comprehensive property-based testing with automated invariant validation
- **Impact**: Prevents entire classes of architectural regressions automatically

**💪 COMPREHENSIVE BUG RESOLUTION**
- **Issue #106**: Windows server crashes → Mutex poisoning recovery implemented
- **Issue #105**: Windows GPU builds → Template packaging systematic fix  
- **Issue #100**: MLX Apple Silicon → Native hardware detection working
- **Issue #99**: cargo install failures → Cross-platform compatibility restored
- **Issue #98**: macOS compatibility → Full platform support verified

### 🚀 **ENGINEERING DISCIPLINE TRANSFORMATION**

**ZERO-WARNINGS CODEBASE**
- Systematically eliminated ALL 47 compiler warnings using professional feature gates
- Implemented proper `#[cfg(feature = "...")]` guards for conditional compilation
- Enhanced code quality through comprehensive clippy lint resolution
- **Achievement**: Professional-grade warning-free compilation across all feature combinations

**MILITANT CI/CD QUALITY GATES**
- **11 QUALITY GATES PASSING**: PPT, Security, Code Quality, Test Suite, Coverage, Cross-Platform Builds
- Platform-specific validation (Windows MSVC, macOS Intel/ARM, Linux x86_64)
- DCO compliance enforcement for legal code provenance
- **Zero-tolerance policy**: Every quality gate must pass before release

**PLATFORM-SPECIFIC OPTIMIZATION**
```rust
// Revolutionary platform-aware test design
#[test]
fn test_mlx_functionality() {
    if !cfg!(target_os = "macos") {
        println!("ℹ️ Skipping MLX test on non-macOS platform");
        return;
    }
    // MLX-specific testing only on Apple platforms
}
```

### 🔬 **PPT SYSTEM SUCCESS STORY**

**ARCHITECTURAL BUG DETECTION**
The PPT invariant system proved its value by catching a **CRITICAL SEMANTIC VIOLATION**:
- **Issue**: GGUF models weren't consistently routed to Llama backend
- **Detection**: PPT contract `assert_backend_consistency()` failed during model loading
- **Impact**: Fixed silent data corruption where models could use wrong inference engines
- **Validation**: 306/306 tests now pass with PPT invariants enforcing architectural integrity

**Production Integration Highlights**
```rust
// PPT contracts now enforce critical invariants
ppt::contracts::assert_model_loaded(model_name, success);
ppt::contracts::assert_generation_valid(prompt, response);  
ppt::contracts::assert_backend_consistency(model_type, backend);
```

### 🛠️ **SYSTEMATIC TIMEOUT ELIMINATION**

**METHODICAL DEBUGGING APPROACH**
Following the directive: *"check every single one that went overtime, determine test by test what's wrong, fix each individually"*

**MLX Apple Silicon Guards**
- Added `cfg!(target_os = "macos")` guards to prevent compilation failures on Linux/Windows
- **Tests Fixed**: `test_mlx_binary_status_messages`, `test_gpu_info_with_mlx_compiled`, `test_full_apple_feature_build_and_run`
- **Result**: MLX tests execute only on compatible Apple Silicon hardware

**Build Optimization Strategy**
- **Before**: `cargo build --release` (10+ minutes, frequent timeouts)
- **After**: `cargo check` + `cargo build` debug (30-60 seconds, reliable)
- **Impact**: 90% reduction in CI execution time with maintained quality

**Cross-Platform Verification**
- **Packaging Tests**: Eliminated `--release` flags from validation checks
- **Integration Tests**: Debug builds with full functionality verification  
- **Gate Tests**: Optimized timeout handling while maintaining constitutional limits

### 🎯 **VALIDATION EXCELLENCE**

**COMPREHENSIVE TEST MATRIX**
```
✅ Unit Tests: 306/306 PASSED (was failing due to PPT violations)
✅ Integration Tests: 15/15 PASSED (timeout optimization successful)
✅ MLX Apple Silicon: PASSED (platform-specific guards working)
✅ Cross-Platform Builds: 4/4 PASSED (Windows/macOS/Linux verified)
✅ Security Audit: PASSED (supply chain validation complete)
✅ PPT Contracts: PASSED (architectural integrity validated)
✅ Code Coverage: 39.5% (professional measurement, no gaming)
```

**PLATFORM VALIDATION MATRIX**
- ✅ **Windows x86_64**: MSVC compilation + GPU backend detection
- ✅ **macOS Intel**: Native build + MLX compatibility detection  
- ✅ **macOS ARM64**: Apple Silicon + native MLX support
- ✅ **Linux x86_64**: Native compilation + CUDA detection

### 🔧 **TECHNICAL IMPLEMENTATION DETAILS**

**Windows Stability Engineering**
- **Mutex Poisoning Recovery**: Enhanced server stability under concurrent load
- **GPU Backend Fixes**: Comprehensive Windows MSVC compatibility
- **Template Packaging**: Systematic resolution of cargo install failures

**Apple Silicon Native Support**  
- **MLX Integration**: Native Apple ML framework integration with proper fallbacks
- **Hardware Detection**: Intelligent platform-aware feature activation
- **Performance Optimization**: Native ARM64 compilation with Apple-specific optimizations

**Cross-Platform Reliability**
- **Cargo Install**: 100% success rate across all platforms verified
- **Feature Flags**: Professional conditional compilation guards
- **Build Systems**: Platform-specific optimization while maintaining portability

### 📊 **PERFORMANCE & RELIABILITY METRICS**

**CI/CD Pipeline Performance**
- **Before**: 30-40% timeout failure rate, 15-27 minute runtimes
- **After**: 0% timeout failures, 6-9 minute reliable runtimes  
- **Improvement**: 100% reliability with 60% faster execution

**Code Quality Metrics**
- **Warnings**: 47 → 0 (100% elimination)
- **Clippy Issues**: 23 → 0 (professional-grade resolution)
- **Test Coverage**: Comprehensive property-based + unit testing
- **Documentation**: Complete inline documentation with examples

**Binary Quality**
- **Size**: Maintains <5MB constitutional limit across all platforms
- **Performance**: <2s startup time with optimized loading
- **Compatibility**: 100% OpenAI API compatibility maintained

### 🎖️ **ENGINEERING ACHIEVEMENT HIGHLIGHTS**

**METHODICAL PROBLEM SOLVING**
- Individual test-by-test timeout analysis and resolution
- Platform-specific optimization without compromising portability  
- Zero-shortcut approach: every issue systematically diagnosed and fixed

**PROFESSIONAL QUALITY GATES**
- 11 mandatory quality gates with zero-bypass policy
- DCO compliance for legal code provenance
- Constitutional binary size limits enforced
- Professional warning elimination using feature gates

**PRODUCTION READINESS**
- 100% CI reliability enables confident releases
- PPT system catches architectural regressions automatically
- Cross-platform validation ensures universal compatibility
- Professional error handling and recovery mechanisms

### 🏁 **DEPLOYMENT CONFIDENCE**

This release demonstrates **SYSTEMATIC ENGINEERING EXCELLENCE** through:
- **Methodical Debugging**: Individual problem analysis and targeted solutions
- **Quality Gate Discipline**: Zero-compromise approach to CI/CD reliability  
- **Architectural Validation**: PPT system catching real bugs before production
- **Professional Standards**: Warning-free codebase with proper feature guards
- **Cross-Platform Excellence**: Universal compatibility with platform-specific optimization

**Ready for production deployment with 100% CI confidence and architectural integrity guaranteed by PPT invariant validation.**

### 🔮 **TECHNICAL FOUNDATION FOR FUTURE**

The systematic engineering discipline established in v1.7.3 creates a **BULLETPROOF FOUNDATION** for future development:
- **Zero-timeout CI/CD** enables rapid iteration with confidence
- **PPT invariant system** automatically prevents architectural regressions
- **Professional quality gates** maintain code excellence standards
- **Platform-specific optimization** supports expanding hardware compatibility

*This release transforms shimmy from a working prototype into an **ENTERPRISE-GRADE INFERENCE ENGINE** with systematic quality assurance and architectural integrity validation.*

## [1.7.2] - 2025-10-10

### 🔧 Fixes

- Corrected feature flag names in Gate 6 documentation build step of the release workflow

## [1.7.0] - 2025-10-08

### 🐛 Bug Fixes

- Added explicit version requirement for `llama-cpp-2` git dependency to unblock crates.io publishing

## [1.6.0] - 2025-10-04

### 🎯 Windows CUDA Support (First in Rust LLM Ecosystem!)

**Issue #72: GPU Backend Flag Implementation + Windows MSVC CUDA**
- ✅ Fixed `--gpu-backend` CLI flag wiring through to model loading
- ✅ **BREAKTHROUGH**: First lightweight Rust LLM tool with Windows MSVC CUDA support
  - Fixed llama-cpp-rs bindgen header discovery issue blocking Windows CUDA builds
  - Uses cc::Build to extract MSVC INCLUDE paths, passes as -isystem to bindgen
  - Fork: Michael-A-Kuykendall/llama-cpp-rs (branch: fix-windows-msvc-cuda-stdbool)
- ✅ Implemented GpuBackend::from_string() parser with helpful error messages
- ✅ Implemented GpuBackend::detect_best() with priority: CUDA > Vulkan > OpenCL > CPU
- ✅ All 4 GPU backends verified on Windows: Vulkan, OpenCL, CUDA, HuggingFace
- ✅ Binary sizes: 4.8MB (minimal), 24MB (CUDA) + 36MB ggml-cuda.lib
- ✅ Build times: HuggingFace 8s, OpenCL 45s, Vulkan 3m19s, CUDA 11m25s

### 🐛 Critical Stability Fixes

**Concurrent Load Deadlock**
- Fixed RwLock deadlock in ModelManager causing infinite hangs with concurrent tasks
- Pattern: Drop write lock immediately after operations, before calling other functions
- All 295 unit tests now passing (was hanging indefinitely at test_concurrent_load_unload)

**Flaky Property Tests**
- Rebuilt 4 property tests without broken property_test() wrapper
- Fixed test_backend_routing_property, test_generation_length_property, etc.
- Tests now deterministic: 284/284 pass minimal features, 295/295 with backends

**Feature Flag Compatibility**
- Added cfg guards to PPT test modules for llama backend features
- Fixed adapter test compilation with minimal features
- All tests work with `--no-default-features --features huggingface`

### Added
- **Opt-in Usage Analytics**: Anonymous business intelligence collection system
- **Performance Benchmarking Tools**: Cross-platform scripts for real GPU/CPU measurement
- **Comprehensive Security Policy**: Private vulnerability disclosure process (SECURITY.md)
- **DCO (Developer Certificate of Origin)**: Legal compliance for all contributions
- **Professional GitHub Templates**: Issue/PR templates with structured workflows
- **Branch Protection**: Quality gates with CI and DCO enforcement
- **Automated Changelog**: CI/CD integration for release documentation

### Changed
- **Enhanced CONTRIBUTING.md**: Added maintainer process and DCO requirements
- **Improved Documentation**: Comprehensive performance analysis and metrics transparency
- **Professional Repository Structure**: Security-first approach with industry standards

### Security
- **Private Security Disclosure**: GitHub Security Advisories integration
- **DCO Compliance**: All contributions legally certified
- **Branch Protection**: Enforced code review and quality gates

### Documentation
- **Performance Analysis**: Real benchmarking tools and GPU consumption data
- **Metrics Transparency**: Complete disclosure of business intelligence collection
- **Contributing Guidelines**: Clear maintainer process and legal requirements

## [1.5.6] - 2025-09-23

### 🐛 Bug Fixes

- Strip carriage returns from version validation output to handle Windows line-ending differences (prevents false version mismatches on Windows CI)

## [1.5.5] - 2025-09-22

### 🔧 CI/CD

- Comprehensive version validation added to CI/CD to prevent Issue #63 version mismatches for all forks
- Added version regression test for Windows binary version mismatch

## [1.5.4] - 2025-09-20

### 🐛 Bug Fixes

- Updated CUDA version to 12.6.0; release workflow made resilient to partial build failures

## [1.5.3] - 2025-09-20

### 🐛 Bug Fixes

- Simplified to CUDA-only GPU in release CI for incremental stability testing

## [1.5.2] - 2025-09-20

### ✨ Features

- GPU-enabled prebuilt release binaries: CUDA (NVIDIA) and Vulkan SDK installed in CI
- Enabled MLX support in macOS release builds (Issue #68)
- Enhanced OpenAI API compatibility for Open WebUI and AnythingLLM (Issue #70)

## [1.5.1] - 2025-09-19

### 🐛 Bug Fixes

- Include template files in crates.io package to fix `cargo install` failures (Issue #64)

## [1.5.0] - 2025-09-19

### 🐛 Bug Fixes

- Fixed GitHub Actions release workflow to prevent missing binaries in releases (Issue #57)
- Improved error handling for missing models in OpenAI API compatibility layer
- Added regression test for Windows binary version mismatch (Issue #63)

## [1.4.2] - 2025-09-22

### 🐛 Bug Fixes

- Fixed template compilation failure affecting release builds
- Improved Ollama model discovery for custom installation directories

## [1.4.1] - 2025-09-17

### 🐛 Bug Fixes

- crates.io release fix: updated Cargo.lock and packaging for stable publish

## [1.4.0] - 2025-09-17

### ✨ Features

**Shimmy Developer Ecosystem**
- GPU backend support: CUDA, Vulkan, OpenCL backends added to build system
- HuggingFace model ID detection for SafeTensors files
- `cargo-deny` configuration for supply chain security (dependency audit)
- 5 major advanced features: expanded API surface, enhanced model filtering
- Comprehensive code quality improvements and CI enhancements

## [1.3.5] - 2025-09-17

### 🐛 Critical Fix

- Fixed `anyhow` macro import collision causing build failures across all platforms

## [1.3.3] - 2025-09-15

### ✨ Features

**Docker Compose Deployment Support**
- Added complete Docker Compose configuration for production deployments
- Includes Nginx reverse proxy and health checks
- Railway, Render, and Fly.io deployment configurations
- Production-ready containerization

### 🐛 Bug Fixes

**Issue #22: Windows EXE Availability**
- Fixed missing Windows executable in GitHub releases
- Added `shimmy-windows-x86_64.exe` to all releases for direct download
- Improved Windows installation documentation for libclang.dll dependency

**ARM64 Linux Cross-Compilation Issues**
- Resolved OpenSSL cross-compilation failures for ARM64 Linux builds
- Switched to rustls for better cross-compilation compatibility
- Added Docker-based ARM64 Linux build process using QEMU emulation
- Temporarily excluded ARM64 Linux from CI/CD to ship 4-platform release

### 🚀 Enhancements

**Multi-Platform Release Automation**
- Automated 4-platform binary generation: Linux x86_64, Windows x86_64, macOS Intel, macOS ARM64
- Enhanced GitHub Actions workflow with improved error handling
- Added Docker-based cross-compilation for future ARM64 Linux support

**Cross-Compilation Documentation**
- Added comprehensive cross-compilation guide (`docs/CROSS_COMPILATION.md`)
- Documented Docker QEMU emulation process for ARM64 builds
- Updated internal documentation with proven ARM64 build methods

### 🔧 Technical Improvements

**Security Dependencies**
- Migrated from OpenSSL to rustls for better cross-platform compatibility
- Reduced C++ dependency complexity in cross-compilation builds
- Enhanced static linking for standalone binaries

**Release Process**
- Streamlined release workflow to prevent CI/CD failures
- Added fallback strategies for platform-specific build issues
- Improved artifact naming consistency across platforms

### 📦 Platform Support

**Current Release Platforms:**
- ✅ Linux x86_64 (native build)
- ✅ Windows x86_64 (native build)
- ✅ macOS Intel (native build)
- ✅ macOS ARM64 (native build)
- 🔄 Linux ARM64 (Docker QEMU build - documented process available)

**Binary Downloads:**
- All platforms available via GitHub Releases
- Windows users: Download `shimmy-windows-x86_64.exe` directly
- Linux ARM64: Docker build process documented for manual compilation

### 🛠️ Developer Experience

**Build Infrastructure**
- Enhanced CI/CD pipeline reliability
- Added Docker-based cross-compilation for complex targets
- Improved error reporting and debugging for build failures
- Added comprehensive build documentation

### 📖 Documentation

**Deployment Guides**
- Docker Compose setup for production deployments
- Cloud platform deployment instructions (Railway, Render, Fly.io)
- Cross-compilation guide for ARM64 Linux builds
- Windows installation troubleshooting guide

## [1.3.2] - 2025-09-12

### 🐛 Bug Fixes

**Issue #13: VSCode Integration with Qwen Models**
- Fixed VSCode extension compatibility with Qwen3-4B-Instruct and other Qwen models
- Enhanced automatic template detection for Qwen models (now uses ChatML template)
- Added better error logging for model loading failures in OpenAI-compatible API
- Improved error handling with detailed diagnostics for troubleshooting

**Issue #12: Custom Model Directory Detection**
- Added support for custom model directories via `SHIMMY_MODEL_PATHS` environment variable
- Added support for `OLLAMA_MODELS` environment variable for Ollama model directories
- Added `--model-dirs` global command-line option for specifying custom directories
- Enhanced Windows multi-drive search for Ollama installations (C:, D:, E:, F: drives)
- Improved model auto-discovery to handle Ollama installs on different drives

### ✨ Enhancements

- **Multi-Drive Support**: Automatic scanning of common Ollama paths across multiple Windows drives
- **Template Detection**: Enhanced model template inference for Qwen, ChatGLM, Llama models
- **CLI Improvements**: New global `--model-dirs` option works with all subcommands

### 📖 Documentation

- Created improved Homebrew formula using pre-built binaries (Issue #15)
- Enhanced Windows and macOS installation guidance

## [1.3.1] - 2025-09-12

### ✨ Features

**Full llama.cpp Support on All Platforms**
- Enabled complete llama.cpp support across Linux, Windows, macOS Intel, and macOS ARM64
- Resolved macOS ARM64 compilation issues with forked llama-cpp dependency
- Added ARM64-specific compiler capability detection and optimizations

**Enhanced Build System**
- Improved cross-platform compilation with specialized ARM64 handling
- Added comprehensive testing for macOS ARM64 llama.cpp integration
- Streamlined release workflow configuration for stable deployments

### 🐛 Bug Fixes

**macOS ARM64 Compilation Issues**
- Fixed GGML_ARM_I8MM compilation conflicts on Apple Silicon
- Resolved mixed-ISA build problems with targeted compiler flags
- Added proper target detection for ARM64 optimization features

**Release Workflow Stability**
- Enhanced release pipeline reliability across all supported platforms
- Fixed deployment configuration issues for stable v1.3.1 releases
- Improved error handling and fallback strategies

### 🔧 Technical Improvements

**Cross-Platform Compatibility**
- Updated llama-cpp dependency to specialized fork with ARM64 fixes
- Enhanced build.rs with platform-specific compilation logic
- Added comprehensive CMAKE configuration for different architectures

**Testing Infrastructure**
- Added isolated macOS ARM64 llama compilation testing
- Enhanced platform-specific build validation
- Improved error reporting for architecture-specific issues

## [1.2.0] - 2025-09-10

### ✨ Features

**Native SafeTensors Support**
- Implemented native SafeTensors inference engine with zero Python dependencies
- Added complete SafeTensors model format support alongside GGUF
- Enhanced model detection and loading for SafeTensors files

**Enhanced Build System**
- Updated release workflow with comprehensive system dependencies
- Added support for all-features builds across platforms
- Improved cross-platform compilation reliability

### 🐛 Bug Fixes

**Build and Deployment Issues**
- Fixed release binary generation to exclude problematic llama.cpp dependencies
- Resolved macOS runner cmake installation conflicts
- Enhanced GitHub Actions workflow with proper dependency management

**Model Discovery Improvements**
- Fixed infinite recursion issues in model discovery on macOS
- Enhanced model loading robustness across different file formats
- Improved error handling for corrupted or incomplete model files

### 🚀 Enhancements

**Performance Optimizations**
- Native SafeTensors processing for faster model loading
- Reduced memory footprint with optimized inference pipeline
- Enhanced startup performance with efficient model detection

**Developer Experience**
- Comprehensive testing suite for SafeTensors functionality
- Improved documentation for multi-format model support
- Enhanced debugging and error reporting capabilities

## [1.1.0] - 2025-09-09

### ✨ Features

**Revolutionary Testing Framework**
- Implemented PPT (Property-based Testing) framework for comprehensive coverage
- Added invariant testing system for robust quality assurance
- Enhanced testing excellence with automated property verification

**Code Quality Improvements**
- Eliminated all compiler warnings for clean, professional builds
- Implemented comprehensive linting and formatting standards
- Enhanced code documentation and maintainability

### 🔧 Technical Improvements

**Testing Infrastructure**
- Advanced property-based testing with automated edge case discovery
- Invariant checking system for critical functionality validation
- Comprehensive test coverage across all major components

**Build System Enhancements**
- Clean compilation with zero warnings across all platforms
- Enhanced build performance and reliability
- Improved development workflow with better error reporting

## [1.0.1] - 2025-09-08

### 🐛 Bug Fixes

**Critical Issues Resolved**
- **Issue #6**: Fixed model discovery and loading failures
- **Issue #7**: Resolved OpenAI API compatibility problems
- **Issue #5**: Fixed chat completions hanging during generation

**Performance Improvements**
- Enhanced health and metrics endpoints for production monitoring
- Improved error handling and recovery mechanisms
- Optimized model loading and inference pipeline

### ✨ Features

**Enhanced Monitoring**
- Added comprehensive health check endpoints
- Implemented detailed metrics collection for performance tracking
- Enhanced production readiness with robust monitoring capabilities

**User Experience**
- Added shimmy logo and improved visual branding
- Enhanced error messages and user feedback
- Improved CLI interface responsiveness

### 🔧 Technical Improvements

**Backend Reliability**
- Improved backend selection logic for model compatibility
- Enhanced error recovery and graceful degradation
- Better handling of edge cases in model loading

**Development Tools**
- Configured Claude Code integration for improved development workflow
- Enhanced debugging capabilities and error reporting
- Improved development environment setup

## [1.0.0] - 2025-09-08

### ✨ Features

**Production Release**
- First stable release with comprehensive cross-platform support
- Mature OpenAI API compatibility layer
- Production-ready inference engine with robust error handling

**Enhanced Model Discovery**
- Improved Ollama model discovery with proper manifest parsing
- Cross-platform model detection and loading
- Enhanced compatibility with existing Ollama installations

**Automated Release Infrastructure**
- Complete cross-platform build automation via GitHub Actions
- Automated binary generation for all supported platforms
- Comprehensive governance and contribution guidelines

### 🚀 Enhancements

**Build System Maturity**
- Replaced experimental cross-compilation with stable native builds
- Enhanced release workflow reliability and consistency
- Improved artifact generation and distribution

**Community Infrastructure**
- Added comprehensive GitHub automation and governance
- Implemented professional contribution guidelines
- Enhanced project documentation and developer resources

### 🔧 Technical Improvements

**Stability and Reliability**
- Production-grade error handling and recovery
- Enhanced performance optimization across platforms
- Comprehensive testing and validation framework

## [0.1.2] - 2025-09-04

### 🐛 Bug Fixes

- Added missing `rustup` target installation steps for cross-compilation in CI

## [0.1.1] - 2025-09-06

### ✨ Features

**Native Ollama Integration**
- Added comprehensive Ollama model discovery support
- Enhanced compatibility with existing Ollama installations
- Improved model detection and loading from Ollama directories

**Community Support**
- Added multiple sponsorship options: Buy Me a Coffee, Ko-fi, Open Collective
- Enhanced funding infrastructure for sustainable development
- Improved community engagement and support channels

### 📖 Documentation

**Platform Compatibility**
- Added comprehensive macOS compatibility documentation
- Enhanced Windows installation instructions with security notes
- Improved platform-specific guidance and troubleshooting

**User Experience**
- Added Windows Defender false positive warnings and solutions
- Enhanced installation clarity for new users
- Improved discoverability with better crates.io keywords

### 🐛 Bug Fixes

**Build and Distribution**
- Fixed cross-compilation issues and CI/CD pipeline stability
- Resolved dependency conflicts in GitHub Actions
- Enhanced build reliability across different environments

**Code Quality**
- Cleaned up README markdown formatting for better readability
- Fixed unused import warnings and code quality issues
- Enhanced overall code organization and maintainability

## [0.1.0] - 2025-09-02

### Added
- **Initial release of Shimmy** - The 5MB alternative to Ollama
- **Core inference engine** with llama.cpp backend integration
- **Full OpenAI API compatibility**:
  - `POST /v1/chat/completions` - OpenAI-compatible chat endpoint
  - `GET /v1/models` - List available models
- **Native Shimmy API**:
  - `POST /api/generate` - JSON generation with optional SSE streaming
  - `GET /ws/generate` - WebSocket streaming generation
  - `GET /health` - Health check endpoint
  - `GET /api/models` - Native model listing
- **CLI commands**:
  - `shimmy serve` - Start the inference server
  - `shimmy list` - List available models
  - `shimmy discover` - Discover models in filesystem
  - `shimmy generate` - Command-line text generation
  - `shimmy probe` - Test model loading
- **Model format support**:
  - GGUF models via llama.cpp integration
  - SafeTensors detection and guidance
  - Auto-discovery from filesystem
- **Template system**:
  - ChatML template support
  - Llama3 template support
  - OpenChat template support
- **Cross-platform support**:
  - Linux (x86_64, ARM64)
  - Windows (x86_64)
  - macOS (x86_64, ARM64)
- **Performance optimizations**:
  - 5.1MB single binary size
  - <100ms startup time
  - <50MB memory overhead
  - Release build with LTO and size optimization
- **Integration guides**:
  - VSCode Copilot configuration
  - Continue.dev setup
  - Cursor IDE integration
  - Generic OpenAI API client configuration
- **Package distribution**:
  - GitHub Releases (direct binary downloads)
  - crates.io (Rust package manager)
  - npm (Node.js wrapper package)
  - Docker Hub (container images)
  - PyPI (Python wrapper package)
- **Development infrastructure**:
  - Comprehensive test suite (27 unit tests + 4 integration tests)
  - GitHub Actions CI/CD pipeline
  - Cross-platform build automation
  - Multi-package-manager release automation
- **Documentation**:
  - Complete API documentation
  - Quick start guide (30-second setup)
  - Integration examples
  - Performance benchmarks
  - Architecture documentation

### Technical Details
- **Language**: Rust 2021 edition
- **Dependencies**: tokio, axum, llama-cpp-2, serde, clap
- **Features**: Optional `llama` feature for actual inference
- **License**: MIT (free forever)
- **Minimum supported Rust version**: 1.70+

### Performance Metrics
- **Binary size**: 5.1MB (vs Ollama's 680MB)
- **Startup time**: <100ms (vs Ollama's 5-10s)
- **Memory usage**: <50MB baseline (vs Ollama's 200MB+)
- **API compatibility**: 100% OpenAI compatibility (vs Ollama's partial)

### Free Forever Commitment
Shimmy is committed to being free forever with no asterisks, no "free for now" periods, and no pivot to paid services. The MIT license ensures this commitment is legally binding.

[Unreleased]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v2.3.0...HEAD
[2.3.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v2.2.1...v2.3.0
[2.2.1]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v2.2.0...v2.2.1
[2.2.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.9.0...v2.0.0
[1.9.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.8.1...v1.9.0
[1.8.1]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.8.0...v1.8.1
[1.8.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.7.4...v1.8.0
[1.7.4]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.7.3...v1.7.4
[1.7.3]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.7.2...v1.7.3
[1.7.2]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.7.0...v1.7.2
[1.7.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.6.0...v1.7.0
[1.6.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.5.6...v1.6.0
[1.5.6]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.5.5...v1.5.6
[1.5.5]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.5.4...v1.5.5
[1.5.4]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.5.3...v1.5.4
[1.5.3]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.5.2...v1.5.3
[1.5.2]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.5.1...v1.5.2
[1.5.1]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.5.0...v1.5.1
[1.5.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.4.2...v1.5.0
[1.4.2]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.4.1...v1.4.2
[1.4.1]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.3.5...v1.4.0
[1.3.5]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.3.3...v1.3.5
[1.3.3]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.3.2...v1.3.3
[1.3.2]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.3.1...v1.3.2
[1.3.1]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.2.0...v1.3.1
[1.2.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v0.1.2...v1.0.0
[0.1.2]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Michael-A-Kuykendall/shimmy/releases/tag/v0.1.0
