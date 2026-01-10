# Quick Start: Shimmy in 30 Seconds

**✨ NEW in v1.9.0**: One binary per platform with ALL GPU backends included! No compilation needed.

## 1. Download Pre-Built Binary

Pick your platform - each includes automatic GPU detection:

```bash
# Windows x64 (includes CUDA + Vulkan + OpenCL)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# Linux x86_64 (includes CUDA + Vulkan + OpenCL)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy && chmod +x shimmy

# macOS ARM64 (includes MLX for Apple Silicon)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy && chmod +x shimmy

# macOS Intel (CPU-only)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-intel -o shimmy && chmod +x shimmy

# Linux ARM64 (CPU-only)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-aarch64 -o shimmy && chmod +x shimmy
```

**That's it!** Your GPU (if available) will be detected automatically at runtime.

## 2. Get a Model
Place any `.gguf` file in one of these locations:
- `./models/your-model.gguf`
- Set `SHIMMY_BASE_GGUF=/path/to/your-model.gguf`
- Or just put it in `~/Downloads/` - Shimmy will find it

**Don't have a model?** Try [microsoft/Phi-3-mini-4k-instruct-gguf](https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf)

## 3. Start Shimmy
```bash
./shimmy serve
```

That's it! Shimmy is now running on `http://localhost:11435`

## 4. Connect Your Tools

**VSCode Copilot**:
```json
// settings.json
{
  "github.copilot.advanced": {
    "serverUrl": "http://localhost:11435"
  }
}
```

**Continue.dev**:
```json
{
  "models": [{
    "title": "Local Shimmy",
    "provider": "openai",
    "model": "your-model-name",
    "apiBase": "http://localhost:11435/v1"
  }]
}
```

**Cursor**:
Set custom endpoint to `http://localhost:11435`

## 5. Test It
```bash
# List available models
./shimmy list

# Test generation
./shimmy generate --name your-model --prompt "Hello!" --max-tokens 10

# Or use curl
curl -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-model",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 10
  }'
```

## Troubleshooting

**No models found?**
- Make sure your `.gguf` file is in `./models/` or set `SHIMMY_BASE_GGUF`
- Run `./shimmy discover` to see what Shimmy can find

**Port already in use?**
```bash
./shimmy serve --bind 127.0.0.1:11436
```

**Need help?**
- [Open an issue](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- Check existing [discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)

---

**Next**: Check out [integrations](integrations.md) for more examples!
