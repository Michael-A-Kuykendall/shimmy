---
license: apache-2.0
license_link: https://huggingface.co/deepseek-ai/deepseek-moe-16b-base/blob/main/LICENSE
base_model: deepseek-ai/deepseek-moe-16b-base
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
- en
- zh
pipeline_tag: text-generation
library_name: llama.cpp
---

# DeepSeek MoE 16B Base - F16 GGUF with MoE CPU Offloading Support

F16 GGUF conversion of [deepseek-ai/deepseek-moe-16b-base](https://huggingface.co/deepseek-ai/deepseek-moe-16b-base) with Rust bindings for llama.cpp's MoE CPU offloading functionality.

## Model Details

- **Base Model**: [deepseek-ai/deepseek-moe-16b-base](https://huggingface.co/deepseek-ai/deepseek-moe-16b-base)
- **Format**: GGUF F16 precision
- **File Size**: 31GB
- **Parameters**: 16.4B total (2.8B active per token)
- **Architecture**: 28 layers, 64 regular experts + 2 shared experts, 6 active per token
- **Context Length**: 4K tokens
- **Converted by**: [MikeKuykendall](https://huggingface.co/MikeKuykendall)

## MoE CPU Offloading

This model supports **MoE CPU offloading** via llama.cpp (implemented in [PR #15077](https://github.com/ggml-org/llama.cpp/pull/15077)). Shimmy provides Rust bindings for this functionality, enabling:

- **VRAM Reduction**: 92.5% (30.1GB → 2.3GB measured on GH200)
- **Performance Trade-off**: 4.1x slower generation (26.8 → 6.5 TPS)
- **Use Case**: Running 16B parameter MoE on consumer GPUs (<4GB VRAM)

### Controlled Baseline (NVIDIA GH200, N=3)

| Configuration | VRAM | TPS | TTFT |
|---------------|------|-----|------|
| **GPU-only** | 30.1GB | 26.8 | 426ms |
| **CPU Offload** | 2.3GB | 6.5 | 1,643ms |

**Trade-off**: Memory for speed. Best for VRAM-constrained scenarios where generation speed is less critical than model size.

### Unique Architecture

DeepSeek MoE uses a **dual-expert architecture** (64 regular + 2 shared experts), validated to work correctly with CPU offloading:
- Regular experts: `ffn_gate_exps.weight`, `ffn_down_exps.weight`, `ffn_up_exps.weight`
- Shared experts: `ffn_gate_shexp.weight`, `ffn_down_shexp.weight`, `ffn_up_shexp.weight`

## Download

```bash
huggingface-cli download MikeKuykendall/deepseek-moe-16b-cpu-offload-gguf \
  --include "deepseek-moe-16b-f16.gguf" \
  --local-dir ./models
```

## Usage

### llama.cpp (CPU Offloading)

```bash
# Standard loading (requires ~32GB VRAM)
./llama-server -m deepseek-moe-16b-f16.gguf -c 4096

# With MoE CPU offloading (requires ~3GB VRAM + 32GB RAM)
./llama-server -m deepseek-moe-16b-f16.gguf -c 4096 --cpu-moe
```

### Shimmy (Rust Bindings)

```bash
# Install Shimmy
cargo install --git https://github.com/Michael-A-Kuykendall/shimmy --features llama-cuda

# Standard loading
shimmy serve --model deepseek-moe-16b-f16.gguf

# With MoE CPU offloading
shimmy serve --model deepseek-moe-16b-f16.gguf --cpu-moe

# Query the API
curl http://localhost:11435/api/generate \
  -d '{
    "model": "deepseek-moe-16b",
    "prompt": "Explain the architecture of DeepSeek MoE",
    "max_tokens": 256,
    "stream": false
  }'
```

## Performance Notes

**Standard GPU Loading**:
- VRAM: 30.1GB
- Speed: 26.8 TPS
- Latency: 426ms TTFT
- Use when: VRAM is plentiful, speed is critical

**CPU Offloading**:
- VRAM: 2.3GB (92.5% reduction)
- Speed: 6.5 TPS (4.1x slower)
- Latency: 1,643ms TTFT
- Use when: Limited VRAM, speed less critical

## Original Model

- **Developers**: DeepSeek AI
- **License**: Apache 2.0
- **Paper**: [DeepSeekMoE: Towards Ultimate Expert Specialization in Mixture-of-Experts Language Models](https://arxiv.org/abs/2401.06066)
- **Languages**: English, Chinese

## Technical Validation

Full validation report with controlled baselines: [Shimmy MoE CPU Offloading Technical Report](https://github.com/Michael-A-Kuykendall/shimmy/blob/feat/moe-cpu-offload/docs/MOE-TECHNICAL-REPORT.md)

## Citation

```bibtex
@article{dai2024deepseekmoe,
  title={DeepSeekMoE: Towards Ultimate Expert Specialization in Mixture-of-Experts Language Models},
  author={Dai, Damai and others},
  journal={arXiv preprint arXiv:2401.06066},
  year={2024}
}
```

---

*GGUF conversion and MoE offloading validation by [MikeKuykendall](https://huggingface.co/MikeKuykendall)*
