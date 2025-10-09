# Quantization & Testing Status Report
**Updated**: October 8, 2025 23:20 UTC

## ✅ Phase 1: Quantization - COMPLETE

### Quantized Models Created (6 total)
All quantizations completed successfully using llama-quantize (b6686, CUDA-enabled):

#### Phi-3.5-MoE (Base: 79GB F16)
| Quantization | Size | Reduction | File |
|-------------|------|-----------|------|
| Q2_K | 15GB | 81% | phi-3.5-moe-Q2_K.gguf |
| Q4_K_M | 24GB | 70% | phi-3.5-moe-Q4_K_M.gguf |
| Q8_0 | 42GB | 47% | phi-3.5-moe-Q8_0.gguf |

#### DeepSeek MoE 16B (Base: 31GB F16)
| Quantization | Size | Reduction | File |
|-------------|------|-----------|------|
| Q2_K | 6.3GB | 80% | deepseek-moe-16b-Q2_K.gguf |
| Q4_K_M | 11GB | 65% | deepseek-moe-16b-Q4_K_M.gguf |
| Q8_0 | 17GB | 45% | deepseek-moe-16b-Q8_0.gguf |

**Note**: GPT-OSS 20B excluded - OpenAI released it with MXFP4 quantization by design, cannot requantize.

## ⏳ Phase 2: Baseline Testing - IN PROGRESS

### Test Matrix (36 runs total)
- **Models**: 6 quantized versions (3 × Phi, 3 × DeepSeek)
- **Configs**: 2 per model (baseline GPU, CPU offload)
- **Runs**: 3 per config (N=3 for statistical validity)
- **Total**: 36 test runs

### Current Progress
- **Started**: October 8, 2025 23:19 UTC
- **Status**: Running baseline tests on phi-3.5-moe-q4-k-m
- **ETA**: ~2-3 hours total
- **Output**: `./quantization-test-results/*.json` + `SUMMARY.md`

### Test Command Format
```bash
shimmy --model-dirs /home/ubuntu/models \
  [--cpu-moe] \
  generate \
  --prompt "Explain quantum computing in simple terms." \
  --max-tokens 100 \
  <model-name>
```

### Metrics Being Collected
- ✅ **Model loads successfully** (yes/no)
- ✅ **Generation completes** (100 tokens)
- ⚠️  **VRAM/TPS/TTFT**: Not currently output by shimmy generate command
  * **Next step**: Add instrumentation or use alternate measurement method

## 📋 Next Steps

### Immediate (After Testing Complete)
1. **Verify all 36 tests passed** (no failures)
2. **Add VRAM/performance measurement** (shimmy doesn't currently output these)
3. **Create model cards** for each quantization (6 total)
4. **Upload to HuggingFace**:
   - `MikeKuykendall/phi-3.5-moe-cpu-offload-gguf` (add Q2_K, Q4_K_M, Q8_0)
   - `MikeKuykendall/deepseek-moe-16b-cpu-offload-gguf` (add Q2_K, Q4_K_M, Q8_0)

### Local Testing (User's Machine)
1. **Download Q2_K models** (smallest, most practical for local)
2. **Test streaming performance** with shimmy serve + SSE
3. **Validate CPU offload benefit** on consumer hardware
4. **Compare to existing F16 baselines**

## 📊 Expected Outcomes

### Hypothesis: Quantization Sweet Spots
1. **Q2_K**: Best for local use (smallest VRAM, fastest CPU offload)
2. **Q4_K_M**: Balance of quality and size (production use)
3. **Q8_0**: Highest quality, larger footprint (minimal degradation)

### Questions to Answer
- Does CPU offload work equally well across all quant levels?
- What's the VRAM minimum for each quant + offload?
- Is there a speed penalty difference by quant level?
- Which quant level is optimal for streaming on consumer hardware?

## 🎯 Success Criteria

### Must Have
- ✅ All 6 quantizations complete (DONE)
- ⏳ All 36 baseline tests run successfully (IN PROGRESS)
- ⏳ Model cards professional and accurate
- ⏳ Files uploaded with proper documentation

### Nice to Have
- Performance metrics (VRAM/TPS/TTFT) for each config
- Local validation on consumer hardware
- Streaming performance comparison
- User recommendations by use case

## 📁 Files & Locations

### Quantized Models
```
/home/ubuntu/models/
├── phi-3.5-moe-Q2_K.gguf (15GB)
├── phi-3.5-moe-Q4_K_M.gguf (24GB)
├── phi-3.5-moe-Q8_0.gguf (42GB)
├── deepseek-moe-16b-Q2_K.gguf (6.3GB)
├── deepseek-moe-16b-Q4_K_M.gguf (11GB)
└── deepseek-moe-16b-Q8_0.gguf (17GB)
```

### Test Results
```
/home/ubuntu/shimmy/quantization-test-results/
├── *-baseline-run*.json (18 files)
├── *-cpu-offload-run*.json (18 files)
└── SUMMARY.md (generated after tests complete)
```

### Scripts
```
/home/ubuntu/shimmy/
├── quantize-all.sh (quantization script - completed)
├── test-quantized-models.sh (baseline testing - running)
├── quantization-status.sh (progress checker)
└── test-quantized-models.log (live output)
```

### Documentation
```
/home/ubuntu/shimmy/
├── QUANTIZATION-TESTING-PLAN.md (this plan)
├── MODEL-CARD-PLAN.md (card strategy)
└── model-cards/
    ├── TEMPLATE-QUANTIZATION.md (professional template)
    ├── phi-3.5-moe-f16-cpu-offload-README.md (F16 version - uploaded)
    └── deepseek-moe-16b-f16-cpu-offload-README.md (F16 version - uploaded)
```

## 🔬 Technical Notes

### Environment
- **Platform**: Lambda Cloud GH200
- **GPU**: NVIDIA GH200 480GB (96GB VRAM, 480GB RAM)
- **CUDA**: 12.8
- **llama-quantize**: b6686 (CUDA-enabled)
- **shimmy**: v1.6.0 (release build)

### Known Issues
1. **GPT-OSS 20B**: Cannot quantize (pre-quantized MXFP4 by OpenAI)
2. **Metrics**: shimmy generate doesn't output VRAM/TPS/TTFT
3. **Test duration**: Each model load ~30-60s, generation ~20-30s = ~2-3 hours total

### Resolved Issues
✅ Command syntax: shimmy uses `--model-dirs` not `--model`
✅ Model names: Auto-discovered names are lowercase (phi-3.5-moe-q4-k-m not Phi-3.5-MoE-Q4_K_M)
✅ Quantization: All 6 models created successfully

---

**Next Update**: After baseline testing completes (~2-3 hours)
