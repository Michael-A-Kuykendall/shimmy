# Chat templates — Shimmy 2.6.7

A template converts ordered messages into model input before tokenization.
Correct role/control markers, special-token handling and BOS/EOS policies are
part of the exact model contract, not interchangeable family decorations.

Validated Airframe requests use the admitted model's strict template/tokenizer
contract and preserve prepared token IDs through generation. A missing required
template or unsupported rendering contract must not silently choose an
incompatible family fallback. Compatibility fallback code elsewhere is not
certification of Gemma, MoE or arbitrary named models.

HTTP chat and completion prompts are rendered once. CLI generate accepts a
positional model name and can explicitly request raw mode; that changes the
input contract and must be tested separately. Do not add an undocumented
per-request template override or assume every Jinja feature is supported.

ShimmyJinja 0.5.0 renders supported GGUF Jinja templates; ShimmyTok 0.8.5 handles
the tokenizer/special-token contract. See
[templating architecture](CHAT_TEMPLATING.md), [API](API.md) and the
[exact fixture/mode matrix](SUPPORTED_MODELS.md). Template parsing alone does
not certify the inference graph.
