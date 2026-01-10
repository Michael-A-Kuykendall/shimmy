# Cross-Platform Vision Testing with Docker

This directory contains Docker configurations for testing Shimmy Vision features across different platforms before release.

## Overview

Since vision features require GPU acceleration and licensing, we need a systematic way to test binaries on all target platforms. This setup uses Docker containers to simulate different environments and run the vision test suite.

## Supported Platforms

- **Linux x86_64** (Ubuntu 22.04 with CUDA support)
- **Linux ARM64** (Ubuntu 22.04)
- **Windows x86_64** (via Wine in Docker)
- **macOS Intel** (via Darling or cross-compilation testing)
- **macOS ARM64** (Apple Silicon - requires external testing)

## Quick Start

```bash
# Build and test Linux x86_64 with CUDA
docker build -f docker/Dockerfile.vision-test-linux-cuda -t shimmy-vision-test-linux-cuda .
docker run --gpus all -v $(pwd):/workspace shimmy-vision-test-linux-cuda

# Build and test Linux ARM64
docker build -f docker/Dockerfile.vision-test-linux-arm64 -t shimmy-vision-test-linux-arm64 .
docker run -v $(pwd):/workspace shimmy-vision-test-linux-arm64

# Build and test Windows x86_64 (via Wine)
docker build -f docker/Dockerfile.vision-test-windows -t shimmy-vision-test-windows .
docker run -v $(pwd):/workspace shimmy-vision-test-windows
```

## Test Results

Each test container will:
1. Build Shimmy with vision features for the target platform
2. Download and cache vision models
3. Run the vision test suite (`scripts/test_cross_compiled_vision.py`)
4. Generate a test report with performance metrics
5. Validate license enforcement

## CI/CD Integration

These containers can be used in GitHub Actions for automated cross-platform testing:

```yaml
- name: Test Vision on Linux CUDA
  run: |
    docker build -f docker/Dockerfile.vision-test-linux-cuda -t test .
    docker run --gpus all test

- name: Test Vision on Linux ARM64
  run: |
    docker build -f docker/Dockerfile.vision-test-linux-arm64 -t test .
    docker run test
```

## Manual Testing Checklist

Before going live, ensure all platforms pass:

- [ ] Linux x86_64 CUDA (GPU acceleration)
- [ ] Linux ARM64 (CPU-only)
- [ ] Windows x86_64 (Vulkan GPU)
- [ ] macOS Intel (MLX if available)
- [ ] macOS ARM64 (MLX GPU - external testing required)

## Test Images

The test suite uses these standard images:
- `assets/vision-samples/final-test.png` - Main test image
- `assets/vision-samples/ocr-test.png` - OCR accuracy test
- `assets/vision-samples/layout-test.png` - Layout analysis test

## Performance Baselines

Expected performance (with GPU):
- Image loading: < 2 seconds
- Model inference: 4-15 seconds
- Total processing: < 30 seconds

CPU-only performance will be 10-50x slower and should be flagged as warnings.</content>
<parameter name="filePath">c:\Users\micha\repos\shimmy-workspace\packaging\docker\README.md