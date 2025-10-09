---
license: mit
license_link: https://huggingface.co/microsoft/Phi-3.5-MoE-instruct/resolve/main/LICENSE
base_model: microsoft/Phi-3.5-MoE-instruct
tags:
- moe
- mixture-of-experts
- gguf
- llama.cpp
- shimmy
- rust
- cpu-offload
quantized_by: MikeKuykendall
language:
- multilingual
pipeline_tag: text-generation
library_name: llama.cpp
---

# Phi-3.5-MoE Instruct - F16 GGUF with MoE CPU Offloading Support

F16 GGUF conversion of [microsoft/Phi-3.5-MoE-instruct](https://huggingface.co/microsoft/Phi-3.5-MoE-instruct) with Rust bindings for llama.cpp's MoE CPU offloading functionality.

## Model Details

- **Base Model**: [microsoft/Phi-3.5-MoE-instruct](https://huggingface.co/microsoft/Phi-3.5-MoE-instruct)
- **Format**: GGUF F16 precision
- **File Size**: 79GB
- **Parameters**: 41.9B total (6.6B active per token)
- **Architecture**: 32 layers, 16 experts per layer, 2 active experts per token
- **Context Length**: 131K tokens
- **Converted by**: [MikeKuykendall](https://huggingface.co/MikeKuykendall)

## MoE CPU Offloading

This model supports **MoE CPU offloading** via llama.cpp (implemented in [PR #15077](https://github.com/ggml-org/llama.cpp/pull/15077)). Shimmy provides Rust bindings for this functionality, enabling:

- **VRAM Reduction**: 96.5% (77.7GB → 2.8GB measured on GH200)
- **Performance Trade-off**: 3.1x slower generation (13.8 → 4.5 TPS)
- **Use Case**: Running 42B parameter MoE on consumer GPUs (<10GB VRAM)

### Controlled Baseline (NVIDIA GH200, N=3)

| Configuration | VRAM | TPS | TTFT |
|---------------|------|-----|------|
| **GPU-only** | 77.7GB | 13.8 | 730ms |
| **CPU Offload** | 2.8GB | 4.5 | 2,251ms |

**Trade-off**: Memory for speed. Best for VRAM-constrained scenarios where generation speed is less critical than model size.

## Download

```bash
huggingface-cli download MikeKuykendall/phi-3.5-moe-cpu-offload-gguf \
  --include "phi-3.5-moe-f16.gguf" \
  --local-dir ./models
```

## Usage

### llama.cpp (CPU Offloading)

```bash
# Standard loading (requires ~80GB VRAM)
./llama-server -m phi-3.5-moe-f16.gguf -c 4096

# With MoE CPU offloading (requires ~3GB VRAM + 80GB RAM)
./llama-server -m phi-3.5-moe-f16.gguf -c 4096 --cpu-moe
```

### Shimmy (Rust Bindings)

```bash
# Install Shimmy
cargo install --git https://github.com/Michael-A-Kuykendall/shimmy --features llama-cuda

# Standard loading
shimmy serve --model phi-3.5-moe-f16.gguf

# With MoE CPU offloading
shimmy serve --model phi-3.5-moe-f16.gguf --cpu-moe

# Query the API
curl http://localhost:11435/api/generate \
  -d '{
    "model": "phi-3.5-moe",
    "prompt": "Explain mixture of experts in simple terms",
    "max_tokens": 256,
    "stream": false
  }'
```

## Prompt Format

```
<|system|>
You are a helpful assistant.<|end|>
<|user|>
Your question here<|end|>
<|assistant|>
```

## Performance Notes

**Standard GPU Loading**:
- VRAM: 77.7GB
- Speed: 13.8 TPS
- Latency: 730ms TTFT
- Use when: VRAM is plentiful, speed is critical

**CPU Offloading**:
- VRAM: 2.8GB (96.5% reduction)
- Speed: 4.5 TPS (3.1x slower)
- Latency: 2,251ms TTFT
- Use when: Limited VRAM, speed less critical

## Original Model

- **Developers**: Microsoft
- **License**: MIT
- **Paper**: [Phi-3 Technical Report](https://arxiv.org/abs/2404.14219)
- **Blog**: [Phi-3.5-MoE Announcement](https://techcommunity.microsoft.com/t5/ai-azure-ai-services-blog/announcing-the-availability-of-phi-3-5-moe-in-azure-ai-studio/ba-p/4256278)

## Technical Validation

Full validation report with controlled baselines: [Shimmy MoE CPU Offloading Technical Report](https://github.com/Michael-A-Kuykendall/shimmy/blob/feat/moe-cpu-offload/docs/MOE-TECHNICAL-REPORT.md)

## Citation

```bibtex
@techreport{abdin2024phi,
  title={Phi-3 Technical Report: A Highly Capable Language Model Locally on Your Phone},
  author={Abdin, Marah and others},
  year={2024},
  institution={Microsoft}
}
```

---

*GGUF conversion and MoE offloading validation by [MikeKuykendall](https://huggingface.co/MikeKuykendall)*
