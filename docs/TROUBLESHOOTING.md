# Troubleshooting Guide

This guide covers the most common failures, error messages, and platform-specific issues you'll encounter with Shimmy v2.0.

---

## Quick Diagnostics

Before diving into specific errors, enable verbose logging:

```bash
RUST_BACKTRACE=1 SHIMMY_VERBOSE=1 SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve
```

`RUST_BACKTRACE=1` prints a full stack trace on any panic. `SHIMMY_VERBOSE=1` enables per-request timing and inference trace headers. If the server crashes silently without these, you're missing the signal.

---

## Server Won't Start

### "Address already in use" / port conflict

**Symptom:** Server exits immediately, no inference.

**Cause:** Another process is bound to port 8080 (or whatever `SHIMMY_PORT`/`SHIMMY_BIND_ADDRESS` specifies). Frequently a previous Shimmy or Ollama instance that wasn't shut down cleanly.

**Fix (Linux/macOS):**
```bash
lsof -i :8080
kill <PID>
```

**Fix (Windows):**
```powershell
netstat -ano | findstr :8080
Stop-Process -Id <PID>
```

**Or just change the port:**
```bash
SHIMMY_BIND_ADDRESS=127.0.0.1:11435 shimmy serve
```

---

### "Model not found" / no models registered

**Symptom:** `shimmy list` returns empty, or `/v1/models` returns `{"data":[]}`.

**Cause:** Shimmy can't find a GGUF file on any of its discovery paths.

**Fix:** Set the path explicitly:
```bash
SHIMMY_BASE_GGUF=/absolute/path/to/model.gguf shimmy serve
```

**Discovery paths searched automatically (checked in order):**
1. `SHIMMY_BASE_GGUF` env var
2. `SHIMMY_MODEL_PATHS` env var (colon-separated list of directories)
3. `~/.cache/huggingface/hub/`
4. `~/.ollama/models/`
5. `~/.cache/lm-studio/models/`
6. `./models/` (relative to the working directory)

If the file exists but isn't being found, check for permission issues and ensure the path doesn't have spaces or special characters (on Windows, use quotes if needed).

---

### "failed to create adapter" / "no suitable GPU adapter found"

**Symptom:** Server starts but immediately fails with a wgpu/adapter error.

**Cause:** wgpu cannot enumerate a usable GPU adapter. This happens when:
- GPU drivers are absent or outdated
- You're running in a virtualized environment without GPU passthrough
- On Linux, Vulkan is not installed

**Fix (Linux):**
```bash
# Install Vulkan support
sudo apt-get install libvulkan1 mesa-vulkan-drivers

# Verify adapter is visible
vulkaninfo --summary
```

**Fix (Windows):** Update your GPU drivers. Direct3D 12 is required; any Windows 10 system with a GPU from 2015+ should work.

**Fix (macOS):** Metal is always available on macOS 12+. If you see this error, try reinstalling the Shimmy binary — the wgpu Metal layer may have failed to link.

**Force CPU (last resort — very slow):**
```bash
shimmy serve --gpu-backend cpu
```

---

### VRAM Out of Memory (OOM)

**Symptom:** Server panics with a buffer allocation error, typically containing `"buffer binding"` or `"out of memory"` in the message. Happens at model load, not during inference.

**Cause:** The model weights or KV cache allocation exceeds available GPU VRAM. The most common cause is trying to run a model that's too large for your GPU, or setting `SHIMMY_MAX_CTX` too high.

**Diagnosis:**
```bash
# Windows: check VRAM usage
nvidia-smi  # or GPU-Z

# Linux
nvidia-smi --query-gpu=memory.free,memory.total --format=csv

# macOS (unified memory)
# Activity Monitor → Window → GPU History
```

**Fixes:**
1. Use a smaller model or more aggressively quantized version (Q4_0 instead of Q4_K_M)
2. Reduce context: `SHIMMY_MAX_CTX=2048`
3. Close other GPU applications (browsers using WebGL/WebGPU, games, other ML tools)

