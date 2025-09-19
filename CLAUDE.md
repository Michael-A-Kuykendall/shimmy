# Claude Code Configuration for Shimmy

## Project Overview
Shimmy is a lightweight sub-20MB Rust inference engine serving as an optimal shim for AI model inference. It provides OpenAI API compatibility with native SafeTensors support, 2x faster loading, and no Python dependencies.

## Critical Development Rules

**READ BEFORE WRITE**: Always read a file before editing or writing to it (Claude Code requirement)
**FOLLOW INTEGRATION PLAN**: Check integration plans before implementation decisions
**PUBLIC RELEASE APPROVAL**: Human approval required for releases, Cargo.toml, README changes
**PROFESSIONAL LANGUAGE**: No profanity, maintain professional standards
**CONVENTIONAL COMMITS**: Use conventional commit format for all commits

## GitHub Spec-Kit Integration

**SPECIFICATION-DRIVEN DEVELOPMENT**: Use GitHub Spec-Kit for all project planning and implementation

### Installation & Setup
GitHub Spec-Kit is installed via uv in a virtual environment:
```bash
export PATH="/c/Users/micha/.local/bin:$PATH"
source spec-kit-env/Scripts/activate
```

### Critical UTF-8 Encoding Fix
**IMPORTANT**: The key that made GitHub Spec-Kit work locally was the UTF-8 encoding override:

```bash
PYTHONIOENCODING=utf-8 specify [command]
```

This environment variable override fixes Unicode encoding issues that cause crashes with the banner display.

### Available Commands
- `PYTHONIOENCODING=utf-8 specify init <project_name>` - Initialize new project
- `PYTHONIOENCODING=utf-8 specify init <project_name> --ai claude` - Initialize with Claude
- `PYTHONIOENCODING=utf-8 specify check` - Check system requirements

### Core Workflow
1. `/specify` - Create detailed feature specification (defines WHAT and WHY)
2. `/plan` - Generate technical implementation plan (translates to HOW)  
3. `/tasks` - Break down into actionable implementation tasks
4. `implement <path_to_plan>` - Execute the structured implementation

### Project Structure
```
project/
├── memory/
│   ├── constitution.md          # Non-negotiable principles
│   └── constitution_update_checklist.md
├── specs/
│   └── [feature-number]-[feature-name]/
│       ├── spec.md             # Feature specification
│       ├── plan.md             # Technical plan
│       └── contracts/          # Acceptance criteria
└── templates/                  # Reusable patterns
```

## Shimmy Architecture

**Core Principle**: Shimmy transforms complexity into simplicity - a sub-20MB binary that provides enterprise-grade AI inference with zero configuration.

### Key Features
- **Model Support**: SafeTensors (native), GGUF via llama.cpp, HuggingFace integration
- **GPU Acceleration**: NVIDIA CUDA, AMD ROCm, Intel GPU detection
- **API Compatibility**: Drop-in replacement for OpenAI API
- **Performance**: 2x faster model loading, <2s startup time
- **Size**: Sub-20MB binary vs 680MB+ alternatives (34x smaller)

### Testing Strategy
- **Command**: `cargo test --all-features`
- **Integration Tests**: `cargo test --test integration_tests`
- **Benchmark Tests**: `cargo bench`

### Development Environment
- **Platform**: Windows with MSYS2, Rust 1.89+
- **Features**: Use `--features "huggingface,llama"` for full functionality
- **Path Quoting**: Quote Windows paths with spaces: `& "C:\path with spaces\file.exe"`

## Git Workflow
- **Main Branch**: Always ensure clean working tree before major changes
- **Commits**: Use conventional commits format
- **Testing**: Run full test suite before commits
- **Releases**: Require explicit human approval

## Package Management
- **Current Issue**: Package size 67.9MiB exceeds crates.io 10MB limit
- **Solution Needed**: Exclude llama.cpp binaries from package
- **Distribution**: GitHub releases for full binaries, crates.io for source

## Architecture Priorities
1. 🔥 Smart Model Preloading & Warmup System
2. ⚡ Response Caching & Deduplication Engine  
3. 🔧 Integration Templates & Auto-Configuration
4. 🎛️ Request Routing & Connection Intelligence
5. 📊 Advanced Observability & Self-Optimization