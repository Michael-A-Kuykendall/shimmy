# MoE CPU Offloading - Complete Validation Checklist
<<<<<<< HEAD
<<<<<<< HEAD
**Date**: October 8, 2025
=======
**Date**: October 8, 2025  
>>>>>>> main
=======
**Date**: October 8, 2025  
>>>>>>> main
**Mission**: Systematic validation and benchmarking of all 3 MoE models with complete metrics for whitepaper

---

## Prerequisites

### Tools Installation
- [ ] Install `jq` for JSON parsing
- [ ] Install `bc` for floating point calculations (or use Python alternative)
- [ ] Verify `curl` available
- [ ] Verify shimmy server running with `--cpu-moe` flag

### Model Downloads
- [ ] **Phi-3.5-MoE 41.9B** - `/home/ubuntu/models/phi-3.5-moe-f16.gguf` (79GB)
- [ ] **GPT-OSS 20B** - Download from HuggingFace
- [ ] **DeepSeek MoE 16B** - Download from HuggingFace

---

## Model 1: Phi-3.5-MoE 41.9B

### Architecture Verification
- [ ] Model loads successfully
- [ ] Expert count confirmed: 16 experts
- [ ] Active experts per token: 2
- [ ] Total parameters: 41.87B
- [ ] Context length: 131K tokens
- [ ] All 96 expert tensors offloaded to CPU (32 layers × 3 types)

### Performance Benchmarks
- [ ] **Test 1 - Code Generation** (fibonacci function)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Valid code with proper logic
- [ ] **Test 2 - Math Reasoning** (train speed problem)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Correct step-by-step math
- [ ] **Test 3 - Creative Writing** (Emily Dickinson poem)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Stylistically appropriate
- [ ] **Test 4 - Technical Writing** (gradient descent explanation)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Accurate and clear

### Streaming Validation
- [ ] **Streaming Test** (code generation)
  - [ ] Verify clean SSE token delivery
  - [ ] Check for token fragmentation issues
  - [ ] Measure approximate TTFT

### Memory Metrics
- [ ] Record GPU VRAM usage
- [ ] Record CPU RAM usage
- [ ] Calculate VRAM savings percentage

### Summary Metrics for Whitepaper
- [ ] Average TPS across all tests
- [ ] Model load time
- [ ] Memory footprint (GPU/CPU split)
- [ ] Quality assessment summary

---

## Model 2: GPT-OSS 20B

### Model Setup
- [ ] Download `gpt-oss-20b-f16.gguf` from HuggingFace
- [ ] Verify file size (~13.8GB expected)
- [ ] Confirm shimmy can discover model

### Architecture Verification
- [ ] Model loads successfully
- [ ] Expert count confirmed: 32 experts
- [ ] Active experts per token: 4
- [ ] Total parameters: 20B
- [ ] Context length: 131K tokens
- [ ] All expert tensors offloaded to CPU

### Performance Benchmarks
- [ ] **Test 1 - Code Generation** (fibonacci function)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Valid code with proper logic
- [ ] **Test 2 - Math Reasoning** (train speed problem)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Correct step-by-step math
- [ ] **Test 3 - Creative Writing** (Emily Dickinson poem)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Stylistically appropriate
- [ ] **Test 4 - Technical Writing** (gradient descent explanation)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Accurate and clear

### Streaming Validation
- [ ] **Streaming Test** (code generation)
  - [ ] Verify clean SSE token delivery
  - [ ] Check for token fragmentation issues
  - [ ] Measure approximate TTFT

### Memory Metrics
- [ ] Record GPU VRAM usage
- [ ] Record CPU RAM usage
- [ ] Calculate VRAM savings percentage

### Summary Metrics for Whitepaper
- [ ] Average TPS across all tests
- [ ] Model load time
- [ ] Memory footprint (GPU/CPU split)
- [ ] Quality assessment summary

---

## Model 3: DeepSeek MoE 16B

### Model Setup
- [ ] Download `deepseek-moe-16b-f16.gguf` from HuggingFace
- [ ] Verify file size (~32.8GB expected)
- [ ] Confirm shimmy can discover model

### Architecture Verification
- [ ] Model loads successfully
- [ ] Expert count confirmed: 64 regular + 2 shared experts
- [ ] Active experts per token: 6
- [ ] Total parameters: 16.38B
- [ ] Context length: 4K tokens
- [ ] All expert tensors offloaded to CPU (dual architecture)

### Performance Benchmarks
- [ ] **Test 1 - Code Generation** (fibonacci function)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Valid code with proper logic
- [ ] **Test 2 - Math Reasoning** (train speed problem)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Correct step-by-step math
- [ ] **Test 3 - Creative Writing** (Emily Dickinson poem)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Stylistically appropriate
- [ ] **Test 4 - Technical Writing** (gradient descent explanation)
  - [ ] Run non-streaming test
  - [ ] Capture: Total time, tokens generated, TPS
  - [ ] Quality check: Accurate and clear

### Streaming Validation
- [ ] **Streaming Test** (code generation)
  - [ ] Verify clean SSE token delivery
  - [ ] Check for token fragmentation issues
  - [ ] Measure approximate TTFT

### Memory Metrics
- [ ] Record GPU VRAM usage
- [ ] Record CPU RAM usage
- [ ] Calculate VRAM savings percentage

### Summary Metrics for Whitepaper
- [ ] Average TPS across all tests
- [ ] Model load time
- [ ] Memory footprint (GPU/CPU split)
- [ ] Quality assessment summary

---

## Whitepaper Updates

### Performance Metrics Table
- [ ] Update with actual TPS for all models
- [ ] Update with actual TTFT estimates
- [ ] Update with actual memory measurements
- [ ] Remove all "TBD" placeholders

### Benchmark Results Section
- [ ] Document all test results in tables
- [ ] Include quality assessments
- [ ] Add comparative analysis across models
- [ ] Note any performance differences by architecture

### Evidence Documentation
- [ ] Screenshot/logs of expert tensor offloading
- [ ] Memory usage charts or logs
- [ ] Sample outputs from quality tests
- [ ] Performance comparison graphs

---

## Final Validation

- [ ] All three models tested with identical protocol
- [ ] All performance metrics captured
- [ ] Whitepaper fully updated with real data
- [ ] No "TBD" or placeholder values remain
- [ ] Ready for upstream contribution consideration

---

## Notes Section

### Phi-3.5-MoE 41.9B
```
[Record observations, issues, notable findings here]
```

### GPT-OSS 20B
```
[Record observations, issues, notable findings here]
```

### DeepSeek MoE 16B
```
[Record observations, issues, notable findings here]
```
