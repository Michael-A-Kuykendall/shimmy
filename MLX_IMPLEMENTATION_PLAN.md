# MLX Implementation Plan for Shimmy

Perfect—let's build it for real. Below is a **tight, shippable plan** to add an MLX backend to Shimmy on Apple Silicon, with references and a minimal code shape (no wall-of-code). I'm assuming your current engine trait looks like `load() → ModelHandle` + `generate()`; if different, I'll adapt.

## What "MLX support" means (scope)

* **Backend:** Native Apple **MLX** (not llama.cpp/Metal).
* **Models:** MLX-ready **`.npz` weights** (or converted from HF via MLX tooling). ([Hugging Face][1])
* **Bindings:** Use **MLX C API** via Rust bindings (**mlx-rs**), with an escape hatch to FFI if we outgrow the crate. ([GitHub][2])

---

## Fastest viable path (Stage 1: "works on my M-series Mac")

**Goal:** Inference for a small LLM in `.npz` on macOS/ARM64 via `--backend mlx`.

### 1) Dependencies & feature gate

* **Cargo:**

  ```toml
  [features]
  mlx = ["mlx-rs"]

  [target.'cfg(all(target_os = "macos", target_arch = "aarch64"))'.dependencies]
  mlx-rs = { version = "0.21", optional = true }   # crate provides Rust bindings to MLX
  ```

  If `mlx-rs` exposes too little, keep a parallel `mlx-sys` module using `bindgen` to MLX-C. ([Crates.io][3])

* **Build tools:** Xcode CLT + CMake (MLX uses them under the hood). ([GitHub][4])

### 2) Backend wire-up

* **`src/engine/mlx.rs`** (new): implement your engine trait with:

  * **Model load:** open `.npz`, create MLX arrays/tensors, build graph.
  * **Generate:** greedy (or top-p) loop using MLX ops.
* **Backend selector:** prefer MLX when `--backend mlx` AND `cfg(macos, aarch64)`.

> MLX has official **C / C++ / Swift** APIs mirroring Python; we access via `mlx-rs` (Rust wrapper over MLX C/C++). ([GitHub][4])

### 3) Tokenizer

* Reuse your existing **SentencePiece/tokenizers** path (as with GGUF), just feed **ids→embeddings** into MLX graph.
* If you want dead-simple bring-up, start with a **toy model** (tiny GPT2-style NPZ) before a 7B. MLX community provides NPZ Llama variants. ([Hugging Face][1])

### 4) Minimal "shape" of the code ( just the joints )

```rust
// src/engine/mlx.rs
#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "mlx"))]
pub struct MlxEngine {
    model: MlxGraph,          // opaque handle you define
    tok: Arc<dyn Tokenizer>,  // reuse your tokenizer abstraction
}

impl Engine for MlxEngine {
    fn load(cfg: &ModelConfig) -> Result<Self> {
        // 1) load NPZ → MLX arrays
        // 2) build attention/MLP blocks with mlx-rs nn ops
        // 3) stash graph + weights
    }

    fn generate(&mut self, prompt: &str, params: &GenParams, cb: impl Fn(&str)) -> Result<()> {
        // 1) tokenize
        // 2) autoregressive loop: run forward() with kv-cache
        // 3) sample + detokenize stream via cb
    }
}
```

(Where `MlxGraph` is your thin wrapper around `mlx_rs` modules/ops and a KV-cache buffer.)

### 5) Where the weights come from

* Use **mlx-lm** helpers (Python) or community repos to **convert HF → `.npz`** and quantize if desired; the MLX ecosystem commonly ships NPZ Llama/Mistral variants. ([GitHub][5])

### 6) Smoke test

* Model: `mlx-community/Llama-2-7b-mlx` (npz) or a smaller one for 1st run.
* Command: `shimmy serve --model path/to/weights.npz --backend mlx` → expect tokens/sec on M-series GPU. ([Hugging Face][1])

---

## Stage 2 (stability & UX)

* **Sampling parity** with llama.cpp flags (temperature, top-p, repeat penalty).
* **Quantization:** use MLX-LM conversion/quant scripts to ship a doc'd path for users. ([GitHub][5])
* **Metrics:** surface tokens/sec and GPU util (if available via MLX introspection).

