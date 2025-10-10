---
language:
- en
- zh
license: apache-2.0
tags:
- gguf
- quantized
- moe
- mixture-of-experts
- cpu-offload
- text-generation
- deepseek
base_model: deepseek-ai/deepseek-moe-16b-base
quantized_by: MikeKuykendall
pipeline_tag: text-generation
---

# DeepSeek-MoE-16B Q4_K_M with CPU Offloading

This is a Q4_K_M quantization of DeepSeek's DeepSeek-MoE-16B model with MoE (Mixture of Experts) CPU offloading capability enabled via Rust bindings for llama.cpp.

## Model Details

- **Base Model**: [deepseek-ai/deepseek-moe-16b-base](https://huggingface.co/deepseek-ai/deepseek-moe-16b-base)
- **Quantization**: Q4_K_M (4-bit, K-quant medium)
- **File Size**: 11 GB (from 31 GB F16)
- **Architecture**: Mixture of Experts (MoE)
- **License**: Apache 2.0
- **Feature**: MoE expert CPU offloading support

## Performance Benchmarks

Tested on Lambda Cloud GH200 (96GB VRAM, 480GB RAM, CUDA 12.8) with shimmy v1.6.0:

| Configuration | VRAM Usage | VRAM Saved | Reduction |
|--------------|------------|------------|-----------|
| **All GPU** (baseline) | 11.10 GB | - | - |
| **CPU Offload** (`--cpu-moe`) | 1.86 GB | 9.24 GB | **83.2%** |

### Key Metrics
- **VRAM Reduction**: 83.2% with CPU offloading enabled
- **Generation Quality**: Good quality for Q4_K_M quantization
- **Average Tokens Generated**: 66 tokens per test (N=3)
- **Test Prompt**: "Explain quantum computing in simple terms"

## What is MoE CPU Offloading?

Mixture of Experts models activate only a subset of parameters per token (sparse activation). This quantization includes Rust bindings that expose llama.cpp's MoE CPU offloading feature, allowing inactive experts to reside in system RAM instead of VRAM.

**Note**: The core MoE CPU offloading algorithm was implemented in llama.cpp (PR #15077, August 2025). This release provides Rust language bindings and production integration for that functionality.

## Usage

### With shimmy CLI

```bash
# Download the model
huggingface-cli download MikeKuykendall/deepseek-moe-16b-q4-k-m-cpu-offload-gguf \
  deepseek-moe-16b-Q4_K_M.gguf --local-dir ./models

# Run with CPU offloading (uses ~1.9 GB VRAM)
shimmy serve \
  --model-dirs ./models \
  --cpu-moe \
  --bind 127.0.0.1:11435

# Run without offloading (uses ~11 GB VRAM)
shimmy serve \
  --model-dirs ./models \
  --bind 127.0.0.1:11435
```

### With llama-cpp-2 (Rust)

```rust
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;

fn main() {
    let backend = LlamaBackend::init().unwrap();
<<<<<<< HEAD
<<<<<<< HEAD

    // Enable MoE CPU offloading
    let model_params = LlamaModelParams::default()
        .with_cpu_moe_all();  // Offload all inactive experts to CPU

=======
=======
>>>>>>> main
    
    // Enable MoE CPU offloading
    let model_params = LlamaModelParams::default()
        .with_cpu_moe_all();  // Offload all inactive experts to CPU
    
<<<<<<< HEAD
>>>>>>> main
=======
>>>>>>> main
    let model = LlamaModel::load_from_file(
        &backend,
        "deepseek-moe-16b-Q4_K_M.gguf",
        &model_params
    ).unwrap();
<<<<<<< HEAD
<<<<<<< HEAD

    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(2048);

    let mut ctx = model.new_context(&backend, ctx_params).unwrap();

=======
=======
>>>>>>> main
    
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(2048);
    
    let mut ctx = model.new_context(&backend, ctx_params).unwrap();
    
<<<<<<< HEAD
>>>>>>> main
=======
>>>>>>> main
    // ... tokenize and generate as normal
}
```

### With llama.cpp (C++)

```bash
# Build llama.cpp with CUDA support
cmake -B build -DGGML_CUDA=ON
cmake --build build --config Release

# Run with CPU offloading
./build/bin/llama-cli \
  -m deepseek-moe-16b-Q4_K_M.gguf \
  -p "Explain quantum computing" \
  --cpu-moe
```

## When to Use This Quantization

### ✅ Use Q4_K_M if you want:
- **Balanced quality/size**: Best general-purpose quantization
- **Production deployments**: Reliable quality with reasonable file size
- **VRAM constraints**: 1.9 GB VRAM with offloading, or 11 GB without
- **Smaller model**: 16B parameters, faster than larger MoE models

### ❌ Consider alternatives if:
- **Maximum compression needed** → Use Q2_K variant (6.3 GB, 1.6 GB VRAM)
- **Highest quality required** → Use Q8_0 variant (17 GB, 2.3 GB VRAM)
- **Original precision needed** → Use F16 base model (31 GB)

## Quantization Details

- **Method**: K-quant medium (Q4_K_M)
- **Bits per weight**: ~4.5 bits average
- **Quantization tool**: llama-quantize (llama.cpp b6686)
- **Source**: F16 version of deepseek-ai/deepseek-moe-16b-base

## Technical Notes

### MoE Architecture
DeepSeek-MoE-16B uses a sparse Mixture of Experts architecture with 16 billion parameters. Only a subset of experts are activated per token, enabling high capacity with efficient inference.

### CPU Offloading Implementation
The `--cpu-moe` flag (or `with_cpu_moe_all()` in Rust) tells llama.cpp to:
1. Keep active experts in VRAM for fast inference
2. Move inactive experts to system RAM
3. Swap experts as needed during generation

This dramatically reduces VRAM usage with a manageable performance trade-off.

### VRAM Breakdown (CPU Offload Mode)
- Model buffer: ~0.7 GB (active experts only)
- KV cache: 0.51 GB
- Compute buffer: 0.10 GB
- **Total**: ~1.9 GB

## Sample Output

**Prompt**: "Explain quantum computing in simple terms"

**Response**: (Generated coherent explanation suitable for Q4_K_M quantization quality)

## Citation

If you use this model in your work, please cite the original DeepSeek paper:

```bibtex
@article{deepseek-moe,
  title={DeepSeekMoE: Towards Ultimate Expert Specialization in Mixture-of-Experts Language Models},
  author={DeepSeek-AI},
  year={2024}
}
```

## Links

- **Original Model**: [deepseek-ai/deepseek-moe-16b-base](https://huggingface.co/deepseek-ai/deepseek-moe-16b-base)
- **shimmy Project**: [github.com/utilityai/shimmy](https://github.com/utilityai/shimmy)
- **llama.cpp**: [github.com/ggerganov/llama.cpp](https://github.com/ggerganov/llama.cpp)
- **Other Quantizations**:
  - [Q2_K (6.3 GB, 1.6 GB VRAM)](../deepseek-moe-16b-q2-k-cpu-offload-gguf)
  - [Q8_0 (17 GB, 2.3 GB VRAM)](../deepseek-moe-16b-q8-0-cpu-offload-gguf)

---

<<<<<<< HEAD
<<<<<<< HEAD
**License**: Apache 2.0 (inherited from base model)
**Quantized by**: MikeKuykendall
=======
**License**: Apache 2.0 (inherited from base model)  
**Quantized by**: MikeKuykendall  
>>>>>>> main
=======
**License**: Apache 2.0 (inherited from base model)  
**Quantized by**: MikeKuykendall  
>>>>>>> main
**Date**: October 2025
