# GPU Architecture Decision Request for Shimmy Issue #72

**✅ RESOLVED in v1.9.0** - Kitchen Sink Architecture Implemented

## Final Solution: Kitchen Sink Architecture

Issue #72 and 22+ related build/GPU issues were comprehensively solved by implementing a "Kitchen Sink" distribution model: **all GPU backends compiled into each platform binary** with **runtime auto-detection**.

### What Changed in v1.9.0

**Before (v1.8.x)**:
- 9 different binaries: cpu, cuda, vulkan, opencl variants per platform
- Users had to choose GPU backend at download time
- Compile-time features blocked runtime detection
- User confusion: "Which one do I need?"

**After (v1.9.0)**:
- 5 platform binaries with ALL backends included
- Runtime auto-detection using existing logic (lines 127-156 of src/engine/llama.rs)
- Zero user choice required at download time
- "Download → Run → GPU detected automatically"

### Binary Distribution

| Platform | Binary | Includes | Size |
|----------|--------|----------|------|
| Windows x64 | shimmy-windows-x86_64.exe | CUDA + Vulkan + OpenCL + Vision | ~45MB |
| Linux x86_64 | shimmy-linux-x86_64 | CUDA + Vulkan + OpenCL + Vision | ~45MB |
| macOS ARM64 | shimmy-macos-arm64 | MLX + Vision | ~35MB |
| macOS Intel | shimmy-macos-intel | CPU-only + Vision | ~20MB |
| Linux ARM64 | shimmy-linux-aarch64 | CPU-only + Vision | ~20MB |

### GPU Detection Priority Order

Already implemented in `src/engine/llama.rs` lines 127-156:

```rust
fn detect_best_gpu_backend() -> GpuBackend {
    // Priority order: CUDA → Vulkan → OpenCL → CPU
    if Self::is_cuda_available() {
        return GpuBackend::Cuda;
    }
    
    if Self::is_vulkan_available() {
        return GpuBackend::Vulkan;
    }
    
    if Self::is_opencl_available() {
        return GpuBackend::OpenCL;
    }
    
    info!("No GPU acceleration available, using CPU backend");
    GpuBackend::Cpu
}
```

### Issues Resolved

**Direct GPU Issues**:
- #72 - GPU detected but layers assigned to CPU
- #130 - GPU not enabled with --backend vulkan flag
- #142 - AMD GPU not detected (Vulkan/OpenCL)

**Build/Compilation Issues** (eliminated by pre-built binaries):
- #129 - GPU support not available in precompiled
- #144 - MLX should be default on Apple Silicon
- #110 - Build failure on cargo install v1.7.0
- #105 - Windows GPU build errors
- #99 - cargo install shimmy fail
- #86 - Missing template files
- #88 - Unable to compile on macOS M2
- Plus 13+ more compilation-related issues

---

## Historical Context: Original Problem Analysis

*The content below documents the original architectural analysis that led to the v1.9.0 Kitchen Sink solution. Preserved for historical reference.*

### User's Original Issue (Issue #72)
```
Commands Ran:
1. cargo build --release --no-default-features --features huggingface,llama-opencl,llama-vulkan
2. ./shimmy.exe serve --gpu-backend auto

Expected: GPU acceleration for model inference
Actual: "CPU is used (verfied with 100% CPU time)"

Logs show:
- "layer X assigned to device CPU"
- "tensor 'token_embd.weight' cannot be used with preferred buffer type CPU_REPACK, using CPU instead"
```

### Root Cause Identified
```toml
[features]
default = ["huggingface"]
llama = ["dep:llama-cpp-2"]
huggingface = []
console = ["dep:shimmy-console-lib", "dep:tokio-tungstenite", "dep:crossterm", "dep:reqwest"]
fast = ["huggingface"]
full = ["huggingface", "llama"]
```

**Note**: Features `llama-opencl`, `llama-vulkan`, `llama-cuda` **DO NOT EXIST** but our code references them.