---

## Stage 3 (CI + release discipline)

* **CI:** GitHub Actions macOS-14 (M3) runner, matrix on `release` / `debug`.
  Install Xcode CLT, cache MLX artifacts; run a 10-token generation smoke test.
* **Feature flag docs:** README table notes **"MLX (macOS/Apple Silicon, NPZ)"** with known-good models + conversion commands.

---

### Why start with `mlx-rs`?

* It's the **lowest-effort** path to first tokens; official MLX C/C++ exists if you need to drop to FFI for hot paths.
  (Crate and project evidence: crates.io, docs site, maintainer posts.) ([Crates.io][3])

---

## Concrete task list (PR-ready)

1. **Scaffold**

   * `engine/mlx.rs`, `cfg(macos,aarch64)` + `feature="mlx"`.
   * Add `mlx-rs` dep and a `--backend mlx` enum variant.

2. **Weights loader**

   * Minimal NPZ reader → MLX arrays (use `ndarray-npz` or call into an MLX helper if exposed).
   * Build **single block** → print logits to verify numerics.

3. **Forward pass**

   * Implement attention (qkv proj, softmax, rope if needed), MLP, residuals, layernorm—via mlx-rs ops.

4. **Generate loop**

   * Greedy decode first (no sampling); then add temp/top-p.

5. **Tokenizer bridge**

   * Reuse your tokenizer; ensure `bos/eos` handling matches model config.

6. **CLI + README**

   * `--backend mlx` docs + a short "Convert to NPZ (MLX-LM)" section with links. ([GitHub][5])

7. **CI**

   * macOS ARM64 smoke test (10 tokens) on tiny NPZ to keep runtime < 60s.

---

## Risk / Assumption Audit (critique-first)

* **Model format variance:** Not all HF models have 1-click NPZ; users may need MLX-LM conversion. (Document exact commands.) ([GitHub][5])
* **API flux:** `mlx-rs` is active; minor breaking changes possible. Pin versions, add a fallback `mlx-sys` FFI shim. ([Crates.io][3])
* **Tokenizer mismatch:** EOS/BOS or added tokens can produce garbage logits; test per model card. ([GitHub][5])
* **Perf expectations:** MLX vs llama.cpp/Metal differs by model/quant; don't promise faster—**promise "native MLX option."** Comparative posts exist but vary. ([Medium][6])

### Action Items for Verification

* **Numerical spot-check:** 1 block forward pass vs MLX-LM Python logits on same prompt/model. ([GitHub][5])
* **Tiny integration test:** Deterministic prompt → deterministic greedy tokens.
* **CI proof:** macOS ARM64 runner logs show "backend=mlx" and non-zero tokens/sec.

### Flagged Claims (treat as TODO until verified)

* "1–2 day implementation" depends on model graph parity and NPZ layout—treat as an estimate, not a promise.
* "Auto-detect Apple Silicon → prefer MLX" should remain **opt-in** until parity is proven.

---

## IMPROVEMENTS & ADDITIONS

### 🔧 Integration with Shimmy's Architecture

Based on current Shimmy structure, here are specific adaptations:

```rust
// Integrate with existing engine trait in src/engine/mod.rs
#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "mlx"))]
impl InferenceEngine for MLXEngine {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        // Check for .npz extension or MLX-compatible models
        if !spec.base_path.extension().map_or(false, |ext| ext == "npz") {
            return Err(anyhow!("MLX engine requires .npz model format"));
        }
        
        let model = MLXLoadedModel::new(spec).await?;
        Ok(Box::new(model))
    }
}
```

### 🧠 Model Discovery Integration

Extend existing discovery system to find MLX models:

```rust
// In src/discovery.rs, add MLX model detection
fn is_mlx_model(&self, path: &Path) -> bool {
    // Check for .npz files or MLX-converted models
    path.extension().and_then(|s| s.to_str()) == Some("npz") ||
    path.to_string_lossy().contains("mlx-community")
}
```

### 🚀 Performance Monitoring

