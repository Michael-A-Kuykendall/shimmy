# Architecture — Shimmy 2.6.6 / Airframe 0.4.5

Shimmy owns discovery, CLI/HTTP protocols, access policy, prompt preparation and
engine integration. Airframe owns validated dense descriptors and per-layer
execution, GPU operators, sampling and request state. ShimmyTok supplies the
shared checked GGUF manifest and tokenizer APIs; ShimmyJinja renders supported
model templates.

The validated execution path admits dense Llama/Qwen3 contracts only, preserving
required metadata, tensor bounds, RoPE semantics and precision. Missing required
metadata and incompatible graphs reject; discovery is not admission.
Architecture names or quantization parsers must not silently alias unsupported
operations to existing dense kernels.

Request-owned state guards generation/reset/cancellation. The engine retains
one model file; aliases to the same canonical file can reuse it. Different-file
hot replacement and multi-model eviction are not supported.

airframe-observe uses Dzero for inference facts, observation and caller
coordination. libfse scans policy bytes/actions under separate license terms.
Neither proves GPU numerical correctness or guarantees whole-system enforcement.

The release preserves exact full-logit reference evidence and fixture limits;
it adds no new model family, performance executor or native GPU certification.
See [fixtures](SUPPORTED_MODELS.md), [templating](CHAT_TEMPLATING.md),
[API](API.md) and [configuration](CONFIGURATION.md).
