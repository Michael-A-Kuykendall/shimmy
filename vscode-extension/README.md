# Shimmy VS Code Extension

The official VS Code extension for [Shimmy](https://github.com/Michael-A-Kuykendall/shimmy) - the 5MB Ollama alternative.

## Features

- **Auto-start detection**: Automatically detects GGUF/SafeTensors files in your workspace
- **Dynamic port management**: No more port conflicts - Shimmy automatically finds available ports
- **Right-click serving**: Serve any model file directly from the file explorer
- **Status bar integration**: Monitor Shimmy server status at a glance
- **Zero configuration**: Works out of the box

## Quick Start

1. Install the Shimmy binary: `cargo install shimmy --features llama`
2. Install this extension from the VS Code Marketplace
3. Open a workspace with model files (.gguf or .safetensors)
4. Extension will prompt to start the server automatically

## Commands

- `Shimmy: Start Shimmy Server` - Start the local inference server
- `Shimmy: Stop Shimmy Server` - Stop the running server  
- `Shimmy: Serve Model with Shimmy` - Serve a specific model file

## Configuration

- `shimmy.autoPort`: Use automatic port allocation (recommended) - Default: true
- `shimmy.manualPort`: Manual port override (only if autoPort is false) - Default: 11435
- `shimmy.binaryPath`: Path to shimmy binary - Default: "shimmy"

## Usage with AI Tools

Once Shimmy is running, configure your AI tools to use the local endpoint:

**Continue.dev**:
```json
{
  "models": [{
    "title": "Local Shimmy",
    "provider": "openai",
    "model": "your-model-name", 
    "apiBase": "http://localhost:[PORT]/v1"
  }]
}
```

**Cursor IDE**: Set API base URL to `http://localhost:[PORT]/v1`

The port will be displayed in the VS Code status bar when Shimmy is running.

## Dynamic Port Management

Shimmy automatically allocates available ports to prevent conflicts with other development tools. This means:

- No more "port already in use" errors
- Seamless integration with multiple AI tools
- Zero configuration port management
- Automatic cleanup when server stops

## Support

- **Issues**: [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- **Documentation**: [Shimmy Docs](https://github.com/Michael-A-Kuykendall/shimmy/blob/main/README.md)
- **Discussions**: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)

## License

MIT License - same as Shimmy itself.