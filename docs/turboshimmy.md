# INT4/TurboShimmy status — Shimmy 2.6.7

INT4 KV code remains experimental and outside the validated release contract.
Validated admission rejects it. This guide no longer recommends enabling INT4,
claims equivalent quality or promises a particular VRAM/speed benefit.

The certified path is F32 KV with reference kernels on the
[exact fixtures](SUPPORTED_MODELS.md). The Q/K ownership correction does not
certify the separate INT4 path. Physical M4 and other unexecuted native GPU configurations, experimental kernels and
unlisted combinations remain NOT_RUN; WSL stability issues remain open.

Any future promotion needs independent full-vector/state/server evidence for
its exact precision, context, backend and model configuration. Historical
experiments are not current support claims or release evidence.
See [quantization](QUANTIZATION.md) and [certification](CERTIFICATION.md).