**Broken GPU Detection Logic**:
```rust
fn detect_best_gpu_backend() -> GpuBackend {
    #[cfg(feature = "llama-cuda")]      // ❌ Feature doesn't exist
    {
        if Self::is_cuda_available() {
            return GpuBackend::Cuda;
        }
    }

    #[cfg(feature = "llama-vulkan")]    // ❌ Feature doesn't exist
    {
        if Self::is_vulkan_available() {
            return GpuBackend::Vulkan;
        }
    }

    #[cfg(feature = "llama-opencl")]    // ❌ Feature doesn't exist
    {
        if Self::is_opencl_available() {
            return GpuBackend::OpenCL;
        }
    }

    info!("No GPU acceleration available, using CPU backend");
    GpuBackend::Cpu  // ❌ ALWAYS returns this
}
```

**Current Model Loading** (conceptually correct but never reached):
```rust
async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
    let gpu_layers = self.determine_gpu_layers(spec);  // Gets value but detection is broken

    let model_params = llama::model::params::LlamaModelParams::default()
        .with_n_gpu_layers(gpu_layers);  // ✅ Correct API call

    let model = llama::model::LlamaModel::load_from_file(&be, &spec.base_path, &model_params)?;
}
```

### Performance Impact
- **Current**: 100% CPU usage, no GPU acceleration despite having Vulkan/OpenCL
- **Expected**: GPU layers should offload computation, reducing CPU usage significantly

---

## Option A: Runtime GPU Detection (Recommended by Claude)

### Implementation
```rust
impl LlamaEngine {
    fn detect_best_gpu_backend() -> GpuBackend {
        // Runtime detection - no compile-time features
        if Self::is_cuda_available() {
            info!("CUDA GPU detected, using CUDA backend");
            return GpuBackend::Cuda;
        }

        if Self::is_vulkan_available() {
            info!("Vulkan GPU detected, using Vulkan backend");
            return GpuBackend::Vulkan;
        }

        if Self::is_opencl_available() {
            info!("OpenCL GPU detected, using OpenCL backend");
            return GpuBackend::OpenCL;
        }

        info!("No GPU acceleration available, using CPU backend");
        GpuBackend::Cpu
    }

    fn is_vulkan_available() -> bool {
        // Actual Vulkan loader detection
        std::process::Command::new("vulkaninfo")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_opencl_available() -> bool {
        // Probe for OpenCL runtime
        std::process::Command::new("clinfo")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}
```

### CLI Integration
```rust
// CLI options work as intended
./shimmy serve --gpu-backend vulkan --gpu-layers 32
./shimmy serve --gpu-backend auto --gpu-layers -1  // Auto-detect layers
```

### Pros
- ✅ **Works immediately** - no Cargo.toml changes needed
- ✅ **Runtime flexibility** - works on any system with GPU drivers
- ✅ **Simple implementation** - remove feature gates, add runtime checks
- ✅ **User control** - CLI can override auto-detection
- ✅ **Robust** - fails gracefully when GPU not available

### Cons
- ❌ **External dependencies** - relies on `vulkaninfo`/`clinfo` being installed
- ❌ **Runtime overhead** - process spawning for detection (one-time cost)
- ❌ **Platform-specific** - detection commands vary by OS

### Implementation Effort
- **Low**: Remove `#[cfg(feature = "...")]`, add runtime detection
- **Testing**: Easy to test on different systems
- **Compatibility**: Works with existing llama.cpp integration

---

## Option B: Engine-Level Configuration

### Implementation
```rust
pub struct LlamaEngine {
    gpu_backend: GpuBackend,
    gpu_layers: i32,
}

impl LlamaEngine {
    pub fn new_with_gpu_config(backend: GpuBackend, layers: i32) -> Self {
        Self {
            gpu_backend: backend,
            gpu_layers: layers,
        }
    }

    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        // Use self.gpu_layers directly, no spec.gpu_layers needed
        let model_params = llama::model::params::LlamaModelParams::default()
            .with_n_gpu_layers(self.gpu_layers);
    }
}

// In main.rs
let engine = LlamaEngine::new_with_gpu_config(
    parse_gpu_backend(&cli.gpu_backend),
    cli.gpu_layers.unwrap_or(-1)
);
```

### ModelSpec Changes
```rust
// Clean ModelSpec - no GPU concerns
pub struct ModelSpec {
    pub name: String,
    pub base_path: PathBuf,
    pub lora_path: Option<PathBuf>,
    pub template: Option<String>,
    pub ctx_len: usize,
    pub n_threads: Option<i32>,
    // gpu_layers: REMOVED
    // gpu_backend: REMOVED
}
```

