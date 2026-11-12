# FAQ — Shimmy 2.6.7

**Which models work?** Only the exact executed dense fixtures and modes in
[SUPPORTED_MODELS](SUPPORTED_MODELS.md) have the recorded numerical evidence.
Discovery, tokenization and Q4_K_M parsing are not inference certification.

**Can I use Gemma, MoE or Qwen3.5 hybrid models?** Those executors are not enabled
by this dense foundation. Unsupported graphs must reject.

**Does fast mean a CPU inference engine?** No. The no-default-features fast
profile is for non-inference checks. Compatibility/backend flags do not restore
removed engines.

**Are Metal, Windows, Linux and WSL all certified?** No. The released 2.6.7/0.4.6 stack passed 198/198 full-logit comparisons on Apple M1 (Virtual) / Paravirtual Metal and bounded numerical checks on NVIDIA WSL. Native CLI/install validation across Linux, Windows and macOS is **not** general GPU correctness certification. Physical M4 and other unexecuted platforms remain NOT_RUN. Both WSL device-destruction and active-generation device-loss investigations remain open.

**Can I enable INT4 for lower memory?** It is outside the validated contract and
rejected. Use F32 KV/reference kernels; no compression or quality promise is made.

**Does 2.6.7 have native downloads, Docker, Homebrew and Scoop?** Yes. Use the [versioned download guide](BINARY_INSTALL.md) and [GHCR instructions](../README-DOCKER.md). GHCR `latest` and npm are unchanged.
See [quickstart](quickstart.md).

**Why does first use take longer?** Lazy loading and initial GPU work happen
before generation. Optional warm-up shifts work to startup. Neither a health
response nor adapter detection proves model readiness.

**Why is remote access rejected?** Non-loopback binds need a private API key;
browser origins must be explicitly allowed. See [configuration](CONFIGURATION.md).

**Why SSE instead of JSON?** Chat SSE requires stream:true; omitted/false returns
JSON. Text-completion streaming rejects. See [API](API.md).

**Can I submit a PR?** The project is open source, not open contribution.
Email first for co-maintainer approval; see [CONTRIBUTING](../CONTRIBUTING.md).
