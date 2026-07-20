# Shimmy Roadmap 🚀

**Vision:** The privacy-first local AI infrastructure that replaces cloud dependencies

Shimmy is a zero-config, OpenAI-compatible inference server with a native WebGPU GPU engine. Its mission is **invisible infrastructure**: drop it in, it works.

## 🆓 Forever Free Commitment

**Shimmy Core will always remain completely free and open-source.** This is not a "free tier" or "community edition" - it's a permanent commitment to the developer community.

- ✅ **No feature limitations** - Full functionality, forever
- ✅ **No usage limits** - Use it commercially, personally, anywhere
- ✅ **No forced upgrades** - Current version will always work
- ✅ **Community first** - Built for developers, by developers

**Premium offerings (Console/Cloud) are separate products** that extend Shimmy's capabilities but never replace or limit the core experience.

## 📊 Market Position
- **Target Market**: 127M+ developers worldwide running AI workloads
- **Problem**: Cloud AI costs $0.002-0.06/token, vendor lock-in, privacy concerns
- **Solution**: 100% local, 100% private, 100% free, drop-in OpenAI replacement

## Current Milestones
- ✅ Basic server skeleton with OpenAI-compatible endpoints
- ✅ Initial `/v1/chat/completions` support
- ✅ Native Ollama model discovery (`~/.ollama/models/`)
- ✅ Auto port allocation with conflict avoidance
- ✅ GGUF model auto-discovery from HuggingFace cache
- ✅ VS Code extension integration
- ✅ WebSocket streaming support
- ✅ LoRA adapter foundation (llama.cpp path)
- ✅ **Airframe engine** — pure-Rust WGSL GPU inference, shipped in v2.0.0
  - Deterministic GPU output, GGUF-native spec, YaRN RoPE extended context
  - No CUDA toolkit or Vulkan SDK required; wgpu handles adapter selection

## 🎯 Q2–Q3 2026 Milestones
- [x] **Stop tokens from GGUF metadata** — read `tokenizer.ggml.eos_token_id` natively
- [x] **Quantization in Airframe** — Q4_K_M and Q8_0 inference on the WebGPU pipeline
- [ ] **SafeTensors support** — ingest `.safetensors` model checkpoints directly
- [ ] **Multi-model serving** — load balancing across multiple active models
- [ ] **Enterprise Embeddings** — `/v1/embeddings` endpoint targeting RAG workloads
- [ ] **Sub-50ms startup** — benchmarking and initialization optimization

## 🧠 Airframe MoE Support (Planned)

Mixture of Experts model support in the Airframe WebGPU engine — enabling Mixtral, DeepSeek, Qwen MoE
and other sparse transformer architectures without falling back to `--legacy`.

**Engineering estimate: 21 story points** across 7 work items (GGUF loader, router shaders,
top-K selection, per-expert dispatch, output combine, buffer management).

MoE models currently supported via `--legacy` (llama.cpp). Native Airframe MoE is post-quantization.

→ See [docs/AIRFRAME_MOE_ROADMAP.md](docs/AIRFRAME_MOE_ROADMAP.md) for full engineering breakdown.

## 🚀 2026 Strategic Initiatives
- [ ] **Shimmy Console** — terminal UI frontend with retro aesthetics and advanced controls
- [ ] **Developer Experience Suite** — integrated development environment for AI workflows
- [ ] **Multi-Model Orchestration** — load balancing across multiple models
- [ ] **Shimmy Cloud** — enterprise cloud deployment and management platform

## 🌟 Long-Term Vision (2027+)

### Technical Excellence
- **100% OpenAI API Parity** - Complete feature compatibility
- **Universal Deployment** - Zero configuration, runs anywhere
- **Hardware Optimization** - WebGPU/wgpu acceleration, MoE sparse dispatch
- **Enterprise Reliability** - 99.99% uptime, consumer simplicity

### Market Expansion
- **1M+ Active Developers** - Become the standard for local AI
- **Product Suite Leadership** - Shimmy Console and Cloud ecosystem dominance
- **Enterprise Standard** - Default choice for privacy-conscious organizations
- **Ecosystem Platform** - Hub for local AI development tools
- **Global Infrastructure** - Enable offline AI development worldwide
- **Revenue Diversification** - Free core + premium products (not freemium limitations)

### Industry Impact
- **Privacy Leadership** - Set standards for local-first AI development
- **Cost Reduction** - Save developers billions in cloud AI costs
- **Innovation Catalyst** - Enable new categories of privacy-first AI applications
- **Trust Building** - Demonstrate sustainable open-source without bait-and-switch tactics

## Non-Goals
- UI/dashboard (invisible infrastructure philosophy)
- Model training (inference only)
- Complex configuration (zero-config principle)
- Feature bloat (lightweight focus)

---

## Governance
- **Lead Maintainer:** Michael A. Kuykendall
- Contributions are welcome via Pull Requests
- The roadmap is set by the lead maintainer to preserve project vision
- All changes must align with Shimmy's core philosophy: lightweight, zero-config, invisible infrastructure
