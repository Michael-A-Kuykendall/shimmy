# Quick Start: Shimmy in 30 Seconds

**✨ NEW in v2.0.0**: Airframe GPU engine — pure-Rust WGSL inference. One binary per platform, no CUDA/Vulkan SDK required.

## 1. Download Pre-Built Binary

Pick your platform — all binaries use automatic WebGPU (wgpu) GPU detection:

```bash
# Windows x64
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# Linux x86_64
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy && chmod +x shimmy

# macOS ARM64 (Apple Silicon)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy && chmod +x shimmy

# macOS Intel
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-intel -o shimmy && chmod +x shimmy

# Linux ARM64
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-aarch64 -o shimmy && chmod +x shimmy
```

**GPU detection is automatic.** Airframe uses wgpu to find the best adapter on your hardware.

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
