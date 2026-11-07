# Roadmap — Shimmy 2.6.6

## Current delivered release

Deliver the coordinated dense-text correctness foundation and reviewed
dependency/documentation corrections. The release contract is
[SUPPORTED_MODELS](docs/SUPPORTED_MODELS.md), with explicit unsupported and
NOT_RUN modes. No broad family count or universal speed promise supersedes that contract. Commercial use is permitted under the [component licenses and additional libfse grant](LICENSING.md).

The five published versions are ShimmyTok 0.8.5, libfse 0.1.4,
airframe-observe 0.1.4, Airframe 0.4.5 and Shimmy 2.6.6. Their published archives and tags remain unchanged.

[Native downloads and custom Homebrew/Scoop installers](docs/BINARY_INSTALL.md) and the [versioned GHCR image](README-DOCKER.md) have separate executed distribution checks. These do not expand GPU numerical certification. Docker latest remains unchanged; npm is excluded.

## Separately scoped later work

Other architectures, larger fixtures, native-platform certification, WSL
stability, performance changes and additional distribution channels each need their own
design, acceptance evidence and approval. Gemma/MoE/recurrent development does
not automatically enable an executor in this release.

Both WSL destructor and active-generation device-loss investigations remain
open. Metal/M4, INT4 and experimental kernels are not certified. No target
date, guaranteed feature delivery or paid-service performance comparison is implied. Licensing rights are described in [stack licensing](LICENSING.md).

See [release process](RELEASE_PROCESS.md) and [historical v2 notes](docs/v2-roadmap.md).
