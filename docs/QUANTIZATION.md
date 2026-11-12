# Quantization — Shimmy 2.6.7

Distinguish **weight quantization**, **KV precision** and **executed model
certification**. A decoder's existence does not prove correct execution of all
models or tensor layouts bearing that quantization.

Airframe contains F32/F16 and Q4_0/Q5_0/Q8_0/Q4_K/Q5_K/Q6_K decoding code.
The coordinated release's executed fixtures use specific Q4_K_M files listed by
SHA-256 in [SUPPORTED_MODELS](SUPPORTED_MODELS.md). Mixed quantization names do
not establish a new architecture executor.

The validated path uses **F32 KV and reference kernels**.
INT4/TurboShimmy and cooperative/vectorized experimental kernels remain
uncertified, disabled and rejected by validated admission. Historical VRAM,
quality and speed figures are not release promises. Do not enable experimental
flags to bypass that contract.

Use --kv-quant f32. Neither successful parsing, model loading nor a short
plausible response replaces full-logit FP32-reference comparisons. See
[certification](CERTIFICATION.md) and [configuration](CONFIGURATION.md).
