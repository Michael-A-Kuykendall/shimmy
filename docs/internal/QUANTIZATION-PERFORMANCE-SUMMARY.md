# Quantization Performance Summary

<<<<<<< HEAD
<<<<<<< HEAD
**Test Date**: October 9, 2025
**Environment**: Lambda Cloud GH200 (96GB VRAM, 480GB RAM, CUDA 12.8)
**Tool**: shimmy v1.6.0 with llama.cpp b6686
=======
**Test Date**: October 9, 2025  
**Environment**: Lambda Cloud GH200 (96GB VRAM, 480GB RAM, CUDA 12.8)  
**Tool**: shimmy v1.6.0 with llama.cpp b6686  
>>>>>>> main
=======
**Test Date**: October 9, 2025  
**Environment**: Lambda Cloud GH200 (96GB VRAM, 480GB RAM, CUDA 12.8)  
**Tool**: shimmy v1.6.0 with llama.cpp b6686  
>>>>>>> main
**Runs per config**: N=3 (averaged below)

---

## Phi-3.5-MoE Quantizations

| Quantization | File Size | VRAM Baseline | VRAM Offload | VRAM Saved | Reduction % |
|-------------|-----------|---------------|--------------|------------|-------------|
| **Q2_K**    | 15 GB     | 14.78 GB      | 1.34 GB      | 13.44 GB   | **90.9%** |
| **Q4_K_M**  | 24 GB     | 24.14 GB      | 1.72 GB      | 22.42 GB   | **92.9%** |
| **Q8_0**    | 42 GB     | 41.91 GB      | 2.46 GB      | 39.45 GB   | **94.1%** |

**Original F16**: 79 GB file size

---

## DeepSeek-MoE-16B Quantizations

| Quantization | File Size | VRAM Baseline | VRAM Offload | VRAM Saved | Reduction % |
|-------------|-----------|---------------|--------------|------------|-------------|
| **Q2_K**    | 6.3 GB    | 7.28 GB       | 1.60 GB      | 5.68 GB    | **78.0%** |
| **Q4_K_M**  | 11 GB     | 11.10 GB      | 1.86 GB      | 9.24 GB    | **83.2%** |
| **Q8_0**    | 17 GB     | 17.11 GB      | 2.33 GB      | 14.78 GB   | **86.4%** |

**Original F16**: 31 GB file size

---

## Key Findings

### VRAM Reduction
- **Phi-3.5-MoE**: 90.9% - 94.1% VRAM reduction with CPU offloading
- **DeepSeek-16B**: 78.0% - 86.4% VRAM reduction with CPU offloading
- Larger quantizations (Q8_0) show higher reduction percentages
- All configurations successfully ran on GPU with <3 GB VRAM in offload mode

### File Size vs VRAM
- VRAM usage closely tracks file size for baseline (all-GPU) mode
- CPU offload mode dramatically reduces VRAM to ~1.3-2.5 GB regardless of quantization
- Offload overhead is small (consistent ~1.5-2.5 GB across all models)

### Generation Quality
- All quantizations produced coherent outputs
- Average token generation: 66-82 tokens per test
- No observed quality degradation in sample outputs (quantum computing explanations)

---

## Use Case Recommendations

### Q2_K - Maximum Compression
- **Best for**: Consumer hardware, tight VRAM budgets
- **Trade-off**: Smallest size, fastest loading, some quality loss
- **VRAM required**: 1.3-1.6 GB (offload) or 7-15 GB (baseline)

### Q4_K_M - Production Balance
- **Best for**: Production deployments, balanced quality/size
- **Trade-off**: Good quality retention, moderate size
- **VRAM required**: 1.7-1.9 GB (offload) or 11-24 GB (baseline)

### Q8_0 - Highest Quality
- **Best for**: Quality-critical applications, minimal degradation
- **Trade-off**: Largest size, closest to F16 quality
- **VRAM required**: 2.3-2.5 GB (offload) or 17-42 GB (baseline)

---

## Testing Notes

### Methodology
- Each configuration tested 3 times (N=3)
- Identical prompt: "Explain quantum computing in simple terms"
- Max tokens: 100
- Temperature: 0.7
- Seed: 42 (deterministic)

### VRAM Measurement
VRAM calculated as sum of three CUDA0 buffers:
1. Model buffer (main weight storage)
2. KV cache buffer (context storage)
3. Compute buffer (inference workspace)

### Excluded Models
- **GPT-OSS 20B**: Pre-quantized with MXFP4 by OpenAI, cannot requantize
  - See: `QUANTIZATION-TESTING-PLAN.md` for details

---

## Next Steps

1. ✅ Complete performance analysis
2. ⏳ Create individual model cards for each quantization
3. ⏳ Upload to HuggingFace with professional documentation
4. ⏳ Update technical validation report

**Status**: Analysis complete, ready for HuggingFace publication
