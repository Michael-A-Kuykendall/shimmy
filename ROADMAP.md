# Roadmap — Shimmy 2.6.7 (released)

## Current release — November 12, 2026

**Shimmy 2.6.7 / Airframe 0.4.6** is published. The certified Apple M1 (Virtual) Metal corrections, frozen FP32 comparison limits, 198/198 numerical results and actual server/state acceptance carry through the qualified source. Verified native archives (Linux x86-64/ARM64, Windows x86-64, macOS ARM64), versioned GHCR 2.6.7, Homebrew and Scoop are available; `latest` and npm are unchanged. See [native downloads](docs/BINARY_INSTALL.md), [Docker](README-DOCKER.md), [model scope](docs/SUPPORTED_MODELS.md), and [release evidence](docs/releases/apple-metal-m1-2026-11.md).

Physical M4, Qwen3.5 and all other untested hardware/model configurations remain unverified. The older 2.6.6/0.4.5 release is retained historically.

## Prior release — Shimmy 2.6.6

Deliver the coordinated dense-text correctness foundation and reviewed
dependency/documentation corrections. The release contract is
[SUPPORTED_MODELS](docs/SUPPORTED_MODELS.md), with explicit unsupported and
NOT_RUN modes. No broad family count or universal speed promise supersedes that contract. Commercial use is permitted under the [component licenses and additional libfse grant](LICENSING.md).

The prior coordinated versions were ShimmyTok 0.8.5, libfse 0.1.4, airframe-observe 0.1.4, Airframe 0.4.5 and Shimmy 2.6.6. Only Airframe and Shimmy subsequently advanced to 0.4.6 and 2.6.7. Original archives and tags remain unchanged.

[Native downloads and custom Homebrew/Scoop installers](docs/BINARY_INSTALL.md) and the [versioned GHCR image](README-DOCKER.md) have separate executed distribution checks. These do not expand GPU numerical certification. Docker latest remains unchanged; npm is excluded.

## Separately scoped later work

Other architectures, larger fixtures, native-platform certification, WSL
stability, performance changes and additional distribution channels each need their own
design, acceptance evidence and approval. Gemma/MoE/recurrent development does
not automatically enable an executor in this release.

Both WSL destructor and active-generation device-loss investigations remain open. The **released** Apple M1 (Virtual) Metal path passed 198/198 full-vector numerical comparisons and combined HTTP/state acceptance; physical M4, Qwen3.5 inference and other unexecuted configurations are NOT_RUN/unsupported. Historical intermittent 8K stall cause and watchdog cleanup remain unresolved. No target
date, guaranteed feature delivery or paid-service performance comparison is implied. Licensing rights are described in [stack licensing](LICENSING.md).

See [release process](RELEASE_PROCESS.md) and [historical v2 notes](docs/v2-roadmap.md).
