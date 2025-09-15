# Changelog

All notable changes to Shimmy will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Opt-in Usage Analytics**: Anonymous business intelligence collection system
- **Performance Benchmarking Tools**: Cross-platform scripts for real GPU/CPU measurement
- **Comprehensive Security Policy**: Private vulnerability disclosure process (SECURITY.md)
- **DCO (Developer Certificate of Origin)**: Legal compliance for all contributions
- **Professional GitHub Templates**: Issue/PR templates with structured workflows
- **Branch Protection**: Quality gates with CI and DCO enforcement
- **Automated Changelog**: CI/CD integration for release documentation

### Changed
- **Enhanced CONTRIBUTING.md**: Added maintainer process and DCO requirements
- **Improved Documentation**: Comprehensive performance analysis and metrics transparency
- **Professional Repository Structure**: Security-first approach with industry standards

### Security
- **Private Security Disclosure**: GitHub Security Advisories integration
- **DCO Compliance**: All contributions legally certified
- **Branch Protection**: Enforced code review and quality gates

### Documentation
- **Performance Analysis**: Real benchmarking tools and GPU consumption data
- **Metrics Transparency**: Complete disclosure of business intelligence collection
- **Contributing Guidelines**: Clear maintainer process and legal requirements

## [1.3.3] - 2025-09-15

### ✨ Features

**Docker Compose Deployment Support**
- Added complete Docker Compose configuration for production deployments
- Includes Nginx reverse proxy and health checks
- Railway, Render, and Fly.io deployment configurations
- Production-ready containerization

### 🐛 Bug Fixes

**Issue #22: Windows EXE Availability**  
- Fixed missing Windows executable in GitHub releases
- Added `shimmy-windows-x86_64.exe` to all releases for direct download
- Improved Windows installation documentation for libclang.dll dependency

**ARM64 Linux Cross-Compilation Issues**
- Resolved OpenSSL cross-compilation failures for ARM64 Linux builds
- Switched to rustls for better cross-compilation compatibility
- Added Docker-based ARM64 Linux build process using QEMU emulation
- Temporarily excluded ARM64 Linux from CI/CD to ship 4-platform release

### 🚀 Enhancements

**Multi-Platform Release Automation**
- Automated 4-platform binary generation: Linux x86_64, Windows x86_64, macOS Intel, macOS ARM64
- Enhanced GitHub Actions workflow with improved error handling
- Added Docker-based cross-compilation for future ARM64 Linux support

**Cross-Compilation Documentation**
- Added comprehensive cross-compilation guide (`docs/CROSS_COMPILATION.md`)
- Documented Docker QEMU emulation process for ARM64 builds
- Updated internal documentation with proven ARM64 build methods

### 🔧 Technical Improvements

**Security Dependencies**
- Migrated from OpenSSL to rustls for better cross-platform compatibility
- Reduced C++ dependency complexity in cross-compilation builds
- Enhanced static linking for standalone binaries

**Release Process**
- Streamlined release workflow to prevent CI/CD failures
- Added fallback strategies for platform-specific build issues
- Improved artifact naming consistency across platforms

### 📦 Platform Support

**Current Release Platforms:**
- ✅ Linux x86_64 (native build)
- ✅ Windows x86_64 (native build) 
- ✅ macOS Intel (native build)
- ✅ macOS ARM64 (native build)
- 🔄 Linux ARM64 (Docker QEMU build - documented process available)

**Binary Downloads:**
- All platforms available via GitHub Releases
- Windows users: Download `shimmy-windows-x86_64.exe` directly
- Linux ARM64: Docker build process documented for manual compilation

### 🛠️ Developer Experience

**Build Infrastructure**
- Enhanced CI/CD pipeline reliability
- Added Docker-based cross-compilation for complex targets
- Improved error reporting and debugging for build failures
- Added comprehensive build documentation

### 📖 Documentation

**Deployment Guides**
- Docker Compose setup for production deployments
- Cloud platform deployment instructions (Railway, Render, Fly.io)
- Cross-compilation guide for ARM64 Linux builds
- Windows installation troubleshooting guide

## [1.3.1] - 2025-09-12

### ✨ Features

**Full llama.cpp Support on All Platforms**
- Enabled complete llama.cpp support across Linux, Windows, macOS Intel, and macOS ARM64
- Resolved macOS ARM64 compilation issues with forked llama-cpp dependency
- Added ARM64-specific compiler capability detection and optimizations

**Enhanced Build System**
- Improved cross-platform compilation with specialized ARM64 handling
- Added comprehensive testing for macOS ARM64 llama.cpp integration
- Streamlined release workflow configuration for stable deployments

### 🐛 Bug Fixes

**macOS ARM64 Compilation Issues**
- Fixed GGML_ARM_I8MM compilation conflicts on Apple Silicon
- Resolved mixed-ISA build problems with targeted compiler flags
- Added proper target detection for ARM64 optimization features

**Release Workflow Stability**
- Enhanced release pipeline reliability across all supported platforms
- Fixed deployment configuration issues for stable v1.3.1 releases
- Improved error handling and fallback strategies

### 🔧 Technical Improvements

**Cross-Platform Compatibility**
- Updated llama-cpp dependency to specialized fork with ARM64 fixes
- Enhanced build.rs with platform-specific compilation logic
- Added comprehensive CMAKE configuration for different architectures

**Testing Infrastructure**
- Added isolated macOS ARM64 llama compilation testing
- Enhanced platform-specific build validation
- Improved error reporting for architecture-specific issues

