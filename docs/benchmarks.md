# Benchmarks: Shimmy vs The Competition

*Last updated: September 2025*

## Binary Size Comparison
| Tool | Binary Size | Winner |
|------|-------------|---------|
| **Shimmy** | **5.1MB** | 🏆 |
| Ollama | 680MB | |
| llama.cpp | 89MB | |

**Why it matters**: Smaller = faster downloads, easier deployment, less disk usage

## Startup Time
| Tool | Cold Start | Winner |
|------|------------|---------|
| **Shimmy** | **<100ms** | 🏆 |
| Ollama | 5-10s | |
| llama.cpp | 1-2s | |

**Why it matters**: Faster startup = better developer experience, container efficiency

## Memory Overhead (Idle)
| Tool | RAM Usage | Winner |
|------|-----------|---------|
| **Shimmy** | **50MB** | 🏆 |
| Ollama | 200MB+ | |
| llama.cpp | 100MB | |

**Why it matters**: Lower overhead = more memory for your models and apps

## OpenAI API Compatibility
| Tool | Chat Completions | Models Endpoint | Streaming | Winner |
|------|------------------|-----------------|-----------|---------|
| **Shimmy** | ✅ | ✅ | ✅ | 🏆 |
| Ollama | ✅ | ✅ | ⚠️ Partial | |
| llama.cpp | ❌ | ❌ | ❌ | |

**Why it matters**: Full compatibility = seamless integration with existing AI tools

## Configuration Required
| Tool | Setup Steps | Winner |
|------|-------------|---------|
| **Shimmy** | **0** (auto-discovery) | 🏆 |
| Ollama | 3+ steps | |
| llama.cpp | 5+ steps | |

**Why it matters**: Zero config = just works, no documentation reading required

---

## Real-World Performance Test

*Test environment: Intel i7-12700K, 32GB RAM, Phi-3-Mini-4K-Instruct (Q4_K_M)*

### Time to First Token
- **Shimmy**: 45ms
- Ollama: 52ms  
- llama.cpp: 48ms

### Tokens per Second
- **Shimmy**: ~85 tok/sec
- Ollama: ~82 tok/sec
- llama.cpp: ~87 tok/sec

**Conclusion**: Performance parity with dramatically better resource efficiency

---

*Want to contribute benchmarks? Open an issue with your results!*
