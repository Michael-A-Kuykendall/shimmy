# Windows GPU Build Guide

This guide covers GPU acceleration on Windows for Shimmy v2.0 (Airframe engine).

## v2.0: Airframe (wgpu) — No Manual Setup Required

Shimmy v2.0 uses **Airframe**, a pure-Rust WebGPU engine. On Windows, wgpu selects the best
available adapter automatically (D3D12 on NVIDIA/AMD/Intel, Vulkan as fallback). No CUDA SDK,
no Vulkan SDK, and no C++ compiler needed.

### Download the Binary (Recommended)

```bash
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe
set SHIMMY_BASE_GGUF=C:\path\to\model.gguf
.\shimmy.exe serve
```

### Check GPU Adapter

```bash
.\shimmy.exe gpu-info
```

### Build from Source with Airframe

Requires the airframe submodule (initialize with `--recurse-submodules`):

```bash
git clone https://github.com/Michael-A-Kuykendall/shimmy --recurse-submodules
cd shimmy
cargo build --release --features airframe,huggingface
```

## Extended Context

```bash
set SHIMMY_BASE_GGUF=C:\path\to\model.gguf
set SHIMMY_MAX_CTX=8192
.\shimmy.exe serve
```

---

## Legacy: llama.cpp GPU Backends (v1.x — use `--legacy` flag)

The llama.cpp CUDA/Vulkan/OpenCL backends are still available in v2.0 via the `--legacy` flag.
They require their respective SDKs. This section is preserved for reference.

### Prerequisites (legacy path only)

#### For NVIDIA CUDA
- **CUDA Toolkit 12.0+**
- Compatible NVIDIA GPU with compute capability 6.0+

#### For OpenCL (AMD/Intel/NVIDIA)
- **OpenCL SDK** or GPU vendor drivers

#### For Vulkan
- **Vulkan SDK** (LunarG)

### Build Instructions (legacy path)

```bash
# CUDA
cargo build --release --features llama-cuda

# OpenCL
cargo build --release --features llama-opencl

# Vulkan
cargo build --release --features llama-vulkan
```

Use with `--legacy` flag to activate:
```bash
.\shimmy.exe serve --legacy --model-path C:\path\to\model.gguf --gpu-backend cuda
```

### Troubleshooting (legacy)

- **CUDA not found**: Ensure `nvcc` is in PATH
- **OpenCL headers missing**: Install GPU vendor SDK
- **Vulkan SDK missing**: Install from LunarG


## Binary Distribution

Pre-built Windows binaries with GPU support are available in GitHub Releases:
- Download from: https://github.com/Michael-A-Kuykendall/shimmy/releases
- Choose the appropriate GPU variant for your system

## Support

If you encounter issues:
1. Check the [main README](../README.md) for general troubleshooting
2. Review [CUDA documentation](../docs/GPU_ARCHITECTURE_DECISION.md) for GPU-specific details
3. Open an issue at: https://github.com/Michael-A-Kuykendall/shimmy/issues

## Version Compatibility

- **v1.7.2+**: Full Windows GPU support with templates included
- **v1.7.1 and earlier**: May have template packaging or MoE compilation issues
- **Always use latest**: `git clone` and build from source for best experience