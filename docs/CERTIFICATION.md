# Certification — Shimmy 2.6.6 / Airframe 0.4.5

The authoritative public scope is the [exact fixture matrix](SUPPORTED_MODELS.md).
A family label, parsed quantization, model load or matching one greedy token is
not numerical certification.

## Numerical gates

Compare identical injected token IDs and quantized model weights against an
independent dequantized-FP32 reference. Full logits must satisfy maximum absolute
error <=0.1 and RMS <=0.03; all measured greedy decisions must match.
Keep exact model/reference/source/precision/backend/kernel/context/chunk/mode
identities with the evidence. Repeated positions are not independent prompts.

## Server and state gates

Use the exact integrated candidate for JSON/chat SSE, actual generated content,
usage and finish reasons, stop boundaries, reset, cancellation, repeated requests,
capacity rejection, long-to-short isolation and typed error recovery.
Protocol-only mocks and numerical probes prove different contracts; neither
substitutes for real model-backed HTTP tests.

## Limits

The release retains selected Llama/Qwen fixtures only. Historical broad model
certification lists are not current support. Structural admission and discovery
are separate from executed numerical/platform results.

**The published 2.6.6 / Airframe 0.4.5 release** did not contain the later Metal correction; its Metal full-logit certification was NOT_RUN.

**Unreleased Apple M1 (Virtual) Metal candidate:** 198/198 full-vector numerical comparisons PASS across seven shards, with all measured greedy decisions agreeing with the frozen FP32 reference. Worst max absolute 0.0027675629 (limit 0.1); worst RMS 0.0006875207 (limit 0.03). The normal, untraced 8K production path completed the remaining 33 comparisons, with typed over-capacity rejection and short recovery. Combined acceptance included actual HTTP JSON/SSE, usage, stops, state isolation, reset, cancellation and recovery. See [exact candidate identities and evidence](releases/apple-metal-m1-2026-11.md). The earlier intermittently stalled 8K run remains preserved as a historical failure; its cause is unknown. Watchdog cleanup remains an independent open defect.

**Physical M4**, other unexecuted native GPU platforms, INT4, experimental kernels and unlisted combinations remain **NOT_RUN**. WSL destructor/device-loss
investigations remain open. No universal determinism, throughput or stability
guarantee is inferred from successful comparisons.

Documentation-only changes preserve numerical evidence only after explicit
source/dependency equivalence; a changed executable path needs targeted testing.
Public promotion requires human approval of exact new package/source hashes.