## [1.2.0] - 2025-09-10

### ✨ Features

**Native SafeTensors Support**
- Implemented native SafeTensors inference engine with zero Python dependencies
- Added complete SafeTensors model format support alongside GGUF
- Enhanced model detection and loading for SafeTensors files

**Enhanced Build System**
- Updated release workflow with comprehensive system dependencies
- Added support for all-features builds across platforms
- Improved cross-platform compilation reliability

### 🐛 Bug Fixes

**Build and Deployment Issues**
- Fixed release binary generation to exclude problematic llama.cpp dependencies
- Resolved macOS runner cmake installation conflicts
- Enhanced GitHub Actions workflow with proper dependency management

**Model Discovery Improvements**
- Fixed infinite recursion issues in model discovery on macOS
- Enhanced model loading robustness across different file formats
- Improved error handling for corrupted or incomplete model files

### 🚀 Enhancements

**Performance Optimizations**
- Native SafeTensors processing for faster model loading
- Reduced memory footprint with optimized inference pipeline
- Enhanced startup performance with efficient model detection

**Developer Experience**
- Comprehensive testing suite for SafeTensors functionality
- Improved documentation for multi-format model support
- Enhanced debugging and error reporting capabilities

## [1.1.0] - 2025-09-09

### ✨ Features

**Revolutionary Testing Framework**
- Implemented PPT (Property-based Testing) framework for comprehensive coverage
- Added invariant testing system for robust quality assurance
- Enhanced testing excellence with automated property verification

**Code Quality Improvements**
- Eliminated all compiler warnings for clean, professional builds
- Implemented comprehensive linting and formatting standards
- Enhanced code documentation and maintainability

### 🔧 Technical Improvements

**Testing Infrastructure**
- Advanced property-based testing with automated edge case discovery
- Invariant checking system for critical functionality validation
- Comprehensive test coverage across all major components

**Build System Enhancements**
- Clean compilation with zero warnings across all platforms
- Enhanced build performance and reliability
- Improved development workflow with better error reporting

## [1.0.1] - 2025-09-08

### 🐛 Bug Fixes

**Critical Issues Resolved**
- **Issue #6**: Fixed model discovery and loading failures
- **Issue #7**: Resolved OpenAI API compatibility problems
- **Issue #5**: Fixed chat completions hanging during generation

**Performance Improvements**
- Enhanced health and metrics endpoints for production monitoring
- Improved error handling and recovery mechanisms
- Optimized model loading and inference pipeline

### ✨ Features

**Enhanced Monitoring**
- Added comprehensive health check endpoints
- Implemented detailed metrics collection for performance tracking
- Enhanced production readiness with robust monitoring capabilities

**User Experience**
- Added shimmy logo and improved visual branding
- Enhanced error messages and user feedback
- Improved CLI interface responsiveness

### 🔧 Technical Improvements

**Backend Reliability**
- Improved backend selection logic for model compatibility
- Enhanced error recovery and graceful degradation
- Better handling of edge cases in model loading

**Development Tools**
- Configured Claude Code integration for improved development workflow
- Enhanced debugging capabilities and error reporting
- Improved development environment setup

## [1.0.0] - 2025-09-08

### ✨ Features

**Production Release**
- First stable release with comprehensive cross-platform support
- Mature OpenAI API compatibility layer
- Production-ready inference engine with robust error handling

**Enhanced Model Discovery**
- Improved Ollama model discovery with proper manifest parsing
- Cross-platform model detection and loading
- Enhanced compatibility with existing Ollama installations

**Automated Release Infrastructure**
- Complete cross-platform build automation via GitHub Actions
- Automated binary generation for all supported platforms
- Comprehensive governance and contribution guidelines

### 🚀 Enhancements

**Build System Maturity**
- Replaced experimental cross-compilation with stable native builds
- Enhanced release workflow reliability and consistency
- Improved artifact generation and distribution

**Community Infrastructure**
- Added comprehensive GitHub automation and governance
- Implemented professional contribution guidelines
- Enhanced project documentation and developer resources

### 🔧 Technical Improvements

**Stability and Reliability**
- Production-grade error handling and recovery
- Enhanced performance optimization across platforms
- Comprehensive testing and validation framework

## [0.1.1] - 2025-09-06

### ✨ Features

**Native Ollama Integration**
- Added comprehensive Ollama model discovery support
- Enhanced compatibility with existing Ollama installations
- Improved model detection and loading from Ollama directories

**Community Support**
- Added multiple sponsorship options: Buy Me a Coffee, Ko-fi, Open Collective
- Enhanced funding infrastructure for sustainable development
- Improved community engagement and support channels

### 📖 Documentation

**Platform Compatibility**
- Added comprehensive macOS compatibility documentation
- Enhanced Windows installation instructions with security notes
- Improved platform-specific guidance and troubleshooting

**User Experience**
- Added Windows Defender false positive warnings and solutions
- Enhanced installation clarity for new users
- Improved discoverability with better crates.io keywords

### 🐛 Bug Fixes

**Build and Distribution**
- Fixed cross-compilation issues and CI/CD pipeline stability
- Resolved dependency conflicts in GitHub Actions
- Enhanced build reliability across different environments

**Code Quality**
- Cleaned up README markdown formatting for better readability
- Fixed unused import warnings and code quality issues
- Enhanced overall code organization and maintainability

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
