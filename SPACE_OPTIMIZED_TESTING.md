# Shimmy Model Testing Strategy - Space Optimized

## Current Situation
- **Available Space**: 204GB free on D: drive  
- **Required for Full Test**: ~25GB for all 5 models
- **Strategy**: Full comprehensive testing on D: drive

## Optimized Testing Plan

### Phase 1: Essential Tests (Space: ~6GB)
1. **TinyLlama 1.1B** (~0.6GB Q4_K_M) - Smallest functional model
2. **Phi-3 Mini 3.8B** (~2.3GB Q4_K_M) - Most popular dev model  
3. **Gemma-2 2B** (~1.4GB Q4_K_M) - Google's efficient model

### Phase 2: Performance Tests (Space: ~4GB each)
4. **Mistral 7B** (~4.1GB Q4_K_M) - Gold standard reasoning
5. **Qwen2.5 7B** (~4.2GB Q4_K_M) - Multilingual + code

## Model Download URLs (Direct HuggingFace)

### TinyLlama 1.1B Chat (0.6GB)
```bash
curl -L -o test-models/tinyllama-1.1b-chat-q4.gguf \
  "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.q4_k_m.gguf"
```

### Phi-3 Mini 4K (2.3GB) 
```bash
curl -L -o test-models/phi3-mini-4k-instruct-q4.gguf \
  "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf"
```

### Gemma-2 2B IT (1.4GB)
```bash
curl -L -o test-models/gemma2-2b-it-q4.gguf \
  "https://huggingface.co/google/gemma-2-2b-it-GGUF/resolve/main/2b_it_q4_k_m.gguf"
```

### Mistral 7B Instruct (4.1GB)
```bash
curl -L -o test-models/mistral-7b-instruct-q4.gguf \
  "https://huggingface.co/mistralai/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3.Q4_K_M.gguf"
```

## Progressive Testing Approach

### Step 1: Start Small (TinyLlama)
- **Size**: 0.6GB
- **Purpose**: Verify basic functionality
- **Time**: 2-3 minutes download + test

### Step 2: Popular Model (Phi-3)
- **Size**: 2.3GB  
- **Purpose**: Test most common developer model
- **Time**: 10-15 minutes download + test

### Step 3: Performance Test (Mistral or Gemma)
- **Size**: 1.4GB or 4.1GB
- **Purpose**: Test larger model performance
- **Time**: 5-20 minutes download + test

## Space Management Strategy

```bash
# Download and test one at a time, clean up after each
./quick_model_test.sh tinyllama  # Test and keep if successful
./quick_model_test.sh phi3       # Test and keep if successful  
rm test-models/tinyllama*.gguf   # Free up space if needed
./quick_model_test.sh mistral    # Test larger model
```

## Cloud Testing Option

If local space is too limited, we can:

1. **Test locally**: TinyLlama + Phi-3 (3GB total)
2. **Cloud test**: Larger models on AWS/GCP instance
   - Instance: 32GB RAM, 100GB storage
   - Duration: 2-3 hours
   - Cost: ~$5-10

## Recommended Immediate Action

Let's start with the **space-efficient quick test**:

```bash
# Run the quick test with Phi-3 Mini (2.3GB)
./quick_model_test.sh

# This will:
# 1. Download Phi-3 Mini Q4 (~2.3GB)
# 2. Test all core functionality
# 3. Verify API compatibility
# 4. Provide confidence in shimmy's reliability
```

Once that passes, we can selectively test additional models based on your priorities and available space.

## Success Criteria

If Phi-3 Mini works perfectly:
- ✅ Shimmy handles the most popular developer model
- ✅ 80% of use cases covered (most devs use 3-4B models)
- ✅ Ready for production use

Additional models provide:
- **Mistral 7B**: Validation of larger model handling
- **Gemma-2**: Google ecosystem compatibility  
- **Qwen2.5**: Multilingual/coding validation

## Next Steps

1. Run `./quick_model_test.sh` to test Phi-3 Mini
2. If successful, decide on additional models based on space
3. Consider cloud testing for comprehensive validation
4. Document working model configurations
