# Configuration — Shimmy 2.6.6

## HTTP access policy

`SHIMMY_API_KEY` must contain at least 16 printable ASCII characters without
spaces. It is required on non-loopback binds. When configured, authenticate
using Authorization: Bearer or Anthropic's x-api-key header.
`SHIMMY_ALLOWED_ORIGINS` is a comma-separated list of exact HTTP(S) browser
origins. Empty is the default; wildcards and paths are rejected.

GET /health is credential-exempt. Disallowed Origin headers are rejected before
route/auth processing, including health. Allowed preflight OPTIONS returns 204;
OPTIONS without Origin returns 400. Other routes require credentials when a key
is configured. Use TLS at a reverse proxy for remote access.

## Model selection and binding

| Setting | Actual behavior |
|---|---|
| `--model-dirs` | Additional semicolon-separated discovery directories |
| `SHIMMY_BASE_GGUF` | Optional explicit model-file registration |
| `serve --model-path` | Direct existing model file, registered using its filename stem |
| `serve --bind` | Explicit listener address, or auto allocation |
| `SHIMMY_BIND_ADDRESS` | Default address consulted when bind is auto |
| `--ctx` / `SHIMMY_MAX_CTX` | Registry capacity defaults to 2048 and is capped at model-native context |
| `--kv-quant f32` | Validated release precision and CLI default |
| `--prefill-chunk` / `SHIMMY_PREFILL_CHUNK` | Chunk override; distinct values need their own certification |
| `RUST_LOG` | tracing filter |

SHIMMY_MAX_CTX accepts values 512 through 131072; unset, invalid or out-of-range values fall back to 2048. The CLI --ctx is routed through this registry setting. Airframe uses the smaller of that capacity and the model-native context. This is not arbitrary context scaling or a certification claim.

Obsolete, unsupported keys: `SHIMMY_PORT`, `SHIMMY_HOST` and `SHIMMY_LOG_LEVEL`.
Use explicit bind and RUST_LOG instead. Model discovery is not execution
admission. An environment variable retained by compatibility code does not
establish LoRA, CPU or MoE inference support.

The `--gpu-backend`, CPU-MoE and legacy options are compatibility surfaces,
not a supported backend switch or new executor. Default Airframe selects its
wgpu adapter independently. INT4 is retained as an option but rejected by
validated admission. Leave experimental flags and output-normalization bypass
options unset. A context override is not YaRN enablement or certification.

## Startup warm-up

Models are lazy-loaded by default. `SHIMMY_WARMUP=1` runs a fixed small prompt
with a one-output-token budget before readiness:

```sh
SHIMMY_WARMUP=1 shimmy --ctx 512 --kv-quant f32 serve --model-path /path/to/model.gguf
```

`SHIMMY_WARMUP_MODEL` selects one discovered name if several models exist.
An explicit model path takes precedence; a conflicting selector errors.
Otherwise selection uses the explicit selector, then SHIMMY_BASE_GGUF, then the
sole available model. It does not load every discovered file.
Accepted values are 1/true and 0/false, with case-insensitive words; unset is
disabled. Invalid settings, unsupported backends, initialization or reset
failures stop startup before readiness.

The same engine handles warm-up and subsequent requests; weights remain
resident and KV request state is reset. One model file is resident per engine.
A different file rejects rather than silently using the resident model;
restart to change files. Automatic eviction/hot replacement is not supported.

Warm-up shifts initialization to startup, not guaranteed faster execution or a
driver-stability fix. See [profiling](https://github.com/Michael-A-Kuykendall/airframe/blob/main/docs/PROFILING.md).

## Inspect the installed CLI

```bash
shimmy --help
shimmy serve --help
shimmy generate --help
```

Generation takes a positional registered model name:
`shimmy generate MODEL_NAME --prompt "Hello" --max-tokens 8`.
See [API](API.md), [quickstart](quickstart.md) and [context limits](EXTENDED_CONTEXT.md).
