# Windows GPU Build Guide

This guide provides step-by-step instructions for building Shimmy with GPU acceleration on Windows.

## Prerequisites

### Required Software
- **Visual Studio 2022** with C++ build tools
- **Rust** (latest stable version)
- **Git** for cloning repositories
- **CMake** (for building llama.cpp dependencies)

### GPU-Specific Prerequisites

#### For NVIDIA CUDA
- **CUDA Toolkit 12.0+** (download from NVIDIA)
- Compatible NVIDIA GPU with compute capability 6.0+

#### For OpenCL (AMD/Intel/NVIDIA)
- **OpenCL SDK** or GPU vendor drivers
- Compatible GPU with OpenCL 1.2+ support

#### For Vulkan
- **Vulkan SDK** (download from LunarG)
- Compatible GPU with Vulkan 1.0+ support

## Build Instructions

### 1. Clone Repository

```bash
git clone https://github.com/Michael-A-Kuykendall/shimmy.git
cd shimmy
```

### 2. Choose GPU Backend

#### Option A: NVIDIA CUDA Build
```bash
cargo build --release --features llama-cuda
```

#### Option B: OpenCL Build (AMD/Intel/NVIDIA)
```bash
cargo build --release --features llama-opencl
```

#### Option C: Vulkan Build (Cross-Platform)
```bash
cargo build --release --features llama-vulkan
```

#### Option D: All GPU Backends
```bash
cargo build --release --features gpu
```

### 3. Verify Build

```bash
./target/release/shimmy.exe gpu-info
```

This should show your GPU backend as "available".

## Installation from Source

For permanent installation:

```bash
# Install specific GPU backend
cargo install --path . --features llama-opencl

# Or install all GPU backends
cargo install --path . --features gpu
```

## Troubleshooting

### Missing Template Files Error

**Error**: `couldn't read '..\templates/docker/Dockerfile'`

**Solution**: This indicates you're using an older version. Use the latest from source:
```bash
git clone https://github.com/Michael-A-Kuykendall/shimmy.git
cargo install --path . --features llama-opencl
```

### MoE Method Compilation Errors

**Error**: `no method named 'with_n_cpu_moe' found`

**Solution**: This is from an older published version. The latest source has these methods properly handled.

### CUDA Build Fails

**Common Issues**:
1. **CUDA Toolkit not found**: Ensure CUDA is in your PATH
2. **Compute capability mismatch**: Check your GPU compatibility
3. **Visual Studio version**: Ensure you have VS 2022 with C++ tools

### OpenCL Build Fails

**Common Issues**:
1. **OpenCL headers missing**: Install your GPU vendor's SDK
2. **No OpenCL runtime**: Update your GPU drivers

## Performance Verification

Test your GPU-accelerated build:

```bash
# Check GPU detection
shimmy gpu-info

# Run a simple generation test
shimmy generate test-model --prompt "Hello" --max-tokens 50
```

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