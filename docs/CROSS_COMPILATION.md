# Cross-Compilation Guide

## ARM64 Linux Build with Docker

When cross-compiling for ARM64 Linux fails in CI/CD due to C++ dependencies (ring crate, llama-cpp), use Docker with QEMU emulation as a reliable alternative.

### Prerequisites

- Docker Desktop with QEMU emulation support
- Windows, macOS, or Linux host system

### Build Command

```bash
docker run --rm --platform linux/arm64 \
  -v "C:\Users\micha\repos\shimmy:/workspace" \
  rust:1.89 \
  bash -c "cd /workspace && cargo build --release --target aarch64-unknown-linux-gnu --no-default-features --features huggingface"
```

### Key Notes

- **Path Format**: Use Windows-style paths with quotes for Docker volume mounts
- **Platform**: `--platform linux/arm64` enables QEMU emulation for ARM64
- **Features**: Use `--no-default-features --features huggingface` to avoid C++ cross-compilation issues
- **Performance**: Slower than native builds due to QEMU emulation, but reliable compilation
- **Output**: Binary will be in `target/aarch64-unknown-linux-gnu/release/shimmy`

### Troubleshooting

If you encounter path issues:
- Ensure proper quote usage around Windows paths
- Use forward slashes in the container path (`:workspace`)
- Verify Docker Desktop is running with QEMU support enabled

### GitHub Actions Alternative

For CI/CD environments where Docker with QEMU is not available, consider temporarily excluding ARM64 from release builds until dedicated ARM64 runners are available.