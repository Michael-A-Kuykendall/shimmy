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

# DeepSeek-MoE-16B Q8_0 with CPU Offloading

Q8_0 quantization of DeepSeek-MoE-16B with CPU offloading support. Highest quality, near-F16 accuracy.

## Performance

| Configuration | VRAM | Saved | Reduction |
|--------------|------|-------|-----------|
| **All GPU** | 17.11 GB | - | - |
| **CPU Offload** | 2.33 GB | 14.78 GB | **86.4%** |

**File Size**: 17 GB (from 31 GB F16)

## Usage

```bash
huggingface-cli download MikeKuykendall/deepseek-moe-16b-q8-0-cpu-offload-gguf
shimmy serve --model-dirs ./models --cpu-moe
```

**Links**: [Q2_K](../deepseek-moe-16b-q2-k-cpu-offload-gguf) | [Q4_K_M](../deepseek-moe-16b-q4-k-m-cpu-offload-gguf)

License: Apache 2.0
