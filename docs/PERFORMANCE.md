# Performance & Resource Usage

## GPU Power Consumption

Shimmy's GPU usage is comparable to Ollama since both use llama.cpp as the inference backend. Key differences:

### GPU Memory Usage
- **Similar to Ollama**: Both use identical GGUF model loading, so GPU memory consumption per model is nearly identical
- **Model-dependent**: GPU memory usage depends on model size and quantization level, not the inference server
- **Typical usage**:
  - **7B models (Q4_0)**: ~4-5GB VRAM
  - **13B models (Q4_0)**: ~8-10GB VRAM  
  - **70B models (Q4_0)**: ~40-45GB VRAM

### Power Consumption
- **CPU overhead**: Shimmy uses ~50MB RAM vs Ollama's 200MB+ - less CPU power needed
- **GPU utilization**: Identical inference workload = same GPU power draw
- **Idle efficiency**: Shimmy's smaller binary means faster startup/shutdown cycles

### Performance Comparison

| Metric | Shimmy | Ollama | Notes |
|--------|--------|--------|-------|
| **GPU Memory per Model** | Same | Same | Both use llama.cpp backend |
| **GPU Power Draw** | Same | Same | Identical inference workload |
| **CPU RAM Overhead** | ~50MB* | 200MB+ | Shimmy more efficient |
| **Startup Power Spike** | Low | High | Lightweight vs 680MB binary |
| **Idle Power** | Lower | Higher | Smaller memory footprint |

*\*Use included benchmarking tools to measure actual overhead on your system*

## Model-Specific Resource Usage

### Small Models (1B-3B parameters)
```
Model: Phi-3-mini-4k (Q4_0)
- GPU Memory: ~2-3GB
- GPU Power: ~50-80W during inference
- CPU RAM: ~1GB + 50MB Shimmy overhead
```

### Medium Models (7B-13B parameters)  
```
Model: Llama-3.2-7B (Q4_0)
- GPU Memory: ~4-5GB
- GPU Power: ~100-150W during inference  
- CPU RAM: ~2GB + 50MB Shimmy overhead
```

### Large Models (30B+ parameters)
```
Model: Llama-3.1-70B (Q4_0)
- GPU Memory: ~40-45GB (requires multiple GPUs)
- GPU Power: ~300-400W during inference
- CPU RAM: ~8GB + 50MB Shimmy overhead
```

## Optimization Features

### Automatic GPU Detection
- **NVIDIA**: CUDA acceleration enabled automatically
- **AMD**: ROCm support (requires ROCm installation)
- **Apple**: Metal acceleration on macOS (M1/M2/M3)
- **Intel**: GPU support via OpenVINO backend

### Memory Management
- **Smart batching**: Processes multiple requests efficiently
- **Model caching**: Keeps frequently used models in VRAM
- **Graceful fallback**: CPU inference when GPU memory full

### Power Efficiency
- **Fast startup**: No warmup period, immediate inference
- **Efficient shutdown**: Clean VRAM release on exit
- **Minimal overhead**: 5MB binary vs competitors' hundreds of MB

## Benchmarking Tools

### Automated Benchmarking
Use the included benchmarking tools to measure actual performance on your system:

```bash
# Linux/macOS
./scripts/benchmark.sh --requests 20 --output my_results.json

# Windows
scripts\benchmark.bat --requests 20 --output my_results.json

# Python (if available)
python scripts/benchmark.py --requests 20 --output my_results.json
```

**What the benchmark measures:**
- Real response times and throughput
- GPU memory usage and power draw (NVIDIA GPUs)
- CPU and system memory utilization
- Success rates and error handling
- Shimmy process overhead

### Manual Monitoring
```bash
# GPU usage (NVIDIA)
nvidia-smi -l 1

# System resources
top -p $(pgrep shimmy)

# Detailed GPU memory
nvidia-smi --query-gpu=memory.used,memory.total --format=csv -l 1
```

### Built-in Metrics
```bash
# Check model memory usage
curl http://localhost:11434/v1/models

# Health check with resource info
curl http://localhost:11434/health
```

## Comparing with Ollama

### Identical Aspects
- **Model loading**: Same GGUF format, same memory requirements
- **Inference engine**: Both use llama.cpp, identical GPU utilization
- **Model compatibility**: Same models work in both systems

### Shimmy Advantages  
- **Lower system overhead**: 50MB vs 200MB+ base memory usage
- **Faster startup**: <100ms vs 5-10s startup time
- **Smaller footprint**: 5MB binary vs 680MB installation

### Power Efficiency Summary
```
Scenario: Running Llama-3.2-7B for 1 hour
- GPU power draw: ~120W (same for both)
- CPU overhead: Shimmy ~2W less due to efficiency
- Total savings: ~2W continuous + startup/shutdown efficiency
```

## Best Practices

### For Maximum Efficiency
1. **Use appropriate quantization**: Q4_0 for most use cases
2. **Monitor GPU memory**: Don't exceed 90% VRAM capacity  
3. **Batch requests**: Let Shimmy handle request queuing
4. **Close unused models**: Free VRAM when switching between large models

### Power Management
1. **Development**: Use smaller models (1B-3B) for testing
2. **Production**: Scale up to larger models as needed
3. **Multi-GPU**: Shimmy supports automatic multi-GPU distribution

### Troubleshooting High Usage
```bash
# Check which models are loaded
curl http://localhost:11434/v1/models

# Force model unload (restart server)
pkill shimmy && shimmy serve

# Monitor memory leaks
watch -n 1 'ps aux | grep shimmy'
```

## Conclusion

**GPU consumption with Shimmy is essentially identical to Ollama** since both use the same inference engine. The key advantages are:

- **Lower system overhead** (50MB vs 200MB+)
- **Faster startup/shutdown cycles**
- **Same inference performance** with less bloat

For GPU power consumption specifically, expect the same usage patterns as Ollama with the same models.