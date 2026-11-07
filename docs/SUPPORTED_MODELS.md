# Supported models — Shimmy 2.6.6 / Airframe 0.4.5

This release is scoped to the validated dense Llama and Qwen3 execution plans.
Model discovery, structural admission and numerical certification are distinct.
Discovering a GGUF file does not certify its architecture, quantization or platform.
Missing required metadata and unsupported execution graphs reject explicitly.

## Executed numerical coverage

The independent reference uses the same quantized weights dequantized to FP32,
identical injected token IDs and full logit vectors. Greedy decisions must match;
maximum absolute error must be at most 0.1 and RMS error at most 0.03.

| Exact GGUF SHA-256 | Model / quantization | Executed context / modes |
|---|---|---|
| `ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a` | Qwen3-0.6B Q4_K_M | 512, 2048, 4096, 8192; selected text/injected/repeat and final-slot cases |
| `72c5c3cb38fa32d5256e2fe30d03e7a64c6c79e668ad84057e3bd66e250b24fb` | Qwen3-1.7B Q4_K_M | Original 2032-token request at context 2048; capture off/on/repeat |
| `dde5aa3fc5ffc17176b5e8bdc82f587b24b2678c6c66101bf7da77af9f7ccdff` | Llama 3.2 3B Instruct Q4_K_M | Context 512; selected text/injected/repeat cases |

These are bounded fixtures, not certification of every prompt, model revision or
combination of settings. The 198 measured comparisons include repeated positions
and modes; they are not 198 independent prompts.

Executed GPU environment: NVIDIA RTX 4070 Laptop through WSL Dozen/Vulkan,
FP32 accumulation, F32 KV, reference kernels. Numerical correctness on this
environment does not establish native Linux, Windows or Apple GPU correctness.
WSL device destruction and active-generation device-loss investigations remain
open. A successful run does not establish that those failures are fixed.

Actual server coverage includes JSON/SSE, usage/finish reasons, state isolation,
reset/cancellation, typed errors and recovery on selected Qwen3/Llama fixtures.
Qwen3-1.7B coverage is the exact context-2048 JSON request; its other contexts and
SSE remain NOT_RUN. Extended-context Llama and distinct alternate prefill chunks
remain NOT_RUN. The release validation record identifies the exact binary used.

## Excluded or unexecuted configurations

Metal/M4 and other unavailable native GPU platforms: **NOT_RUN**.
INT4 KV and experimental kernels: **NOT_RUN**, disabled for this candidate.
Gemma2 and other new architecture executors: not enabled by this dense foundation.
MoE, MLA, hybrid recurrent/Gated DeltaNet, state-space and multimodal execution:
unsupported by the validated dense plan and must reject explicitly.
Other models and quantizations are not certified by the three fixtures above.
Do not use historical family-wide support lists as release certification.

GGUF parsing and tokenizer support alone do not establish inference support.
SafeTensors parsing does not establish a certified SafeTensors inference path.
Use `SHIMMY_MAX_CTX` or `--ctx` within the selected model's validated capacity;
a configured capacity is not evidence of numerical certification at that size.
