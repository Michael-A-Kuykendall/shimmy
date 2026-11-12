# GPU pipeline — Airframe 0.4.6 in Shimmy 2.6.7

The dense engine executes metadata-validated per-layer operations through WGSL
and wgpu: normalization, projections, Q/K normalization and RoPE, attention,
FFN/residual paths and the output head. Weight decoders, intermediate scratch
and KV state must match the admitted tensor/operator contracts.

The frozen correction history includes request scratch ownership, synchronized
Q/K ownership and full tensor-span window planning. Those are correctness
corrections, not a new performance kernel or universal model certification.

F32 KV and reference kernels define the tested path. Experimental cooperative/
vectorized kernels, INT4, output-normalization bypass and unsupported routing
configurations reject in validated admission. Do not enable them as
troubleshooting advice.

Inspect actual stage/sample coverage using
[Airframe profiling](https://github.com/Michael-A-Kuykendall/airframe/blob/main/docs/PROFILING.md).
Timing and capture work can perturb execution; missing measurements do not
establish zero overhead. No host/GPU speed claim is made here.

See [certification](CERTIFICATION.md) and [exact fixtures](SUPPORTED_MODELS.md).
Apple M1 (Virtual) / Paravirtual Metal numerical and HTTP/state certification passed on the released engine; physical M4 and other unexecuted native GPU paths remain NOT_RUN. Both WSL stability issues remain open.
