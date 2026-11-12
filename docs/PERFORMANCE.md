# Performance scope — Shimmy 2.6.7

This is a dense-text correctness and dependency/documentation release.
It establishes no universal throughput, startup, binary-size, RAM, VRAM, power,
multi-GPU or competitor-comparison guarantee. Old benchmark/INT4 claims do not
define the supported release contract.

Model loading is lazy unless warm-up is selected. Warm-up moves loading and
initial forward work before readiness; it does not remove initialization or
prove faster steady-state inference. /health does not prove model readiness.

For measurements, record exact model hash/token IDs, source, binary, backend,
precision, kernels, context/chunk and machine. Separate cold process startup,
cold HTTP, warm generation and GPU stage timing. Avoid overlapping heavyweight
jobs; missing timestamps are not zero-cost work.
See [Airframe profiling](https://github.com/Michael-A-Kuykendall/airframe/blob/main/docs/PROFILING.md).

Performance experiments remain deferred. INT4 and experimental kernels are
uncertified; WSL driver stability is unresolved, and Metal/native GPU results
are NOT_RUN unless separately executed. See [fixture scope](SUPPORTED_MODELS.md).
