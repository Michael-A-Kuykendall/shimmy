# Supported Models

Shimmy loads **GGUF**-format models and auto-detects the architecture from each
file's metadata — no hardcoded per-model constants, no recompilation. Context
window is read directly from the GGUF.

## Certified Models

The following model configurations have passed the full **5-gate certification**
pipeline (dequant + structural peel + numerical + decode≡prefill + logits),
validated on the RTX 3060 reference GPU. Certification records live in the
workspace ledger (`cert/math_ledger.duckdb`) and per-model packages under
`cert/packages/`. The README's **Supported Models** table is generated from this
same data, so the two stay in lockstep.

**11 model families · 25 certified model/quant combinations** — every model below passes Shimmy's 5-gate GPU math verification pipeline (dequant, structural peel, numerical, decode≡prefill, logits) against the certification ledger. GGUF files load as-is; no recompilation, no hardcoded per-model constants.

| Family | Model | Quants |
|---|---|---|
| **Llama** | [Llama-3.2-1B-Instruct](https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF) | Q4_K_M · Q6_K |
|  | [Llama-3.2-3B-Instruct](https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF) | Q4_K_M |
|  | [TinyLlama-1.1B-Chat](https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF) | Q4_0 · Q5_K_M · Q6_K |
| **Qwen3** | [Qwen3-0.6B](https://huggingface.co/Qwen/Qwen3-0.6B-GGUF) | Q4_K_M |
|  | [Qwen3-1.7B](https://huggingface.co/Qwen/Qwen3-1.7B-GGUF) | Q4_K_M |
|  | [Qwen3-4B](https://huggingface.co/Qwen/Qwen3-4B-GGUF) | Q4_K_M |
|  | [Qwen3-4B-Thinking](https://huggingface.co/Qwen/Qwen3-4B-Thinking-GGUF) | Q4_K_M |
|  | [Qwen3-8B](https://huggingface.co/Qwen/Qwen3-8B-GGUF) | Q4_K_M |
| **Qwen2** | [Qwen2-0.5B-Instruct](https://huggingface.co/Qwen/Qwen2-0.5B-Instruct-GGUF) | Q4_K_M |
|  | [Qwen2-1.5B-Instruct](https://huggingface.co/Qwen/Qwen2-1.5B-Instruct-GGUF) | Q4_K_M |
|  | [Qwen2-7B-Instruct](https://huggingface.co/Qwen/Qwen2-7B-Instruct-GGUF) | Q4_K_M |
| **Qwen3.5** | [Qwen3.5-9B](https://huggingface.co/Qwen/Qwen3.5-9B-GGUF) | Q4_K_M |
| **Phi-3** | [Phi-3.5-mini-Instruct](https://huggingface.co/microsoft/Phi-3.5-mini-instruct-gguf) | Q4_K_M |
|  | [Phi-3-mini-4k-Instruct](https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf) | Q4_0 |
| **Phi-2** | [Phi-2](https://huggingface.co/TheBloke/phi-2-GGUF) | Q4_K_M |
| **Gemma-2** | [Gemma-2-2B-it](https://huggingface.co/bartowski/gemma-2-2b-it-GGUF) | Q4_K_M |
|  | [Gemma-2-9B-it](https://huggingface.co/bartowski/gemma-2-9b-it-GGUF) | Q4_K_M |
| **Gemma-4** | [Gemma-4-12B-coder](https://huggingface.co/google/gemma-4-12B-coder-GGUF) | Q4_K_M |
|  | [Gemma-4-E4B](https://huggingface.co/google/gemma-4-E4B-it-GGUF) | Q4_K_M |
| **DeepSeek-R1** | [DeepSeek-R1-0528-Qwen3-8B](https://huggingface.co/deepseek-ai/DeepSeek-R1-Distill-Qwen-8B-GGUF) | Q4_K_M |
| **Ministral** | [Ministral-3-14B-Reasoning](https://huggingface.co/bartowski/Ministral-3-14B-Reasoning-GGUF) | Q4_K_M |
| **StarCoder2** | [StarCoder2-3B](https://huggingface.co/second-state/StarCoder2-3B-GGUF) | Q4_K_M |



> **GPU Math Verified** means the Airframe GPU dequantization shader produces
> results matching the CPU reference implementation, independently confirmed for
> every tensor type in each model via `quant_verify` (512 elements per quant
> type per model).

> Llama-3.2's native context is 131072 tokens. Airframe reads this from GGUF and
> allocates KV cache accordingly. Use `SHIMMY_MAX_CTX=8192` for a practical 8K
> window on consumer hardware (~256 MB KV cache for the 1B model).

## Supported Quantization Types

| Type | GGML ID | Notes |
|---|---|---|
| `F32` | 0 | Raw floats — maximum precision |
| `F16` | 1 | Half-precision floats |
| `Q4_0` | 2 | 4-bit, 32-element blocks |
| `Q5_0` | 6 | 5-bit, 32-element blocks |
| `Q8_0` | 8 | 8-bit, 32-element blocks |
| `Q4_K` | 12 | 4-bit K-quant superblocks (256 elements) — used in Q4_K_M GGUFs |
| `Q5_K` | 13 | 5-bit K-quant superblocks — used alongside Q4_K in mixed-precision models |
| `Q6_K` | 14 | 6-bit K-quant superblocks — typically used for output/embedding layers |

All types are implemented in both the GPU inference shader and a CPU reference
implementation. GPU vs CPU agreement is validated for every type. In practice,
any standard `Q4_K_M`, `Q5_K_M`, or `Q6_K` GGUF will load and run — those
formats mix Q4_K / Q5_K / Q6_K blocks across layers.

## Auto-Discovery

If Shimmy finds GGUF models in your HuggingFace cache, Ollama directory, LM
Studio cache (`~/.cache/lm-studio/models`), or local `./models/` folder, it
registers and serves them automatically. See
[docs/MODEL_EXPANSION.md](MODEL_EXPANSION.md) for the full onboarding protocol
and how to add a new architecture or quant type.
