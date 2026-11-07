<div align="center">
  <img src="https://raw.githubusercontent.com/Michael-A-Kuykendall/shimmy/main/assets/shimmy-logo.png" alt="Shimmy Logo" width="300" height="auto" />

  # Shimmy — Local Inference, OpenAI-Compatible

  ### 🔒 Local GPU inference — Rust/WGSL application 🚀

  [![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)
  [![Security](https://img.shields.io/badge/Security-profile--reviewed-blue)](https://github.com/Michael-A-Kuykendall/shimmy/security)
  [![CI](https://github.com/Michael-A-Kuykendall/shimmy/actions/workflows/ci.yml/badge.svg)](https://github.com/Michael-A-Kuykendall/shimmy/actions/workflows/ci.yml)
  [![Crates.io](https://img.shields.io/crates/v/shimmy.svg)](https://crates.io/crates/shimmy)
  [![Downloads](https://img.shields.io/crates/d/shimmy.svg)](https://crates.io/crates/shimmy)
  [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
  [![GitHub Stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=social)](https://github.com/Michael-A-Kuykendall/shimmy/stargazers)
  [![💝 Sponsor this project](https://img.shields.io/badge/💝_Sponsor_this_project-ea4aaa?style=for-the-badge&logo=github&logoColor=white)](https://github.com/sponsors/Michael-A-Kuykendall)
  [![Trans rights](https://pride-badges.pony.workers.dev/static/v1?label=trans%20rights&stripeWidth=6&stripeColors=5BCEFA,F5A9B8,FFFFFF,F5A9B8,5BCEFA)](https://translifeline.org/)
  [![LGBTQ+ friendly](https://pride-badges.pony.workers.dev/static/v1?label=lgbtq%2B%20friendly&stripeWidth=6&stripeColors=E40303,FF8C00,FFED00,008026,24408E,732982)](https://www.thetrevorproject.org/)

</div>

## Shimmy 2.6.6

Shimmy is a local inference server and CLI; **Airframe 0.4.5 is its engine**.
This is a coordinated dense-text correctness, dependency-security and
documentation release, not a new architecture or performance release.
See [CHANGELOG](CHANGELOG.md).

**Default dependency licensing:** Shimmy → Airframe → libfse. The default
build includes libfse 0.1.4 under the FSE Noncommercial Evaluation License.
Commercial use or embedding of that distribution requires separate permission
for libfse, as do derivative works under its terms. Shimmy's Apache-2.0 license
does not waive those restrictions; the complete default distribution is not
unrestricted Apache-2.0 open source. See [libfse licensing](https://github.com/Michael-A-Kuykendall/airframe/blob/main/crates/libfse/LICENSE).

## Install and run

Install the versioned crate with a Rust toolchain:

```bash
cargo install shimmy --version 2.6.6 --locked
shimmy --version
shimmy --ctx 512 --kv-quant f32 serve --model-path /path/to/model.gguf --bind 127.0.0.1:11435
```

Use a verified admitted fixture in place of the path. A working Rust toolchain
and GPU driver are required. The fast profile does not provide inference.
A **crates.io-only 2.6.6 release does not create GitHub binary assets, Docker
images, Homebrew, Scoop or npm packages**. Verify the installed version; an old latest asset is not version 2.6.6.

See [quickstart](docs/quickstart.md) for model hashes, listing, requests and
network access. Unavailable translations are not linked as current guides.

## Correctness and support

The [exact fixture matrix](docs/SUPPORTED_MODELS.md) covers selected
Qwen3-0.6B/Qwen3-1.7B and Llama 3.2 3B Q4_K_M cases, with independent FP32 full
logits, maximum absolute 0.1/RMS 0.03 gates and matching measured greedy decisions.
These are bounded model/context/mode results, not universal family support.

F32 KV and reference kernels define the validated path. Parsing metadata,
tokenizers or quantization is not certification. Gemma2, MoE, MLA, recurrent,
state-space and multimodal executors are not enabled.
INT4/TurboShimmy and experimental kernels remain excluded and rejected.

Executed GPU evidence is NVIDIA through WSL Dozen/Vulkan. **NOT_RUN:** Metal/M4,
other unexecuted native GPU platforms, extended-context Llama, alternate
prefill chunks and unlisted combinations. WSL device-destruction and
active-generation device-loss investigations remain open.
No universal determinism, startup, multi-GPU, memory or speed guarantee is made.

## Server contract

Selected OpenAI-compatible endpoints provide model listing, JSON chat/text
completions and explicit **chat-only SSE**. Chat defaults to JSON; text
streaming rejects. Usage, stop/finish metadata and typed errors are part of
the tested contract. This is not every OpenAI endpoint/SDK feature.

Loopback can run without a key. Non-loopback binds require SHIMMY_API_KEY;
browser origins require SHIMMY_ALLOWED_ORIGINS. With a key, routes except
GET /health require credentials; origin checks still apply to health.
Use TLS and appropriate network access controls remotely.
Lazy loading is the default; optional selected-model warm-up shifts cold work
before readiness without eliminating it or guaranteeing stability.

## Documentation

- [Quickstart](docs/quickstart.md) and [configuration](docs/CONFIGURATION.md)
- [API](docs/API.md), [compatibility](docs/OPENAI_COMPAT.md) and [integration](docs/INTEGRATION.md)
- [Model scope](docs/SUPPORTED_MODELS.md), [certification](docs/CERTIFICATION.md) and [quantization](docs/QUANTIZATION.md)
- [Context capacity](docs/EXTENDED_CONTEXT.md) and [templates](docs/CHAT_TEMPLATES.md)
- [Docker scope](README-DOCKER.md), [Windows scope](docs/WINDOWS_GPU_BUILD_GUIDE.md) and [troubleshooting](docs/TROUBLESHOOTING.md)
- [Security](SECURITY.md), [release process](RELEASE_PROCESS.md) and [release gates](RELEASE_GATES_CHECKLIST.md)
- [Developer guide](DEVELOPERS.md), [contribution policy](CONTRIBUTING.md) and [roadmap](ROADMAP.md)

Shimmy source is Apache-2.0. Airframe runtime is MIT. libfse retains its
FSE Noncommercial Evaluation License: commercial use, product embedding and
derivatives require separate permission. Public source availability and
sponsorship do not waive component/model licenses.

## Support

This project is independently maintained. Component licensing terms described above apply. Sponsorship helps fund ongoing development, certification, compatibility work, and releases.

[Sponsor on GitHub](https://github.com/sponsors/Michael-A-Kuykendall) · [Buy Me a Coffee](https://www.buymeacoffee.com/michaelakuykendall)

Any amount is appreciated. Suggested tiers:

- **$5/month** — Thanks
- **$15/month** — Supporter
- **$50/month** — Backer
- **Custom** — [Email for invoicing](mailto:michaelallenkuykendall@gmail.com)

---

## License & Philosophy

Shimmy server code: Apache-2.0 — see [LICENSE](LICENSE). Airframe and libfse retain their separate licensing terms; commercial deployments using libfse require separate permission.

**Promise**: This will never become a paid product.

Shimmy is infrastructure: it should be invisible. Reliability through comprehensive validation and property-based testing.

---

**Maintainer**: Michael A. Kuykendall · **Mission**: Making local model inference simple and reliable

---

## Support

This project is a safe space. Trans rights are human rights.

If you or someone you love needs support:

- [The Trevor Project](https://www.thetrevorproject.org/) — 24/7 for LGBTQ+ young people. Call 1-866-488-7386 or text START to 678-678
- [Trans Lifeline](https://translifeline.org/) — peer support run by and for trans people. US: 877-565-8860
- [988 Suicide & Crisis Lifeline](https://988lifeline.org/) — call or text 988