### Pros
- ✅ **Clean separation** - GPU config separate from model specs
- ✅ **Single configuration point** - engine configured once at startup
- ✅ **Simpler ModelSpec** - models don't carry GPU baggage
- ✅ **Performance** - no per-model GPU configuration overhead

### Cons
- ❌ **Less flexibility** - can't have different GPU settings per model
- ❌ **Architectural change** - requires refactoring how engines are created
- ❌ **Breaking change** - affects existing ModelSpec usage throughout codebase

### Implementation Effort
- **Medium**: Refactor engine creation, remove GPU from ModelSpec, update all callers
- **Testing**: Need to verify all ModelSpec usages still work
- **Risk**: More invasive changes to core architecture

---

## Option C: Fix Feature Architecture

### Implementation
**Add Missing Features to Cargo.toml**:
```toml
[features]
default = ["huggingface"]
llama = ["dep:llama-cpp-2"]
llama-cuda = ["llama", "llama-cpp-2/cuda"]
llama-vulkan = ["llama", "llama-cpp-2/vulkan"]
llama-opencl = ["llama", "llama-cpp-2/opencl"]
huggingface = []
gpu = ["llama-cuda", "llama-vulkan", "llama-opencl"]  # Convenience
```

**Build Commands**:
```bash
# GPU-enabled builds
cargo build --features llama-vulkan
cargo build --features gpu  # All GPU backends
cargo build --features llama,llama-vulkan,llama-opencl

# Current approach would work
cargo build --features huggingface,llama-opencl,llama-vulkan
```

### Pros
- ✅ **Compile-time optimization** - only include GPU code when needed
- ✅ **Smaller binaries** - exclude unused GPU backends
- ✅ **Clear dependencies** - explicit about what's included
- ✅ **Current code works** - minimal changes to existing logic

### Cons
- ❌ **Complex build matrix** - many feature combinations
- ❌ **User confusion** - users must know which features to enable
- ❌ **Distribution complexity** - need multiple binary variants
- ❌ **llama-cpp-2 dependency** - assumes these features exist in the crate

### Implementation Effort
- **High**: Verify llama-cpp-2 supports these features, test all combinations
- **Risk**: May not be possible if llama-cpp-2 doesn't expose granular features
- **Distribution**: Need to build multiple binary variants for releases

---

## Critical Technical Data

### Current llama-cpp-2 Integration
```rust
// How we currently load models
let model = llama::model::LlamaModel::load_from_file(
    &be,
    &spec.base_path,
    &model_params,  // This is where GPU layers are configured
)?;

// The with_n_gpu_layers API exists and works
let model_params = llama::model::params::LlamaModelParams::default()
    .with_n_gpu_layers(32);  // ✅ This API call is correct
```

### llama-cpp-2 Crate Features (needs verification)
```bash
# Need to check what features llama-cpp-2 actually exposes
cargo search llama-cpp-2 --features
```

### User's Build Environment
- Windows system with GPU capabilities
- Used: `cargo build --release --no-default-features --features huggingface,llama-opencl,llama-vulkan`
- **Problem**: These features don't exist, so build succeeded but with no GPU code

### Performance Requirements
- **Startup time**: <100ms (constitutional requirement)
- **Binary size**: <5MB (constitutional limit)
- **Memory usage**: Minimal overhead for GPU detection
- **Reliability**: Must fail gracefully when GPU unavailable

### Release Gate Implications
```yaml
# Current release gates that must pass
- Core Build Validation
- CUDA Build Timeout Detection (<3min)
- Binary Size Limit (5MB)
- Test Suite Validation
```

Any solution must not break existing release gates or constitutional requirements.

---

## Decision Framework

Please evaluate each option against these criteria:

1. **Implementation Complexity**: How much code needs to change?
2. **Performance Impact**: Runtime costs vs compile-time optimization
3. **User Experience**: Build complexity vs runtime flexibility
4. **Maintainability**: Long-term code clarity and debugging
5. **Reliability**: Failure modes and graceful degradation
6. **Constitutional Compliance**: Binary size, startup time, release gates

## Request

**Provide a detailed recommendation with**:
1. **Primary choice** and reasoning
2. **Implementation roadmap** with specific steps
3. **Risk assessment** and mitigation strategies
4. **Testing strategy** to prevent regressions
5. **Migration path** from current broken state

Consider that this is a production system with users depending on GPU acceleration, and the fix must be robust enough to ship to end users.
