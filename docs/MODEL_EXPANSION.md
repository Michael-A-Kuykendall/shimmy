# Model admission and expansion — Shimmy 2.6.6

Current support is the [fixture matrix](SUPPORTED_MODELS.md), not a family-count
marketing list. Airframe admits validated dense Llama and Qwen3 plans and rejects
unsupported execution graphs or required metadata/operations.

Adding a tokenizer, GGUF architecture name or quantization decoder does not add
an executor. A Q4_K_M filename describes quantized tensors, not mathematical
compatibility with every model using them. Gemma2 remains disabled; MoE, MLA,
hybrid recurrent/DeltaNet, SSM and multimodal graphs are not enabled by this
release.

A new executor needs checked metadata/tensor layouts, explicit per-layer
operators and precision, architecture-aware state, tokenizer/template semantics,
independent full-vector reference fixtures and real server/state regressions.
Preserve existing working paths and original tolerances. No aliasing an
incompatible model to a supported dense graph or silently inventing missing
metadata.

Further architecture work is separate from this documentation/release
preparation; no roadmap item is a current capability. See
[architecture](ARCHITECTURE.md) and [certification](CERTIFICATION.md).
