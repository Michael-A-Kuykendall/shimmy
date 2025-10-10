# GPU Architecture Decision Request for Shimmy Issue #72

## Prompt for GPT-5

You are an expert systems architect reviewing a critical GPU acceleration issue in a Rust-based LLM inference engine. The current implementation is fundamentally broken and needs architectural redesign. Please analyze the provided data and recommend the optimal solution.

**Context**: Shimmy is a 5MB Ollama alternative that serves GGUF models via llama.cpp. A user reported that GPU backends (Vulkan/OpenCL) are detected but no layers are offloaded to GPU, causing everything to run on CPU.

**Current Problem**: Our attempted fix is architecturally flawed - we added CLI options that store values but the GPU detection logic uses compile-time features that don't exist, so GPU detection always falls through to CPU.

**Your Task**: Analyze the three architectural options below and recommend the best approach considering performance, maintainability, reliability, and implementation complexity.

---

## Current Broken Implementation

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

### Current Broken Code

**Cargo.toml Features (THE PROBLEM)**:
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
