# MoE CPU Offloading Benchmark Evidence

<<<<<<< HEAD
<<<<<<< HEAD
**Date**: October 8, 2025
=======
**Date**: October 8, 2025  
>>>>>>> main
=======
**Date**: October 8, 2025  
>>>>>>> main
**Purpose**: Raw benchmark data and logs for audit verification

## Contents

### Streaming vs Non-Streaming Benchmarks

- **phi35-streaming-bench.log** - Phi-3.5-MoE 41.9B performance comparison
- **gpt-oss-streaming-bench.log** - GPT-OSS 20B performance comparison
- **deepseek-streaming-bench.log** - DeepSeek MoE 16B performance comparison

Each log contains:
- 4 test prompts (short, medium, long, very long)
- Non-streaming TPS measurements
- Streaming TPS measurements with actual token counts
- TTFT (Time To First Token) estimates
- Performance delta calculations

### Model Loading and Offloading Logs

- **shimmy-phi35.log** - Phi-3.5-MoE server startup with CPU offloading
- **shimmy-gpt-oss.log** - GPT-OSS server startup with CPU offloading
- **shimmy-deepseek.log** - DeepSeek server startup with CPU offloading

Each log contains:
- Model architecture detection (expert count, active experts)
- Expert tensor CPU offloading confirmation
- Memory distribution (GPU vs CPU allocation)
- Context configuration

## Verification

These logs provide evidence for claims in the MoE CPU Offloading White Paper:

1. **Expert Detection**: Search for `expert_count` and `expert_used_count` in loading logs
2. **CPU Offloading**: Search for `CUDA_Host` buffer overrides in loading logs
3. **Memory Savings**: Search for `CPU_Mapped` and `CUDA0 model buffer size` in loading logs
4. **Performance Data**: Raw TPS and TTFT measurements in streaming-bench logs

## Reproduction

To reproduce these results:

```bash
# Start shimmy server with CPU offloading
cd /home/ubuntu/shimmy
SHIMMY_BASE_GGUF=/path/to/model.gguf \
  ./target/release/shimmy serve --bind 127.0.0.1:11435 --cpu-moe > server.log 2>&1 &

# Run streaming benchmark
./scripts/benchmark-moe-streaming.sh <model-name> > benchmark.log

# Compare results with evidence files in this directory
```

## File Integrity

| File | Size | Date | Purpose |
|------|------|------|---------|
| phi35-streaming-bench.log | 2.6K | Oct 8, 2025 | Phi-3.5 benchmarks |
| gpt-oss-streaming-bench.log | 2.6K | Oct 8, 2025 | GPT-OSS benchmarks |
| deepseek-streaming-bench.log | 2.5K | Oct 8, 2025 | DeepSeek benchmarks |
| shimmy-phi35.log | 414K | Oct 8, 2025 | Phi-3.5 loading logs |
| shimmy-gpt-oss.log | 431K | Oct 8, 2025 | GPT-OSS loading logs |
| shimmy-deepseek.log | 698K | Oct 8, 2025 | DeepSeek loading logs |

---
*Evidence preserved for audit verification and reproducibility*
