# Context capacity — Shimmy 2.6.7

--ctx and SHIMMY_MAX_CTX configure capacity subject to validated model metadata,
RoPE semantics and device limits. Increasing capacity does not enable arbitrary
YaRN scaling, guarantee coherent long-context output or certify that context.

The complete executed scope is the [fixture matrix](SUPPORTED_MODELS.md).
Selected Qwen3-0.6B modes cover512/2048/4096/8192, including final legal slots and
over-capacity rejection where recorded. Qwen3-1.7B is limited to the original
2032-token/context2048 case. Extended-context Llama and distinct alternate
prefill chunks remain NOT_RUN; do not extrapolate across combinations.

Requests beyond capacity must return an error, not silently truncate the prompt
or reuse stale state. Longer prompts also affect KV memory and initialization.
One successful final-slot test is not universal capacity/platform certification.

Leave INT4 and experimental kernels disabled. Prefill chunk overrides are not
a proven WSL/Windows timeout fix. Both WSL stability investigations remain open;
Apple M1 (Virtual) / Paravirtual Metal has 198/198 certified numerical comparisons for the exact published 2.6.7/0.4.6 fixtures. Physical M4 and other unexecuted native GPU platforms remain NOT_RUN.
See [configuration](CONFIGURATION.md), [certification](CERTIFICATION.md).
