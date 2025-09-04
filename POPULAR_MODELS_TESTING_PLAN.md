# Popular GGUF Models Testing Plan

## Top 5 Most Popular GGUF Models for Testing

Based on community usage, downloads, and development patterns, here are the top GGUF models to test:

### 1. Microsoft Phi-3 Mini (3.8B) - **PRIMARY TARGET**
- **Why**: Fastest, smallest, most developer-friendly
- **Size**: ~2.3GB (Q4_K_M), ~4.2GB (F16)
- **Use Case**: Code completion, chat, lightweight inference
- **Download**: `microsoft/Phi-3-mini-4k-instruct-gguf`
- **Template**: ChatML

### 2. Meta Llama-3.2 (3B) - **SECONDARY TARGET**  
- **Why**: Latest Meta model, excellent performance
- **Size**: ~1.9GB (Q4_K_M), ~3.4GB (F16)
- **Use Case**: General purpose, instruction following
- **Download**: `meta-llama/Llama-3.2-3B-Instruct-GGUF`
- **Template**: Llama3

### 3. Mistral 7B Instruct v0.3 (7B) - **PERFORMANCE TARGET**
- **Why**: Gold standard for 7B models, excellent reasoning
- **Size**: ~4.1GB (Q4_K_M), ~7.8GB (F16)
- **Use Case**: Complex reasoning, high-quality responses
- **Download**: `mistralai/Mistral-7B-Instruct-v0.3-GGUF`
- **Template**: OpenChat

### 4. Google Gemma-2 (2B/9B) - **EFFICIENCY TARGET**
- **Why**: Google's latest, excellent efficiency
- **Size**: 2B: ~1.4GB (Q4_K_M), 9B: ~5.4GB (Q4_K_M)
- **Use Case**: Balanced performance/efficiency
- **Download**: `google/gemma-2-2b-it-GGUF` or `google/gemma-2-9b-it-GGUF`
- **Template**: ChatML

### 5. Alibaba Qwen2.5 (7B) - **MULTILINGUAL TARGET**
- **Why**: Best multilingual support, coding capabilities
- **Size**: ~4.2GB (Q4_K_M), ~7.9GB (F16)
- **Use Case**: Multilingual, code generation
- **Download**: `Qwen/Qwen2.5-7B-Instruct-GGUF`
- **Template**: ChatML

## Test Matrix

| Model | Size (Q4_K_M) | Local Test | Cloud Test | Priority |
|-------|---------------|------------|------------|----------|
| Phi-3 Mini 3.8B | ~2.3GB | ✅ Local | N/A | HIGH |
| Llama-3.2 3B | ~1.9GB | ✅ Local | N/A | HIGH |
| Mistral 7B | ~4.1GB | ✅ Local | N/A | MEDIUM |
| Gemma-2 2B | ~1.4GB | ✅ Local | N/A | MEDIUM |
| Gemma-2 9B | ~5.4GB | ✅ Local | ☁️ Cloud | LOW |
| Qwen2.5 7B | ~4.2GB | ✅ Local | N/A | MEDIUM |

## Testing Framework

### Phase 1: Download & Verify
```bash
# Create models directory
mkdir -p ./test-models

# Download using HuggingFace Hub
# We'll create a download script for each model
```

### Phase 2: Shimmy Integration Tests
```bash
# Test model discovery
shimmy discover

# Test model loading
shimmy probe <model-name>

# Test generation
shimmy generate <model-name> --prompt "Hello, I am testing shimmy" --max-tokens 50

# Test server mode
shimmy serve --bind 127.0.0.1:11435
```

### Phase 3: API Compatibility Tests
```bash
# Test HTTP API
curl -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{"model":"<model-name>","prompt":"Test","max_tokens":10}'

# Test OpenAI API compatibility
curl -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"<model-name>","messages":[{"role":"user","content":"Hello"}]}'
```

### Phase 4: Performance Benchmarks
```bash
# Throughput test
shimmy bench <model-name> --max-tokens 100

# Memory usage monitoring
# Startup time measurement
# Concurrent request handling
```

## Implementation Steps

### Step 1: Create Download Manager
- HuggingFace Hub integration
- Automatic GGUF file detection
- Progress tracking for large downloads

### Step 2: Enhanced Model Discovery
- Automatic template detection based on model family
- Parameter count inference from filename
- Quantization format detection

### Step 3: Comprehensive Test Suite
- Unit tests for each model family
- Integration tests for API endpoints
- Performance regression tests

### Step 4: Validation Reports
- Model compatibility matrix
- Performance benchmarks per model
- Memory usage profiling

## Expected System Requirements

### Minimum for All Tests (Local)
- **RAM**: 16GB (to handle 7B Q4_K_M models)
- **Storage**: 25GB free space for all models
- **CPU**: Modern x64 processor

### Cloud Testing (if needed)
- **Instance**: 32GB RAM, 50GB storage
- **Models**: Gemma-2 9B, larger Qwen variants
- **Duration**: 2-4 hours for full test suite

## Success Criteria

### ✅ Model Loading
- All 5 models load without errors
- Proper template detection
- Reasonable memory usage

### ✅ Generation Quality
- Coherent responses for standard prompts
- Proper tokenization and formatting
- No crashes during generation

### ✅ API Compatibility
- HTTP API works for all models
- OpenAI compatibility maintained
- WebSocket streaming functional

### ✅ Performance
- Startup time < 10 seconds per model
- Generation speed > 1 token/second
- Memory usage within expected bounds

This testing plan ensures Shimmy works reliably with the models developers actually use in production, covering the spectrum from lightweight (Phi-3) to powerful (Mistral 7B) to specialized (multilingual Qwen).
