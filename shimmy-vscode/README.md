# Shimmy VS Code Extension

The official VS Code extension for **Shimmy: The 5MB Alternative to Ollama**.

## Features

- **One-click server management**: Start, stop, and restart Shimmy from the status bar
- **Instant code generation**: Right-click any text selection to generate code completions
- **Auto-discovery**: Works with any GGUF model, with zero-friction LoRA serving
- **Zero configuration**: Smart defaults that just work
- **Status monitoring**: Real-time server status in the status bar

## Quick Start

1. Install Shimmy binary: `cargo install shimmy --features llama`
2. Install this extension
3. Set your model path in settings (or use `SHIMMY_BASE_GGUF` environment variable)
4. Click the status bar to start the server
5. Right-click selected code → "Generate from Selection"

## Commands

- `Shimmy: Start Server` - Start the Shimmy inference server
- `Shimmy: Stop Server` - Stop the running server  
- `Shimmy: Restart Server` - Restart the server
- `Shimmy: Generate Code` - Open prompt dialog for code generation
- `Generate from Selection` - Generate code based on selected text (context menu)

## Configuration

| Setting | Description | Default |
|---------|-------------|---------|
| `shimmy.serverUrl` | Shimmy server URL | `http://localhost:11435` |
| `shimmy.binaryPath` | Path to shimmy binary | `shimmy` |
| `shimmy.modelPath` | Path to GGUF model file | (uses `SHIMMY_BASE_GGUF` env var) |
| `shimmy.autoStart` | Auto-start server on activation | `false` |

## Integration with Other Tools

Shimmy provides OpenAI-compatible endpoints, so it works with:

- **GitHub Copilot**: Configure server URL in settings
- **Cursor**: Point to `http://localhost:11435` 
- **Continue.dev**: Add as custom model provider
- **Any OpenAI-compatible tool**: Just change the base URL

## Troubleshooting

**Server won't start?**
- Check that `shimmy` binary is in your PATH
- Verify model path is correct
- Ensure port 11435 is available

**No code generation?**
- Check server status in status bar (should show green checkmark)
- Verify model is loaded properly
- Check VS Code developer console for errors

**Performance tips:**
- Use smaller models (1-3B parameters) for faster responses
- Place models on SSD for faster loading
- Consider LoRA adapters for specialized tasks

## About Shimmy

Shimmy is the **5MB alternative to Ollama** - a single-binary local inference server that:

- Starts in <100ms vs Ollama's 5-10 seconds
- Uses <50MB RAM vs Ollama's 200MB+ overhead  
- 100% OpenAI API compatible
- First-class LoRA adapter support
- Zero configuration required

**Privacy-first, cost-free, blazing fast local AI.**

## License

MIT License - see [Shimmy repository](https://github.com/Michael-A-Kuykendall/shimmy) for details.