**WebGPU buffer cap:** Airframe has a hard limit of ~2 GB per GPU buffer allocation (enforced by the WebGPU spec). Models with single tensors exceeding 2 GB (e.g., the output embedding in large models) will fail to load until chunked output head support is implemented. See [Model Compatibility](#model-specific-known-failures) below.

---

## Inference Issues

### Model outputs garbage / nonsense responses

**Symptom:** Model loads, generates tokens, but the output is incoherent or wrong. Can range from random characters to plausible-looking but wrong answers.

**Possible causes and fixes:**

**1. Wrong chat template applied**

Different model families require different conversation formatting (see [CHAT_TEMPLATES.md](CHAT_TEMPLATES.md)). If the wrong template is applied, the model receives a malformed prompt and outputs garbage.

```bash
# Check which template shimmy detected
SHIMMY_VERBOSE=1 shimmy serve  # will log template detection on first request
```

**2. EOS token not stopping generation**

If generation runs to `max_tokens` instead of stopping naturally, the model's EOS token may not be recognized. This is most common with Llama-3 models where `<|eot_id|>` is the stop token, not `<eos>`.

**Fix:** Pass an explicit stop token in the request:
```json
{
  "model": "llama-3.2-1b",
  "messages": [...],
  "stop": ["<|eot_id|>"]
}
```

**3. Context overflow (silent truncation)**

If your prompt plus history exceeds `SHIMMY_MAX_CTX` tokens, the input is silently truncated from the front. The model loses the beginning of the conversation and may appear incoherent.

**Fix:** Set a higher `SHIMMY_MAX_CTX` or summarize long conversations before sending.

**4. Temperature too high**

With `temperature > 1.5`, the softmax distribution becomes nearly uniform and outputs become random.

---

### Generation is very slow

**Symptom:** Tokens take several seconds each to produce.

**Cause and fix:**

**GPU not being used:**
```bash
shimmy gpu-info  # Should show a non-CPU adapter
```

If this shows CPU adapter (llvmpipe, SwiftShader, or similar), GPU acceleration is not active. See the "no suitable GPU adapter" section above.

**Large context window:**
Inference time scales as `O(n²)` with context length during the attention computation. A 8192-token context is 16x slower than a 2048-token context for the same model.

**Model too large for VRAM:**
If the model doesn't fit fully in VRAM, wgpu will fall back to system RAM for some buffers, causing PCIe bus transfers that are 5-50x slower than VRAM access.

---

### `max_tokens` validation errors (HTTP 400)

**Symptom:** API returns `400 Bad Request` with a message about `max_tokens`.

**Valid range:** 1 to 131072. Values of 0 or above 131072 are rejected.

**Fix:** Check your client code is setting `max_tokens` to a positive integer within range. If not specified, it defaults to a safe value (model-dependent, typically 512).

---

## Model-Specific Known Failures

### Phi-3 / Phi-3.5 mini — Fused QKV tensors

**Symptom:** Server panics at model load with an error about `attn_qkv.weight` or "unexpected tensor shape".

**Cause:** Phi-3 and Phi-3.5 pack the Q, K, and V weight matrices into a single fused tensor (`attn_qkv.weight`). Airframe expects separate `attn_q.weight`, `attn_k.weight`, `attn_v.weight` tensors.

**Status:** Planned fix — fused QKV splitting at load time.

**Workaround:** Use Phi-2 instead (supported), or wait for the fused QKV release.

---

### Gemma-2-2B — Output head buffer limit

**Symptom:** Server panics during model load with a buffer allocation error on the `output.weight` tensor.

**Cause:** Gemma-2-2B's output embedding tensor is approximately 2.19 GB, which exceeds the WebGPU single-buffer limit (~2 GB on most adapters). The entire tensor cannot be bound in a single buffer.

**Status:** Planned fix — chunked output head projection.

**Workaround:** None currently. Use a different model.

---

### Qwen models — Missing QK norm shader

**Symptom:** Server panics or produces incorrect output on Qwen or Qwen2 models.

**Cause:** Qwen uses QK normalization (a normalization layer applied to Q and K after RoPE), which isn't implemented in the current Airframe attention shader.

**Status:** Planned for a future release.

---

## Platform-Specific Issues

### Windows

**"The procedure entry point could not be located in the dynamic link library"**

Usually a Visual C++ runtime mismatch. Install the latest [Visual C++ Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist).

**DirectX 12 errors**

Shimmy uses D3D12 on Windows via wgpu. Requires Windows 10 1809 or later and DirectX 12-capable GPU (any GPU from 2014 or later on Windows 10).

**Antivirus flagging the binary**

Some antivirus software flags new Rust binaries as suspicious. The binary is built from open-source code — check the GitHub Actions release workflow for transparency. Add an exclusion if needed.

**Long path issues on Windows**

If your GGUF model path contains very long directory names, Windows path length limits (260 chars by default) may cause file-not-found errors. Enable long paths in Windows registry or use a shorter path.

---

### Linux

**"error while loading shared libraries: libvulkan.so.1"**

```bash
sudo apt-get install libvulkan1
# or for AMD:
sudo apt-get install mesa-vulkan-drivers
# for NVIDIA (if using Vulkan):
# Vulkan is included with the NVIDIA proprietary driver
```

**wgpu falling back to OpenGL instead of Vulkan**

Force Vulkan explicitly:
```bash
WGPU_BACKEND=vulkan shimmy serve
```

**Permission denied on model file**

```bash
chmod 644 /path/to/model.gguf
```

---

### macOS

**"shimmy" cannot be opened because Apple cannot check it for malicious software**

This is Gatekeeper blocking the unsigned binary. Fix:
```bash
xattr -d com.apple.quarantine shimmy
```
or right-click → Open in Finder on first launch.

**Metal GPU not selected**

On Apple Silicon, Metal should be auto-selected. If it isn't:
```bash
WGPU_BACKEND=metal shimmy serve
```

**High memory pressure on Apple Silicon**

Apple Silicon uses unified memory shared between CPU and GPU. "VRAM" is just regular RAM allocated to the GPU. The VRAM budgets in this guide apply directly to your total RAM minus OS overhead (~2-3 GB).

---

## Debugging Crashes with RUST_BACKTRACE

When Shimmy crashes, set `RUST_BACKTRACE=1` to get a full stack trace:

```bash
RUST_BACKTRACE=1 SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve 2>&1 | tee crash.log
```

Key things to look for in the backtrace:
- `wgpu::` frames — GPU/driver-level failure
- `airframe::backend::bindless` — weight tensor allocation or shader issue
- `airframe::runtime::gpu::generate` — inference loop issue
- `shimmy::openai_compat` — HTTP/API layer issue

When reporting a crash, always include:
1. The full backtrace from `RUST_BACKTRACE=1`
2. The model file name and size
3. Your OS and GPU model
4. The exact command or API call that triggered the crash

---

## Getting Help

- **GitHub Issues**: [github.com/Michael-A-Kuykendall/shimmy/issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
  - Search existing issues before filing — most errors have been seen before
  - Include model info, OS, GPU, and `RUST_BACKTRACE=1` output
- **GitHub Discussions**: [github.com/Michael-A-Kuykendall/shimmy/discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)
  - Good for "how do I..." questions and configuration help

---

## Further Reading

- [EXTENDED_CONTEXT.md](EXTENDED_CONTEXT.md) — VRAM sizing and YaRN configuration
- [QUANTIZATION.md](QUANTIZATION.md) — which quant format to use and why
- [CHAT_TEMPLATES.md](CHAT_TEMPLATES.md) — stop token and template issues per model family
- [ARCHITECTURE.md](ARCHITECTURE.md) — understanding what Shimmy is doing internally
