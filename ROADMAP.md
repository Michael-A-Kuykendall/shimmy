# Shimmy Roadmap 🚀

**Mission:** Lightweight, local-first AI inference infrastructure.

Shimmy is a sub-5MB, zero-config, OpenAI-compatible inference server targeting the **$50B+ AI inference market**. Its mission is **invisible infrastructure**: drop it in, it works.

## 🆓 Forever Free Commitment

**Shimmy Core will always remain completely free and open-source.** This is not a "free tier" or "community edition" - it's a permanent commitment to the developer community.

- ✅ **No feature limitations** - Full functionality, forever
- ✅ **No usage limits** - Use it commercially, personally, anywhere
- ✅ **No forced upgrades** - Current version will always work
- ✅ **Community first** - Built for developers, by developers

Product roadmap items in this repository track core open-source Shimmy scope.

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
- ✅ LoRA adapter foundation

## 🎯 Q4 2025 Milestones
- [ ] **Enterprise Embeddings** - `/v1/embeddings` endpoint (targeting RAG workloads)
- [ ] **Sub-50ms Startup** - Micro-benchmarking and optimization
- [ ] **Model Marketplace** - Discovery improvements with popularity rankings
- [ ] **Container-First** - Docker packaging and multi-platform binaries
- [ ] **Enterprise Analytics** - Optional usage metrics and monitoring hooks
- [ ] **10,000 User Milestone** - Community growth and feedback integration

## 🚀 2026 Strategic Initiatives
- [ ] **Multi-Model Orchestration** - Load balancing across multiple models
- [ ] **Reliability Hardening** - Better recovery and stability under sustained load
- [ ] **Developer Experience** - Faster local setup and diagnostics

## 🌟 Long-Term Direction (2027+)

### Technical Excellence
- **100% OpenAI API Parity** - Complete feature compatibility
- **Sub-5MB Binary** - Maintain lightweight footprint (current: 4.8MB, 142x smaller than Ollama)
- **Universal Deployment** - Zero configuration, runs anywhere
- **Hardware Optimization** - SIMD/GPU acceleration within size constraints
- **Enterprise Reliability** - 99.99% uptime, consumer simplicity

### Market Expansion
- **1M+ Active Developers** - Become the standard for local AI
- **Enterprise Standard** - Default choice for privacy-conscious organizations
- **Ecosystem Platform** - Hub for local AI development tools
- **Global Infrastructure** - Enable offline AI development worldwide

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
