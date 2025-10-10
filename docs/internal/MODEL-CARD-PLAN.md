# Model Card Update Plan

<<<<<<< HEAD
<<<<<<< HEAD
**Status**: Active Plan
=======
**Status**: Active Plan  
>>>>>>> main
=======
**Status**: Active Plan  
>>>>>>> main
**Date**: October 8, 2025

---

## Mission

Update ALL model cards (existing + new quantizations) to match professional, popular HuggingFace model card styles.

---

## Step 1: Research Professional Model Card Styles ✅

Find 3-5 highly popular models with excellent model cards and analyze their structure:

**Target models to study**:
- [x] Meta Llama models (official)
- [x] Microsoft Phi models (official)
- [x] Popular quantization repos (bartowski)

**What we extracted**:
- YAML frontmatter structure (quantized_by, pipeline_tag, license, base_model, tags, language)
- Section organization (Model Details, Download, Usage Examples, Performance)
- Detailed quantization tables (bartowski style)
- Usage examples with collapsible sections
- Multiple tool examples (llama.cpp, Shimmy, Ollama)
- License inheritance patterns

---

## Step 2: Identify All Models Needing Cards

### Existing Models (Already Uploaded)
- [ ] `MikeKuykendall/gpt-oss-20b-cpu-offload-gguf` - Doesn't exist yet
- [x] `MikeKuykendall/phi-3.5-moe-cpu-offload-gguf` - **UPDATED** (professional, accurate)
- [x] `MikeKuykendall/deepseek-moe-16b-cpu-offload-gguf` - **UPDATED** (professional, accurate)

### New Quantizations (To Create)
**GPT-OSS 20B**:
- [ ] Q4_K_M quantization + card
- [ ] Q2_K quantization + card
- [ ] Q8_0 quantization + card

**Phi-3.5-MoE 42B**:
- [ ] Q4_K_M quantization + card
- [ ] Q2_K quantization + card
- [ ] Q8_0 quantization + card

**DeepSeek MoE 16B**:
- [ ] Q4_K_M quantization + card
- [ ] Q2_K quantization + card
- [ ] Q8_0 quantization + card

**Total**: 3 existing cards to update + 9 new quantizations with cards = **12 model cards**

---

## Step 3: Create Model Card Template ✅

Based on research, create a standardized template with:
- [x] YAML frontmatter (tags, license, metrics, etc.)
- [x] Model overview and details
- [x] Quantization details (for quant cards)
- [x] Usage examples (llama.cpp, Shimmy with MoE offloading)
- [x] Performance notes
- [x] Citation
- [x] License

**Template**: `/home/ubuntu/shimmy/model-cards/TEMPLATE-QUANTIZATION.md`

---

## Step 4: Execute Quantizations

**Tool**: `~/llama-cpp-rs/llama-cpp-sys-2/llama.cpp/build/bin/llama-quantize`

**For each model**:
1. Quantize F16 → Q4_K_M, Q2_K, Q8_0
2. Measure file sizes
3. Create model card from template
4. Upload to HuggingFace with `huggingface-cli upload`

---

## Step 5: Update Existing Model Cards

For the 3 CPU offload models already uploaded:
1. Fetch current card with `huggingface-cli download`
2. Rewrite using new professional template
3. Upload updated card

---

## Workflow Commands

### Research Phase
```bash
# Download example model cards for study
huggingface-cli download meta-llama/Llama-3.2-3B --include "README.md" --local-dir /tmp/llama-card
huggingface-cli download microsoft/Phi-3.5-MoE-instruct --include "README.md" --local-dir /tmp/phi-card
```

### Quantization Phase
```bash
# Example quantization command
~/llama-cpp-rs/llama-cpp-sys-2/llama.cpp/build/bin/llama-quantize \
  /home/ubuntu/models/gpt-oss-20b-f16.gguf \
  /home/ubuntu/models/gpt-oss-20b-Q4_K_M.gguf \
  Q4_K_M
```

### Upload Phase
```bash
# Create repo (if new)
huggingface-cli repo create MikeKuykendall/gpt-oss-20b-Q4_K_M-gguf --type model

# Upload files
huggingface-cli upload MikeKuykendall/gpt-oss-20b-Q4_K_M-gguf \
  /home/ubuntu/models/gpt-oss-20b-Q4_K_M.gguf \
  gpt-oss-20b-Q4_K_M.gguf

# Upload model card
huggingface-cli upload MikeKuykendall/gpt-oss-20b-Q4_K_M-gguf \
  /home/ubuntu/shimmy/model-cards/gpt-oss-20b-Q4_K_M-README.md \
  README.md
```

---

## Success Criteria

- [ ] All 12 models have professional model cards matching top-tier HF repos
- [ ] Cards include accurate file sizes, quantization details, usage examples
- [ ] YAML frontmatter properly formatted for HF discovery
- [ ] All quantizations successfully uploaded and accessible

---

## Notes

- Use llama-quantize locally (already built, CUDA-enabled)
- Use HF CLI for uploads (already authenticated as MikeKuykendall)
- Model cards are markdown files named `README.md` in the repo root
- Study the best, duplicate their style, improve where possible
