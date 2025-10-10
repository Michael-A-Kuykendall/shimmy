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

# DeepSeek-MoE-16B Q2_K with CPU Offloading

Q2_K quantization of DeepSeek-MoE-16B with CPU offloading support. Smallest size, maximum VRAM savings.

## Performance

| Configuration | VRAM | Saved | Reduction |
|--------------|------|-------|-----------|
| **All GPU** | 7.28 GB | - | - |
| **CPU Offload** | 1.60 GB | 5.68 GB | **78.0%** |

**File Size**: 6.3 GB (from 31 GB F16)

## Usage

```bash
huggingface-cli download MikeKuykendall/deepseek-moe-16b-q2-k-cpu-offload-gguf
shimmy serve --model-dirs ./models --cpu-moe
```

**Links**: [Q4_K_M](../deepseek-moe-16b-q4-k-m-cpu-offload-gguf) | [Q8_0](../deepseek-moe-16b-q8-0-cpu-offload-gguf)

License: Apache 2.0
