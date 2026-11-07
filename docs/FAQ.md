# FAQ — Shimmy 2.6.6

**Which models work?** Only the exact executed dense fixtures and modes in
[SUPPORTED_MODELS](SUPPORTED_MODELS.md) have the recorded numerical evidence.
Discovery, tokenization and Q4_K_M parsing are not inference certification.

**Can I use Gemma, MoE or Qwen3.5 hybrid models?** Those executors are not enabled
by this dense foundation. Unsupported graphs must reject.

**Does fast mean a CPU inference engine?** No. The no-default-features fast
profile is for non-inference checks. Compatibility/backend flags do not restore
removed engines.

**Are Metal, Windows, Linux and WSL all certified?** No. NVIDIA WSL numerical
evidence does not certify other native platforms; Metal/M4 is NOT_RUN. Both
WSL device-destruction and active-generation device-loss investigations remain
open.

**Can I enable INT4 for lower memory?** It is outside the validated contract and
rejected. Use F32 KV/reference kernels; no compression or quality promise is made.

**Will 2.6.6 update Docker or my package manager?** A crates.io-only publication
does not create binaries, images, Homebrew, Scoop or npm updates.
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
