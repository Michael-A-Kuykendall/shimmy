---
language:
- en
- multilingual
license: mit
tags:
- gguf
- quantized
- moe
- mixture-of-experts
- cpu-offload
- text-generation
base_model: microsoft/Phi-3.5-MoE-instruct
quantized_by: MikeKuykendall
pipeline_tag: text-generation
---

# Phi-3.5-MoE Q2_K with CPU Offloading

This is a Q2_K quantization of Microsoft's Phi-3.5-MoE-Instruct model with MoE (Mixture of Experts) CPU offloading capability enabled via Rust bindings for llama.cpp.

## Model Details

- **Base Model**: [microsoft/Phi-3.5-MoE-instruct](https://huggingface.co/microsoft/Phi-3.5-MoE-instruct)
- **Quantization**: Q2_K (2-bit, K-quant)
- **File Size**: 15 GB (from 79 GB F16)
- **Architecture**: Mixture of Experts (MoE)
- **License**: MIT
- **Feature**: MoE expert CPU offloading support

## Performance Benchmarks

Tested on Lambda Cloud GH200 (96GB VRAM, 480GB RAM, CUDA 12.8) with shimmy v1.6.0:

| Configuration | VRAM Usage | VRAM Saved | Reduction |
|--------------|------------|------------|-----------|
| **All GPU** (baseline) | 14.78 GB | - | - |
| **CPU Offload** (`--cpu-moe`) | 1.34 GB | 13.44 GB | **90.9%** |

### Key Metrics
- **VRAM Reduction**: 90.9% with CPU offloading enabled
- **Generation Quality**: Coherent outputs for general use
- **Average Tokens Generated**: 73 tokens per test (N=3)
- **Test Prompt**: "Explain quantum computing in simple terms"

## What is MoE CPU Offloading?

Mixture of Experts models activate only a subset of parameters per token (sparse activation). This quantization includes Rust bindings that expose llama.cpp's MoE CPU offloading feature, allowing inactive experts to reside in system RAM instead of VRAM.

**Note**: The core MoE CPU offloading algorithm was implemented in llama.cpp (PR #15077, August 2025). This release provides Rust language bindings and production integration for that functionality.

## Usage

### With shimmy CLI

```bash
# Download the model
huggingface-cli download MikeKuykendall/phi-3.5-moe-q2-k-cpu-offload-gguf \
  phi-3.5-moe-Q2_K.gguf --local-dir ./models

# Run with CPU offloading (uses ~1.3 GB VRAM)
shimmy serve \
  --model-dirs ./models \
  --cpu-moe \
  --bind 127.0.0.1:11435

# Run without offloading (uses ~15 GB VRAM)
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
        "phi-3.5-moe-Q2_K.gguf",
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
  -m phi-3.5-moe-Q2_K.gguf \
  -p "Explain quantum computing" \
  --cpu-moe
```

## When to Use This Quantization

### ✅ Use Q2_K if you want:
- **Maximum compression**: Smallest file size (15 GB vs 79 GB F16)
- **Minimal VRAM**: Only 1.3 GB VRAM with CPU offloading
- **Consumer hardware**: Perfect for local/personal machines with limited VRAM
- **Experimentation**: Fast downloads, quick to test

### ❌ Consider alternatives if:
- **Production quality needed** → Use [Q4_K_M variant](../phi-3.5-moe-q4-k-m-cpu-offload-gguf) (24 GB, better quality)
- **Highest quality required** → Use [Q8_0 variant](../phi-3.5-moe-q8-0-cpu-offload-gguf) (42 GB, minimal degradation)
- **Original precision needed** → Use F16 base model (79 GB)

## Quantization Details

- **Method**: K-quant 2-bit (Q2_K)
- **Bits per weight**: ~2.5 bits average
- **Quantization tool**: llama-quantize (llama.cpp b6686)
- **Source**: F16 version of microsoft/Phi-3.5-MoE-instruct
- **Trade-off**: Smaller size, some quality loss acceptable for most tasks

## Technical Notes

### MoE Architecture
Phi-3.5-MoE uses a sparse Mixture of Experts architecture where only a subset of experts are activated per token. This allows the model to have high capacity (many parameters) while maintaining efficiency (sparse activation).

### CPU Offloading Implementation
The `--cpu-moe` flag (or `with_cpu_moe_all()` in Rust) tells llama.cpp to:
1. Keep active experts in VRAM for fast inference
2. Move inactive experts to system RAM
3. Swap experts as needed during generation

This dramatically reduces VRAM usage with a manageable performance trade-off.

### VRAM Breakdown (CPU Offload Mode)
- Model buffer: ~0.2 GB (active experts only)
- KV cache: 0.51 GB
- Compute buffer: 0.10 GB
- **Total**: ~1.3 GB

## Sample Output

**Prompt**: "Explain quantum computing in simple terms"

**Response**:
> Sure! Imagine you have a magical coin that can land on heads or tails in a super-special way. When you flip it, instead of just being heads OR tails, it can be both at the same time...

(Coherent response generated, suitable quality for Q2_K quantization)

## Citation

If you use this model in your work, please cite the original Phi-3.5 paper and acknowledge the quantization:

```bibtex
@article{phi3.5,
  title={Phi-3 Technical Report: A Highly Capable Language Model Locally on Your Phone},
  author={Microsoft Research},
  year={2024}
}
```

## Links

- **Original Model**: [microsoft/Phi-3.5-MoE-instruct](https://huggingface.co/microsoft/Phi-3.5-MoE-instruct)
- **shimmy Project**: [github.com/utilityai/shimmy](https://github.com/utilityai/shimmy)
- **llama.cpp**: [github.com/ggerganov/llama.cpp](https://github.com/ggerganov/llama.cpp)
- **Other Quantizations**:
  - [Q4_K_M (24 GB, 1.7 GB VRAM)](../phi-3.5-moe-q4-k-m-cpu-offload-gguf)
  - [Q8_0 (42 GB, 2.5 GB VRAM)](../phi-3.5-moe-q8-0-cpu-offload-gguf)

---

<<<<<<< HEAD
<<<<<<< HEAD
**License**: MIT (inherited from base model)
**Quantized by**: MikeKuykendall
=======
**License**: MIT (inherited from base model)  
**Quantized by**: MikeKuykendall  
>>>>>>> main
=======
**License**: MIT (inherited from base model)  
**Quantized by**: MikeKuykendall  
>>>>>>> main
**Date**: October 2025
