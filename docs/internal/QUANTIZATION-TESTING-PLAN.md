# Quantization Testing Plan - MoE CPU Offloading
<<<<<<< HEAD
<<<<<<< HEAD
**Date**: October 8, 2025
=======
**Date**: October 8, 2025  
>>>>>>> main
=======
**Date**: October 8, 2025  
>>>>>>> main
**Goal**: Validate MoE CPU offloading performance across multiple quantization levels

## Overview
Testing MoE CPU offloading feature with quantized models to:
1. Validate feature works across different quantization levels
2. Measure VRAM reduction vs speed tradeoff at each level
3. Provide data for users to choose optimal quant for their use case
4. Enable local testing on consumer hardware

## Models & Quantizations

### Selected Models (F16 Base)
1. **Phi-3.5-MoE 42B** (79GB F16)
   - 16 experts, 4096 hidden dim
   - Excellent for testing at multiple quant levels
<<<<<<< HEAD
<<<<<<< HEAD

=======
   
>>>>>>> main
=======
   
>>>>>>> main
2. **DeepSeek MoE 16B** (31GB F16)
   - 64 regular + 2 shared experts
   - Unique dual-expert architecture

**Note**: GPT-OSS 20B excluded (pre-quantized with MXFP4, cannot requantize)

### Target Quantizations (6 total)
For each model, create 3 quantization levels:
- **Q4_K_M**: Medium quality, ~4-bit per weight (good balance)
- **Q2_K**: Extreme compression, ~2-bit per weight (max VRAM savings)
- **Q8_0**: High quality, ~8-bit per weight (minimal quality loss)

**Quantized Models**:
```
phi-3.5-moe-Q4_K_M.gguf    (~20GB estimated)
phi-3.5-moe-Q2_K.gguf      (~10GB estimated)
phi-3.5-moe-Q8_0.gguf      (~40GB estimated)

deepseek-moe-16b-Q4_K_M.gguf  (~8GB estimated)
deepseek-moe-16b-Q2_K.gguf    (~4GB estimated)
deepseek-moe-16b-Q8_0.gguf    (~16GB estimated)
```

## Testing Protocol

### Test Configuration (N=3 per config)
For each quantized model:
1. **Baseline (GPU)**: No CPU offload, measure full VRAM usage
2. **CPU Offload**: With `--cpu-moe` flag, measure reduced VRAM usage

### Metrics Collected
- **VRAM Usage**: GPU memory consumed (MB)
- **TPS**: Tokens per second (throughput)
- **TTFT**: Time to first token (ms, latency)

### Test Command
```bash
./shimmy generate \
  --model <model_file> \
  --prompt "Explain quantum computing in simple terms." \
  --max-tokens 100 \
  [--cpu-moe]  # For offload tests
```

### Test Matrix (36 total runs)
| Model | Quant | Config | Runs | Total |
|-------|-------|--------|------|-------|
| Phi-3.5-MoE | Q4_K_M | Baseline | 3 | 3 |
| Phi-3.5-MoE | Q4_K_M | CPU Offload | 3 | 3 |
| Phi-3.5-MoE | Q2_K | Baseline | 3 | 3 |
| Phi-3.5-MoE | Q2_K | CPU Offload | 3 | 3 |
| Phi-3.5-MoE | Q8_0 | Baseline | 3 | 3 |
| Phi-3.5-MoE | Q8_0 | CPU Offload | 3 | 3 |
| DeepSeek | Q4_K_M | Baseline | 3 | 3 |
| DeepSeek | Q4_K_M | CPU Offload | 3 | 3 |
| DeepSeek | Q2_K | Baseline | 3 | 3 |
| DeepSeek | Q2_K | CPU Offload | 3 | 3 |
| DeepSeek | Q8_0 | Baseline | 3 | 3 |
| DeepSeek | Q8_0 | CPU Offload | 3 | 3 |
| **TOTAL** | | | | **36 runs** |

## Execution Timeline

### Phase 1: Quantization (In Progress ✅)
- **Script**: `./quantize-all.sh`
- **ETA**: ~30-60 minutes
- **Status**: Running (currently on Phi-3.5-MoE Q2_K, layer 29/32)
- **Output**: 6 quantized GGUF files in `/home/ubuntu/models/`

### Phase 2: Baseline Testing (Cloud Instance)
- **Script**: `./test-quantized-models.sh`
- **ETA**: ~2-3 hours (6 models × 2 configs × 3 runs × ~3min each)
- **Environment**: Lambda Cloud GH200 (96GB VRAM, 480GB RAM)
- **Output**: JSON results in `./quantization-test-results/`

### Phase 3: Local Testing (User's Machine)
- **Goal**: Validate low-quant models (Q2_K, Q4_K_M) on consumer hardware
- **Focus**: Phi-3.5-MoE Q2_K with CPU offload (most practical for local use)
- **Use Case**: Streaming inference on limited VRAM setups

## Expected Results

### Hypothesis: Quantization Level vs Offload Benefit
1. **Q2_K**: Smallest VRAM footprint, fastest offload (less data to move)
2. **Q4_K_M**: Good balance of quality and VRAM savings
3. **Q8_0**: Highest quality, larger VRAM footprint (still benefits from offload)

### Key Questions to Answer
1. Does CPU offload work equally well across all quant levels?
2. Is there a "sweet spot" quantization for local use with offload?
3. How does speed penalty change with quantization level?
4. What's the minimum VRAM needed for each quant + offload?

## Deliverables

### 1. Test Results
- Raw JSON output for each run
- Summary markdown with aggregated metrics
- Comparison tables (baseline vs offload, by quant level)

### 2. Model Cards (6 total)
Professional HuggingFace model cards for each quantization:
- Model specs (size, quant method, architecture)
- Usage instructions (shimmy CLI + code examples)
- Performance data (VRAM, TPS, TTFT with/without offload)
- Recommended use cases for each quant level

### 3. HuggingFace Uploads
- 6 quantized GGUF files
- 6 model cards (README.md)
- Repos:
  * `MikeKuykendall/phi-3.5-moe-cpu-offload-gguf` (3 quants)
  * `MikeKuykendall/deepseek-moe-16b-cpu-offload-gguf` (3 quants)

### 4. Technical Documentation
- Update `docs/MOE-TECHNICAL-VALIDATION.md` with quantization results
- Add quantization comparison section
- Include recommendations for users

## Success Criteria
<<<<<<< HEAD
<<<<<<< HEAD
✅ All 6 quantizations complete successfully
✅ All 36 baseline tests run without errors
✅ VRAM measurements accurate (no 0MB/3MB issues)
✅ CPU offload shows consistent VRAM reduction across quant levels
✅ Model cards professional and accurate
✅ Files uploaded to HuggingFace with proper documentation
=======
=======
>>>>>>> main
✅ All 6 quantizations complete successfully  
✅ All 36 baseline tests run without errors  
✅ VRAM measurements accurate (no 0MB/3MB issues)  
✅ CPU offload shows consistent VRAM reduction across quant levels  
✅ Model cards professional and accurate  
✅ Files uploaded to HuggingFace with proper documentation  
<<<<<<< HEAD
>>>>>>> main
=======
>>>>>>> main

## Notes
- **GPT-OSS 20B excluded**: Original OpenAI model uses MXFP4 quantization by design, cannot requantize
- **Test environment**: GH200 with CUDA 12.8, llama-quantize b6686
- **Baseline from**: F16 models downloaded from MaziyarPanahi (Phi), unsloth (DeepSeek)
- **Previous testing**: F16 baselines already collected, this extends to quantized versions
