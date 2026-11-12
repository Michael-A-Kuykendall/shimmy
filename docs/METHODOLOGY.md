# Verification methodology — Shimmy 2.6.7

Certification is scoped to immutable model/source/reference/configuration
identities, not a count of supported families.

1. Validate metadata/tensor spans and required architecture/operators.
2. Preserve exact template rendering and injected input token IDs.
3. Compare full logits to independent arithmetic using the same quantized
   weights dequantized to FP32: max absolute <=0.1, RMS <=0.03, matching greedy
   decisions. Keep reference precision explicit.
4. Verify repeated execution, state isolation, reset/cancellation, capacity
   boundaries and invalid-input recovery on the integrated candidate.
5. Exercise actual HTTP content and protocol accounting, not only mock engines.
6. Retain source, dependency, binary and evidence hashes with exact modes.

Matching one token, compilation or loading cannot establish these contracts.
Do not relax bounds to accommodate a failure or conflate quantized-CPU rounding
with an independent FP32 reference. Repeated positions are not independent
prompts. Unexecuted modes/platforms remain NOT_RUN.

Current bounded results are in [SUPPORTED_MODELS](SUPPORTED_MODELS.md).
No old family-wide certification list or performance experiment expands that
matrix. See [release gates](../RELEASE_GATES_CHECKLIST.md).
