# Changelog

All notable changes to Shimmy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.2] - 2025-09-12

### 🐛 Bug Fixes

**Issue #13: VSCode Integration with Qwen Models**
- Fixed VSCode extension compatibility with Qwen3-4B-Instruct and other Qwen models
- Enhanced automatic template detection for Qwen models (now uses ChatML template)
- Added better error logging for model loading failures in OpenAI-compatible API
- Improved error handling with detailed diagnostics for troubleshooting

**Issue #12: Custom Model Directory Detection** 
- Added support for custom model directories via `SHIMMY_MODEL_PATHS` environment variable
- Added support for `OLLAMA_MODELS` environment variable for Ollama model directories
- Added `--model-dirs` global command-line option for specifying custom directories
- Enhanced Windows multi-drive search for Ollama installations (C:, D:, E:, F: drives)
- Improved model auto-discovery to handle Ollama installs on different drives

### ✨ Enhancements

- **Multi-Drive Support**: Automatic scanning of common Ollama paths across multiple Windows drives
- **Template Detection**: Enhanced model template inference with better support for:
  - Qwen models → ChatML template
  - ChatGLM models → ChatML template  
  - Llama models → Llama3 template
  - Improved fallback to OpenChat template
- **Error Handling**: Added comprehensive error logging for debugging model loading issues
- **CLI Improvements**: New global `--model-dirs` option works with all commands

### 🛠️ Developer Experience

- Added comprehensive regression testing suite
- Fixed missing `discover_models_from_directory` function for benchmarking
- Enhanced error messages with model-specific context
- Improved code documentation and examples

### 📖 Documentation

**Issue #15: Homebrew Formula Improvements**
- Created improved Homebrew formula using pre-built binaries instead of source compilation  
- Generated installation script for faster Homebrew installations
- Provided migration path from source-based to binary-based Homebrew formula

### 🎯 Usage Examples

**Custom Model Directories:**
```bash
# Environment variables
export SHIMMY_MODEL_PATHS="D:\models;E:\ollama\models"
export OLLAMA_MODELS="F:\MyOllama\models"

# Command line options
shimmy --model-dirs "D:\models;E:\ollama\models" serve
shimmy --model-dirs "/path/to/models" list
```

**VSCode Integration:**
- Qwen3-4B-Instruct models now work seamlessly with VSCode extensions
- Improved error reporting for troubleshooting integration issues

### 🔧 Technical Details

- Enhanced `ModelDiscovery` and `ModelAutoDiscovery` systems
- Improved OpenAI API compatibility layer
- Better template selection algorithm  
- Comprehensive Windows drive scanning
- Added regression testing infrastructure

## [0.1.0] - 2025-09-02

### Added
- **Initial release of Shimmy** - The 5MB alternative to Ollama
- **Core inference engine** with llama.cpp backend integration
- **Full OpenAI API compatibility**:
  - `POST /v1/chat/completions` - OpenAI-compatible chat endpoint
  - `GET /v1/models` - List available models
- **Native Shimmy API**:
  - `POST /api/generate` - JSON generation with optional SSE streaming
  - `GET /ws/generate` - WebSocket streaming generation
  - `GET /health` - Health check endpoint
  - `GET /api/models` - Native model listing
- **CLI commands**:
  - `shimmy serve` - Start the inference server
  - `shimmy list` - List available models
  - `shimmy discover` - Discover models in filesystem
  - `shimmy generate` - Command-line text generation
  - `shimmy probe` - Test model loading
- **Model format support**:
  - GGUF models via llama.cpp integration
  - SafeTensors detection and guidance
  - Auto-discovery from filesystem
- **Template system**:
  - ChatML template support
  - Llama3 template support  
  - OpenChat template support
- **Cross-platform support**:
  - Linux (x86_64, ARM64)
  - Windows (x86_64)
  - macOS (x86_64, ARM64)
- **Performance optimizations**:
  - 5.1MB single binary size
  - <100ms startup time
  - <50MB memory overhead
  - Release build with LTO and size optimization
- **Integration guides**:
  - VSCode Copilot configuration
  - Continue.dev setup
  - Cursor IDE integration
  - Generic OpenAI API client configuration
- **Package distribution**:
  - GitHub Releases (direct binary downloads)
  - crates.io (Rust package manager)
  - npm (Node.js wrapper package)
  - Docker Hub (container images)
  - PyPI (Python wrapper package)
- **Development infrastructure**:
  - Comprehensive test suite (27 unit tests + 4 integration tests)
  - GitHub Actions CI/CD pipeline
  - Cross-platform build automation
  - Multi-package-manager release automation
- **Documentation**:
  - Complete API documentation
  - Quick start guide (30-second setup)
  - Integration examples
  - Performance benchmarks
  - Architecture documentation

### Technical Details
- **Language**: Rust 2021 edition
- **Dependencies**: tokio, axum, llama-cpp-2, serde, clap
- **Features**: Optional `llama` feature for actual inference
- **License**: MIT (free forever)
- **Minimum supported Rust version**: 1.70+

### Performance Metrics
- **Binary size**: 5.1MB (vs Ollama's 680MB)
- **Startup time**: <100ms (vs Ollama's 5-10s)
- **Memory usage**: <50MB baseline (vs Ollama's 200MB+)
- **API compatibility**: 100% OpenAI compatibility (vs Ollama's partial)

### Free Forever Commitment
Shimmy is committed to being free forever with no asterisks, no "free for now" periods, and no pivot to paid services. The MIT license ensures this commitment is legally binding.

[Unreleased]: https://github.com/Michael-A-Kuykendall/shimmy/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Michael-A-Kuykendall/shimmy/releases/tag/v0.1.0
