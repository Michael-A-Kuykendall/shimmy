# Troubleshooting — Shimmy 2.6.6

Start with the installed version, exact model SHA-256, command, backend/adapter,
context/chunk/precision and sanitized error. Never publish keys, prompt data or
large captures inadvertently.

| Symptom | Check |
|---|---|
| Missing executable | cargo install shimmy --version 2.6.6 --locked; inspect PATH |
| Unknown model | GET /v1/models or shimmy list; use the exact registered ID |
| Invalid command | Commands include serve/list/discover/probe/bench/generate/gpu-info/init; generate needs positional NAME |
| Bind rejected | Non-loopback requires SHIMMY_API_KEY; explicit --bind controls listener |
|401 | Supply configured Bearer or x-api-key credentials |
|403 browser origin | Set exact SHIMMY_ALLOWED_ORIGINS; a valid key does not bypass origin checks |
| Unsupported model/plan | Check metadata and exact certified fixtures; do not alias families |
| Capacity error | Reduce prompt/output budget or use an already validated capacity; no silent truncation |
| No inference with fast | Build default Airframe engine; fast is non-inference |
| First request slow | Separate lazy loading/initialization from warm generation; warm-up shifts work |
| Driver loss/crash | Preserve logs and reproduction; WSL stability remains unresolved |

No generic instruction to enable INT4, bypass output normalization, force a
compatibility backend or enable experimental kernels is a supported remedy.
Do not claim a smaller prefill chunk fixes WSL device loss.
Metal/native GPU failures need real-platform evidence, not Docker emulation.

See [quickstart](quickstart.md), [API](API.md), [configuration](CONFIGURATION.md),
[fixture matrix](SUPPORTED_MODELS.md) and [security reporting](../SECURITY.md).