```rust
// Add MLX-specific metrics to match existing GPU monitoring
impl MLXLoadedModel {
    fn get_metal_memory_usage(&self) -> Option<u64> {
        // Query MLX for Metal memory usage if available
        // mlx_rs may expose device memory info
    }
}
```

### 📦 Cargo.toml Enhancements

```toml
# More specific feature combinations
[features]
mlx = ["mlx-rs", "ndarray-npz"]
gpu = ["llama-cuda", "llama-vulkan", "llama-opencl", "mlx"]
apple-optimized = ["mlx", "accelerate"]

# Add NPZ support
[dependencies]
ndarray-npz = { version = "0.8", optional = true }
```

### 🔍 Better Error Messages

```rust
// Shimmy-specific error context
pub enum MLXError {
    ModelNotFound { path: PathBuf },
    UnsupportedFormat { expected: &'static str, got: String },
    MetalNotAvailable,
    TokenizerMismatch { model_vocab: usize, tokenizer_vocab: usize },
}
```

### 🧪 Testing Strategy

```rust
#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
mod mlx_tests {
    #[test]
    fn test_mlx_detection() {
        let engine = MLXEngine::new();
        assert!(engine.is_available());
    }
    
    #[tokio::test]
    async fn test_tiny_model_generation() {
        // Use a minimal test model for CI
        let spec = ModelSpec::from_path("tests/fixtures/tiny-mlx.npz");
        // ... test basic generation
    }
}
```

### 📊 CLI Integration

```bash
# Extend existing CLI with MLX options
shimmy serve --backend mlx --model path/to/model.npz
shimmy gpu-info  # Should show MLX backend status
shimmy discover --format npz  # Find .npz models specifically
```

### 🔧 Fallback Strategy

```rust
// If mlx-rs is insufficient, prepare FFI escape hatch
#[cfg(feature = "mlx-sys")]
mod mlx_sys {
    use std::ffi::c_void;
    
    extern "C" {
        fn mlx_array_new() -> *mut c_void;
        fn mlx_forward_pass(model: *mut c_void, input: *mut c_void) -> *mut c_void;
    }
}
```

---

## ENHANCED TASK BREAKDOWN

### Week 1: Foundation
1. **Day 1**: Scaffold + dependency setup + basic MLX detection
2. **Day 2**: NPZ loading + single forward pass verification  
3. **Day 3**: Basic generation loop (greedy decoding)

### Week 2: Integration  
4. **Day 4**: Tokenizer integration + EOS/BOS handling
5. **Day 5**: Sampling implementation (temperature, top-p)
6. **Day 6**: CLI integration + error handling

### Week 3: Polish
7. **Day 7**: Performance monitoring + metrics
8. **Day 8**: CI setup + smoke tests
9. **Day 9**: Documentation + user guides

### Validation Checkpoints
- **Checkpoint 1**: Can load .npz model without crashing
- **Checkpoint 2**: Generates deterministic tokens for fixed prompt
- **Checkpoint 3**: Matches MLX-LM Python output for same model/prompt
- **Checkpoint 4**: CI passes on GitHub macOS ARM64 runner

---

## Handy references (for your PR description)

* **MLX (core) repo & API overview.** ([GitHub][4])
* **MLX-C (official C API) + docs**—for FFI fallback. ([GitHub][2])
* **mlx-rs crate page & docs.** ([Crates.io][3])
* **MLX-LM (conversion, quantization, HF Hub).** ([GitHub][5])
* **NPZ Llama community weights (example).** ([Hugging Face][1])

---

**READY TO PROCEED**: This plan provides multiple escape hatches, clear validation points, and integrates with Shimmy's existing architecture. The risk is well-managed with fallback options.

[1]: https://huggingface.co/mlx-community/Llama-2-7b-mlx
[2]: https://github.com/ml-explore/mlx-c
[3]: https://crates.io/crates/mlx-rs
[4]: https://github.com/ml-explore/mlx
[5]: https://github.com/ml-explore/mlx-lm
[6]: https://medium.com/@zaiinn440/apple-mlx-vs-llama-cpp-vs-hugging-face-candle-rust-for-lightning-fast-llms-locally-5447f6e9255a