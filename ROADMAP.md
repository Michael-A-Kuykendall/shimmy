# Shimmy Roadmap

Shimmy is a 5MB, zero-config, OpenAI-compatible inference server.
Its mission is **invisible infrastructure**: drop it in, it works.

## Current Milestones
- ✅ Basic server skeleton with OpenAI-compatible endpoints
- ✅ Initial `/v1/chat/completions` support
- ✅ Native Ollama model discovery (`~/.ollama/models/`)
- ✅ Auto port allocation with conflict avoidance
- ✅ GGUF model auto-discovery from HuggingFace cache
- ✅ VS Code extension integration
- ✅ WebSocket streaming support
- ✅ LoRA adapter foundation

## Next Goals
- [ ] `/v1/embeddings` endpoint (minimal stub, then model-backed)
- [ ] Micro-benchmarking of startup time and request latency
- [ ] Model discovery improvements (config-free)
- [ ] Enhanced test suite for API compatibility
- [ ] Docker packaging and multi-platform binaries
- [ ] Performance optimization for <100ms startup target

## Long-Term Vision
- Full OpenAI API coverage (completions, embeddings, chat, models)
- Maintain <5MB binary target (currently 5.1MB)
- Zero external configuration, portable deployment
- Optional acceleration via SIMD/GPU if footprint remains small
- Enterprise-grade reliability with consumer-grade simplicity

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