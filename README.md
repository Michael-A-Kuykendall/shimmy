# Shimmy

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)

**Shimmy** is a fast, lightweight local LLM inference server that acts as a universal shim between AI tools and local language models. Built in Rust with a focus on offline-first AI workflows.

## ✨ Key Features

- **🚀 Fast Local Inference**: GGUF model support via llama.cpp integration
- **🔌 Multiple APIs**: HTTP/JSON, Server-Sent Events (SSE), and WebSocket streaming
- **⚡ Single Binary**: Zero-dependency deployment, just download and run
- **🎯 Tool Integration**: Built-in compatibility with punch-discovery, RustChain, and more
- **🔧 LoRA Support**: Dynamic adapter loading for specialized models
- **📡 Offline First**: Complete functionality without internet connectivity

## 🚀 Quick Start

### Installation

Download the latest release for your platform or build from source:

```bash
# Clone and build
git clone https://github.com/yourusername/shimmy.git
cd shimmy
cargo build --release --features llama

# Or download pre-built binary from releases
```

### Basic Usage

```bash
# Set your model path
export SHIMMY_BASE_GGUF=/path/to/your/model.gguf

# Start the server
./shimmy serve --bind 127.0.0.1:11435

# Generate text via CLI
./shimmy generate --prompt "Hello, world!" --max-tokens 50

# Check available models
./shimmy list
```

### API Usage

```bash
# HTTP API
curl -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "default",
    "prompt": "Hello, world!",
    "max_tokens": 50,
    "stream": false
  }'

# Streaming with Server-Sent Events
curl -X POST http://localhost:11435/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "default", 
    "prompt": "Hello, world!",
    "stream": true
  }'
```

## 📖 Documentation

- [API Reference](docs/API.md) - Complete API documentation
- [Configuration](docs/CONFIGURATION.md) - Setup and configuration options
- [Integration Guide](docs/INTEGRATION.md) - Using shimmy with other tools
- [Examples](docs/EXAMPLES.md) - Common usage patterns

## 🛠️ Configuration

### Environment Variables

- `SHIMMY_BASE_GGUF`: Path to the base GGUF model file (required)
- `SHIMMY_LORA_GGUF`: Path to LoRA adapter file (optional)

### Supported Model Formats

- GGUF files (primary support)
- LoRA adapters in GGUF format

## 🔌 Integration

Shimmy is designed to work seamlessly with:

- **RustChain**: AI agent mission execution
- **Punch Discovery**: Codebase analysis and insights
- **VSCode Extensions**: Any tool expecting OpenAI-compatible APIs
- **Custom Tools**: Via HTTP API, CLI, or direct integration

## 🏗️ Architecture

Shimmy follows a modular architecture:

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│   HTTP/WS API   │    │   Engine     │    │  llama.cpp      │
│                 │◄──►│              │◄──►│                 │
│ • REST          │    │ • Inference  │    │ • GGUF Loading  │
│ • SSE Streaming │    │ • Templates  │    │ • LoRA Support  │
│ • WebSocket     │    │ • Threading  │    │ • Optimization  │
└─────────────────┘    └──────────────┘    └─────────────────┘
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/yourusername/shimmy.git
cd shimmy

# Build with llama feature
cargo build --features llama

# Run tests
cargo test
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built on [llama.cpp](https://github.com/ggerganov/llama.cpp) for fast local inference
- Inspired by the need for reliable offline AI workflows
- Part of the broader ecosystem including punch-discovery and RustChain

---

**Status**: Active development | **Stability**: Beta | **Platform**: Cross-platform

For questions, issues, or feature requests, please [open an issue](https://github.com/yourusername/shimmy/issues) on GitHub.
