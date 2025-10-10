# Shimmy MoE CPU Offloading: Technical Validation & User Guide
**Production Integration of llama.cpp MoE Expert Tensor Offloading in Rust**

*Version 1.0 - October 8, 2025*

---

## What This Document Is

This is a **technical validation** of MoE CPU offloading in Shimmy, demonstrating:
- How to use `--cpu-moe` and `--n-cpu-moe` flags in production
- Measured VRAM/RAM usage on real hardware (NVIDIA GH200)
- Performance characteristics across three model families
- Reproduction instructions with exact commits, commands, and checksums

**This is NOT a research novelty claim.** llama.cpp added native MoE offloading on August 4, 2025 (PR #15077 by @slaren). Our contribution is **Rust bindings** (`llama-cpp-2` crate) and **production integration** in Shimmy with comprehensive testing.

---

## Executive Summary

### What We Built
- **Rust bindings** for llama.cpp's MoE CPU offloading (methods: `with_cpu_moe_all()`, `with_n_cpu_moe(n)`)
- **CLI integration** in Shimmy: `--cpu-moe` and `--n-cpu-moe N` flags
- **Validation** across three MoE model families (20B-42B parameters)

### Measured Results (NVIDIA GH200 480GB)
- **GPT-OSS 20B**: ~1.8-2.3GB VRAM with `--cpu-moe` vs ~15GB estimated baseline
<<<<<<< HEAD
<<<<<<< HEAD
- **Phi-3.5-MoE 42B**: ~2.8GB VRAM with `--cpu-moe` vs ~80GB estimated baseline
=======
- **Phi-3.5-MoE 42B**: ~2.8GB VRAM with `--cpu-moe` vs ~80GB estimated baseline  
>>>>>>> main
=======
- **Phi-3.5-MoE 42B**: ~2.8GB VRAM with `--cpu-moe` vs ~80GB estimated baseline  
>>>>>>> main
- **DeepSeek 16B**: Full CPU offloading confirmed via tensor buffer logs

### Known Limitations
- **No controlled baselines**: Baseline numbers are estimates from model size, not measured A/B comparisons
- **Token counting inaccurate**: Current measurements use word_count × 1.3 (non-streaming) or SSE chunk counting (streaming)
- **TTFT estimated**: First token latency derived from 10% heuristic, not per-token timestamps
- **Single-run measurements**: No statistical variance (N=1 for all tests)
- **Historical 2MB claim unreproducible**: Earlier builds showed ~2MB VRAM; current builds measure 1.8-2.3GB

---

## Quick Start

### Basic Usage
```bash
# Offload ALL expert tensors to CPU
shimmy serve --bind 127.0.0.1:11435 --cpu-moe

# Offload first 10 layers' experts to CPU (fine-grained control)
shimmy serve --bind 127.0.0.1:11435 --n-cpu-moe 10
```

### When to Use This
- **Large MoE models** that don't fit in VRAM (Phi-3.5-MoE, GPT-OSS, DeepSeek)
- **High RAM, limited VRAM** setups (e.g., 256GB system RAM, 24GB GPU)
- **Batch processing** where throughput > latency (expect ~10% TTFT overhead)

### Instance Sizing Guide
| Model | VRAM (offload) | RAM (offload) | Recommended Instance |
|-------|----------------|---------------|----------------------|
| GPT-OSS 20B | ~2-3GB | ~13GB | 24GB GPU + 32GB RAM |
| Phi-3.5-MoE 42B | ~3-4GB | ~80GB | 24GB GPU + 128GB RAM |
| DeepSeek 16B | ~2-3GB | ~31GB | 24GB GPU + 64GB RAM |

---

## How It Works (At a Glance)

### Tensor Placement Strategy
```
┌─────────────────────────────────────┐
│ GPU (CUDA0)                         │
│ ✓ Attention layers                  │
│ ✓ Embeddings                        │
│ ✓ Normalization                     │
│ ✓ Output projection                 │
└─────────────────────────────────────┘
          ↕ PCIe transfers
┌─────────────────────────────────────┐
│ CPU (CUDA_Host pinned memory)       │
│ ✓ Expert tensors (ffn_*_exps)       │
│   - ffn_gate_exps.weight            │
│   - ffn_down_exps.weight            │
│   - ffn_up_exps.weight              │
└─────────────────────────────────────┘
```

### Rust Implementation
```rust
// In llama-cpp-2/src/model/params.rs
pub fn with_cpu_moe_all(mut self) -> Self {
    self.push_tensor_override(r"\.ffn_(up|down|gate)_exps");
    self
}

pub fn with_n_cpu_moe(mut self, n: usize) -> Self {
    for i in 0..n {
        let pattern = format!(r"blk\.{}\.ffn_(up|down|gate)_exps", i);
        self.push_tensor_override(&pattern);
    }
    self
}
```

**Technical Details**:
- Uses llama.cpp's `tensor_buft_overrides` mechanism (added PR #15077)
- Patterns matched via regex against GGUF tensor names
- Matched tensors allocated using `ggml_backend_cpu_buffer_type()` (pinned host memory)
- NULL-terminated array lifetime managed in Rust wrapper

---

## Validated Results

### Test Environment
- **Hardware**: NVIDIA GH200 480GB (97,871 MiB VRAM available)
- **Driver**: 570.148.08, CUDA 12.8
- **Shimmy**: Commit `cb75f5a` (feat/moe-cpu-offload branch)
- **llama-cpp-rs**: Commit `6c9a704` (llama.cpp submodule at b6686)
- **Date**: October 6-8, 2025
- **Location**: Lambda Cloud

### Model 1: GPT-OSS 20B

<<<<<<< HEAD
<<<<<<< HEAD
**Architecture**: 32 experts per layer, 4 active per token, 24 layers
**File**: `gpt-oss-20b-f16.gguf` (13.8GB)
**Source**: https://huggingface.co/tensorblock/GPT-OSS-20B-GGUF
=======
**Architecture**: 32 experts per layer, 4 active per token, 24 layers  
**File**: `gpt-oss-20b-f16.gguf` (13.8GB)  
**Source**: https://huggingface.co/tensorblock/GPT-OSS-20B-GGUF  
>>>>>>> main
=======
**Architecture**: 32 experts per layer, 4 active per token, 24 layers  
**File**: `gpt-oss-20b-f16.gguf` (13.8GB)  
**Source**: https://huggingface.co/tensorblock/GPT-OSS-20B-GGUF  
>>>>>>> main
**SHA256**: *(not recorded - add in reproduction)*

#### Memory Usage (Measured)
```
Configuration          GPU VRAM    CPU RAM     Method
────────────────────────────────────────────────────────
Baseline (estimated)   ~15GB       ~1GB        Model size heuristic
With --cpu-moe         2.33GB      13.09GB     llama.cpp allocator logs
With --cpu-moe (live)  ~1.8GB      ~13GB       nvidia-smi process view
```

**Evidence** (from llama.cpp logs):
```
load_tensors: CPU_Mapped model buffer size = 13090.25 MiB
load_tensors: CUDA0 model buffer size = 2329.33 MiB

tensor blk.0.ffn_gate_exps.weight (134 MiB) buffer type overridden to CUDA_Host
tensor blk.0.ffn_down_exps.weight (134 MiB) buffer type overridden to CUDA_Host
tensor blk.0.ffn_up_exps.weight (134 MiB) buffer type overridden to CUDA_Host
[... 23 more layers with same pattern ...]
```

**VRAM Reduction**: ~84-88% (based on 2.3GB measured vs 15GB estimated)

#### Performance (Single-Run, Streaming Mode)
```
Test Prompt             Tokens  TTFT (ms)   TPS     Notes
──────────────────────────────────────────────────────────────
Short (7 tok)           100     313         31.93   Estimate via SSE chunk count
Medium (6 tok)          100     336         30.93   Estimate via SSE chunk count
Long (10 tok)           100     328         30.50   Estimate via SSE chunk count
Very Long (27 tok)      100     318         33.36   Estimate via SSE chunk count
Average                 100     324         31.68
```

**Limitations**:
- Token counts are **SSE chunk counts**, not tokenizer-derived
- TTFT is **estimated from total time**, not first-token timestamp
- No baseline comparison (would require running without `--cpu-moe` on same hardware)
- N=1 (no variance measurements)

### Model 2: Phi-3.5-MoE 41.9B

<<<<<<< HEAD
<<<<<<< HEAD
**Architecture**: 16 experts per layer, 2 active per token, 32 layers
**File**: `phi-3.5-moe-f16.gguf` (79GB)
**Source**: Converted from https://huggingface.co/microsoft/Phi-3.5-MoE-instruct
=======
**Architecture**: 16 experts per layer, 2 active per token, 32 layers  
**File**: `phi-3.5-moe-f16.gguf` (79GB)  
**Source**: Converted from https://huggingface.co/microsoft/Phi-3.5-MoE-instruct  
>>>>>>> main
=======
**Architecture**: 16 experts per layer, 2 active per token, 32 layers  
**File**: `phi-3.5-moe-f16.gguf` (79GB)  
**Source**: Converted from https://huggingface.co/microsoft/Phi-3.5-MoE-instruct  
>>>>>>> main
**Conversion Command**: *(see Reproduction section)*

#### Memory Usage (Measured)
```
Configuration          GPU VRAM    CPU RAM     Method
────────────────────────────────────────────────────────
Baseline (estimated)   ~80GB       ~1GB        Model size heuristic
With --cpu-moe         2.8GB       ~76GB       llama.cpp allocator logs
```

**VRAM Reduction**: ~96.5% (based on 2.8GB measured vs 80GB estimated)

#### Performance (Single-Run, Streaming Mode)
```
Test Prompt             Tokens  TTFT (ms)   TPS     Notes
──────────────────────────────────────────────────────────────
Short (7 tok)           100     366         13.94   Estimate via SSE chunk count
Medium (6 tok)          100     706         14.44   Estimate via SSE chunk count
Long (10 tok)           100     688         16.28   Estimate via SSE chunk count
Very Long (27 tok)      100     686         15.45   Estimate via SSE chunk count
Average                 100     612         15.03
```

### Model 3: DeepSeek MoE 16B

<<<<<<< HEAD
<<<<<<< HEAD
**Architecture**: 64 regular experts + 2 shared experts, 6 active per token
**File**: `deepseek-moe-16b-f16.gguf` (30.51GB)
=======
**Architecture**: 64 regular experts + 2 shared experts, 6 active per token  
**File**: `deepseek-moe-16b-f16.gguf` (30.51GB)  
>>>>>>> main
=======
**Architecture**: 64 regular experts + 2 shared experts, 6 active per token  
**File**: `deepseek-moe-16b-f16.gguf` (30.51GB)  
>>>>>>> main
**Source**: https://huggingface.co/MikeKuykendall/deepseek-moe-16b-cpu-offload-gguf

#### Memory Usage (Measured)
```
Configuration          GPU VRAM    CPU RAM     Method
────────────────────────────────────────────────────────
With --cpu-moe         ~2-3GB      ~31GB       llama.cpp allocator logs
```

**Unique Architecture Note**: DeepSeek uses dual-expert system (64 regular + 2 shared). All expert tensors successfully offloaded to CPU.

#### Performance (Single-Run, Streaming Mode)
```
Test Prompt             Tokens  TTFT (ms)   TPS     Notes
──────────────────────────────────────────────────────────────
Short (7 tok)           100     335         30.76   Estimate via SSE chunk count
Medium (6 tok)          100     275         28.74   Estimate via SSE chunk count
Long (10 tok)           100     328         35.32   Estimate via SSE chunk count
Very Long (27 tok)      100     327         32.39   Estimate via SSE chunk count
Average                 100     316         31.80
```

---

## Cross-Model Performance Summary

| Model | Avg TPS (Stream) | Avg TTFT (ms) | VRAM (GB) | Best Use Case |
|-------|------------------|---------------|-----------|---------------|
| GPT-OSS 20B | 31.68 | 324 | 1.8-2.3 | Fastest throughput, batch processing |
| DeepSeek 16B | 31.80 | 316 | 2-3 | Balanced performance |
| Phi-3.5-MoE 42B | 15.03 | 612 | 2.8 | Large context, interactive (slower but works) |

**Performance Characteristics**:
- Smaller models (GPT-OSS, DeepSeek) achieve ~30 TPS despite CPU offloading
- Larger model (Phi-3.5-MoE) shows ~50% throughput reduction but remains usable
- TTFT ranges 275-706ms across all models (acceptable for most use cases)
- Streaming vs non-streaming shows variable results (model-dependent)

---

## Quality Validation & Limitations

### Manual Quality Assessment
Each model tested with 4 prompt types (code, math, creative, technical). All models produced **coherent, contextually appropriate responses**.

**Examples** (GPT-OSS 20B):
```
Prompt: "Write a Python function to calculate fibonacci numbers recursively"
Output: [Valid Python code with proper base cases and recursion]

Prompt: "Explain quantum computing in simple terms"
Output: [Clear explanation with appropriate analogies]
```

### Known Quality Issues
- **October 7, 2025**: GPT-OSS showed repetition artifacts in automated validator
- **Root Cause**: Sampler configuration mismatch (under investigation)
- **Status**: Manual validation (Oct 8) confirms acceptable production quality
- **Action**: Re-evaluate sampler chain vs upstream defaults

### Objective Quality Metrics (Not Yet Implemented)
**Recommended for future validation**:
- Embedding similarity (cosine) between baseline/offload outputs (N=20 prompts)
- Pass@k for code generation (N=10 prompts)
- Edit distance for deterministic prompts (temperature=0.0)

---

## Reproduce Our Numbers

### Environment Setup
```bash
# Clone repositories
git clone https://github.com/Michael-A-Kuykendall/shimmy.git
cd shimmy
git checkout cb75f5a  # feat/moe-cpu-offload

git clone https://github.com/utilityai/llama-cpp-rs.git ../llama-cpp-rs
cd ../llama-cpp-rs
git checkout 6c9a704  # MoE support, llama.cpp b6686

# Build shimmy
cd ../shimmy
cargo build --release --features llama-cuda
```

### Model Conversion (Phi-3.5-MoE Example)
```bash
# Download SafeTensors
git clone https://huggingface.co/microsoft/Phi-3.5-MoE-instruct

# Convert to GGUF
cd llama-cpp-rs/llama-cpp-sys-2/llama.cpp
python convert_hf_to_gguf.py \
  --outfile phi-3.5-moe-f16.gguf \
  --outtype f16 \
  ../../../Phi-3.5-MoE-instruct/

# Verify conversion
ls -lh phi-3.5-moe-f16.gguf  # Should be ~79GB
```

**Expected Output**:
```
Expert structure detected: 16 experts, 2 active per token
96 expert tensors (32 layers × 3 tensor types)
Output file: phi-3.5-moe-f16.gguf (79GB)
```

### Run Server
```bash
cd shimmy
./target/release/shimmy serve \
  --bind 127.0.0.1:11435 \
  --cpu-moe \
  --model-path /path/to/phi-3.5-moe-f16.gguf
```

**Expected Logs** (excerpt):
```
llama_model_loader: - kv 15: phi3.expert_count u32 = 16
llama_model_loader: - kv 16: phi3.expert_used_count u32 = 2
tensor blk.0.ffn_gate_exps.weight (XXX MiB) buffer type overridden to CUDA_Host
load_tensors: CPU_Mapped model buffer size = XXXX MiB
load_tensors: CUDA0 model buffer size = XXXX MiB
```

### Benchmark
```bash
# Streaming test
curl -N -X POST http://127.0.0.1:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "phi-3.5-moe",
    "prompt": "Write a haiku about AI",
    "stream": true,
    "max_tokens": 100,
    "temperature": 0.3
  }'
```

### Conversion & Model Checksums
| Model | HF Source | Converter | Input SHA256 | Output SHA256 | License |
|-------|-----------|-----------|--------------|---------------|---------|
| GPT-OSS 20B | tensorblock/GPT-OSS-20B-GGUF | N/A (pre-converted) | *(add)* | *(add)* | Apache 2.0 |
| Phi-3.5-MoE | microsoft/Phi-3.5-MoE-instruct | llama.cpp b6686 | *(add)* | *(add)* | MIT |
| DeepSeek 16B | deepseek-ai/deepseek-moe-16b-base | llama.cpp b6686 | *(add)* | *(add)* | DeepSeek License |

**TODO**: Add SHA256 checksums for all files in reproduction run.

---

## Licensing & Compliance

### Model Licenses
- **GPT-OSS 20B**: Apache 2.0 (commercial use allowed)
- **Phi-3.5-MoE**: MIT License (commercial use allowed)
- **DeepSeek 16B**: DeepSeek License (check terms for commercial use)

### Redistribution Notice
GGUF files hosted on HuggingFace under our account are **derivative works** of original SafeTensors checkpoints. Usage must comply with upstream model licenses. We provide these for **research and evaluation purposes**.

### Shimmy License
- **Code**: MIT License
- **llama-cpp-rs fork**: MIT License (upstream: MIT)
- **llama.cpp**: MIT License

---

## Upstream Attribution

### llama.cpp MoE Offloading
- **Original Implementation**: PR #15077 by @slaren (https://github.com/ggml-org/llama.cpp/pull/15077)
- **Merged**: August 4, 2025
- **Flags**: `--cpu-moe`, `--n-cpu-moe N`
- **Mechanism**: `tensor_buft_overrides` with regex pattern matching

### Our Contribution
- **Rust Bindings**: `llama-cpp-2` crate methods `with_cpu_moe_all()`, `with_n_cpu_moe(n)`
- **Shimmy Integration**: CLI flags, configuration plumbing, testing framework
- **Validation**: Cross-model testing, documentation, HuggingFace model cards
- **Not Novel**: The core MoE offloading algorithm was already in llama.cpp

---

## Known Issues & Future Work

### Current Limitations
1. **No controlled A/B baselines**: Need paired runs (with/without `--cpu-moe`) on same hardware
2. **Inaccurate token counting**: Replace word_count heuristic with tokenizer-based counting
3. **Estimated TTFT**: Implement per-token timestamp logging
4. **Single-run measurements**: Add N≥3 runs with mean ± σ for all benchmarks
5. **Missing SHA256s**: Add checksums for all model files
6. **2MB claim unreproducible**: Historical build showed ~2MB VRAM; current builds measure 1.8-2.3GB

### Planned Improvements
- [ ] Implement accurate token counting (use model tokenizer)
- [ ] Add per-token timestamp logging for precise TTFT/TPS
- [ ] Run controlled A/B baselines (with/without `--cpu-moe`)
- [ ] Add statistical variance (N=3 minimum per test)
- [ ] Document SHA256 checksums for all files
- [ ] Add objective quality metrics (embedding similarity, pass@k)
- [ ] Reproduce or remove 2MB VRAM claim
- [ ] Add memory profiling (cudaMemGetInfo deltas)
- [ ] Document CPU pinning semantics (page-locked host memory)

### Discrepancy Investigation: 2MB vs 1.8GB
<<<<<<< HEAD
<<<<<<< HEAD
**Historical Claim**: Earlier builds (Oct 6) showed ~2MB VRAM usage
**Current Measurement**: Oct 7-8 builds show 1.8-2.3GB VRAM usage
=======
**Historical Claim**: Earlier builds (Oct 6) showed ~2MB VRAM usage  
**Current Measurement**: Oct 7-8 builds show 1.8-2.3GB VRAM usage  
>>>>>>> main
=======
**Historical Claim**: Earlier builds (Oct 6) showed ~2MB VRAM usage  
**Current Measurement**: Oct 7-8 builds show 1.8-2.3GB VRAM usage  
>>>>>>> main
**Possible Causes**:
1. Earlier measurement excluded CUDA allocator pools / KV cache
2. Different flash-attn or graph reservation flags
3. Sampler/KV cache configuration changes
4. More aggressive tensor mapping in earlier patch (since reverted)

**Status**: Under investigation. Until reproduced, we report **measured range of 1.8-2.3GB** and exclude the 2MB figure from summaries.

---

## Appendix: Raw Evidence

### Log File Locations
All raw benchmark outputs and server logs preserved for audit:
```
docs/benchmark-evidence/phi35-streaming-bench.log       # Phi-3.5-MoE performance
docs/benchmark-evidence/gpt-oss-streaming-bench.log     # GPT-OSS performance
docs/benchmark-evidence/deepseek-streaming-bench.log    # DeepSeek performance
docs/benchmark-evidence/shimmy-phi35.log                # Phi-3.5-MoE server logs
docs/benchmark-evidence/shimmy-gpt-oss.log              # GPT-OSS server logs
docs/benchmark-evidence/shimmy-deepseek.log             # DeepSeek server logs
```

### Key Log Patterns
**Expert Detection**:
```
llama_model_loader: - kv XX: <model>.expert_count u32 = <count>
llama_model_loader: - kv XX: <model>.expert_used_count u32 = <active>
```

**CPU Offloading Confirmation**:
```
tensor blk.X.ffn_gate_exps.weight (...) buffer type overridden to CUDA_Host
tensor blk.X.ffn_down_exps.weight (...) buffer type overridden to CUDA_Host
tensor blk.X.ffn_up_exps.weight (...) buffer type overridden to CUDA_Host
```

**Memory Distribution**:
```
load_tensors: CPU_Mapped model buffer size = XXXX MiB
load_tensors: CUDA0 model buffer size = XXXX MiB
```

---

## Contact & Support

<<<<<<< HEAD
<<<<<<< HEAD
**Repository**: https://github.com/Michael-A-Kuykendall/shimmy
**Branch**: feat/moe-cpu-offload
**Issues**: https://github.com/Michael-A-Kuykendall/shimmy/issues
=======
**Repository**: https://github.com/Michael-A-Kuykendall/shimmy  
**Branch**: feat/moe-cpu-offload  
**Issues**: https://github.com/Michael-A-Kuykendall/shimmy/issues  
>>>>>>> main
=======
**Repository**: https://github.com/Michael-A-Kuykendall/shimmy  
**Branch**: feat/moe-cpu-offload  
**Issues**: https://github.com/Michael-A-Kuykendall/shimmy/issues  
>>>>>>> main
**HuggingFace Models**:
- GPT-OSS 20B: https://huggingface.co/MikeKuykendall/gpt-oss-20b-moe-cpu-offload-gguf
- Phi-3.5-MoE: https://huggingface.co/MikeKuykendall/phi-3.5-moe-cpu-offload-gguf
- DeepSeek 16B: https://huggingface.co/MikeKuykendall/deepseek-moe-16b-cpu-offload-gguf

---

<<<<<<< HEAD
<<<<<<< HEAD
*Document Version: 1.0*
*Last Updated: October 8, 2025*
=======
*Document Version: 1.0*  
*Last Updated: October 8, 2025*  
>>>>>>> main
=======
*Document Version: 1.0*  
*Last Updated: October 8, 2025*  
>>>>>>> main
*Status: Technical validation for production use. Limitations and future work clearly documented.*
