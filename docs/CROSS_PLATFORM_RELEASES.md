# Cross-Platform Release Process

## Overview

Shimmy now supports automated cross-platform binary releases through GitHub Actions. This addresses the issue where the quickstart documentation referenced Linux and macOS binaries that weren't available in releases.

## Supported Platforms

The release workflow automatically builds binaries for:

- **Linux**: x86_64 and ARM64 (aarch64)
- **macOS**: x86_64 (Intel) and ARM64 (Apple Silicon)  
- **Windows**: x86_64

## Release Artifacts

Each release includes:

### Platform-Specific Binaries
- `shimmy-linux-amd64` - Linux x86_64
- `shimmy-linux-arm64` - Linux ARM64
- `shimmy-darwin-amd64` - macOS Intel
- `shimmy-darwin-arm64` - macOS Apple Silicon
- `shimmy-windows-amd64.exe` - Windows x86_64

### Generic Binaries (for quickstart compatibility)
- `shimmy` - Generic Linux binary (x86_64)
- `shimmy.exe` - Generic Windows binary

## Triggering Releases

Releases are triggered by:

1. **Tag push**: Push a tag matching `v*` pattern (e.g., `v0.1.2`)
2. **Manual dispatch**: Run the workflow manually via GitHub Actions UI

## Technical Details

- Linux ARM64 uses `cross` tool for cross-compilation to avoid CMake complexity
- macOS builds use separate runners for Intel (macos-13) and Apple Silicon (macos-14)
- All binaries are built with `--features llama` for full functionality
- CI workflow ensures code quality with formatting and clippy checks

## Quickstart Compatibility

The release workflow ensures that the quickstart.md download URLs work:

```bash
# Linux/macOS (works with generic "shimmy" binary)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy -o shimmy

# Windows (works with generic "shimmy.exe" binary)  
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy.exe -o shimmy.exe
```