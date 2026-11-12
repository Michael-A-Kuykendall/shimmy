# Apple M1 Metal certification — released in Shimmy 2.6.7 / Airframe 0.4.6

**Status:** Exact-source certification PASS and published November 12, 2026 in [Shimmy v2.6.7](https://github.com/Michael-A-Kuykendall/shimmy/releases/tag/v2.6.7) and [Airframe v0.4.6](https://github.com/Michael-A-Kuykendall/airframe/releases/tag/v0.4.6). Physical M4 remains NOT_RUN.

## Published integration versions

Shimmy **2.6.7** consumes published Airframe **0.4.6**. Only these two crates required new versions; ShimmyTok 0.8.5, libfse 0.1.4, airframe-observe 0.1.4 and shimmyjinja 0.5.0 remain unchanged. [Native installation and hashes](../BINARY_INSTALL.md) · [Versioned Docker image](../../README-DOCKER.md). No prior tag or archive was moved.

## Previous publication baseline (historical)

| Crate | Published version | Repository | Disposition |
| --- | --- | --- | --- |
| shimmy | 2.6.6 | Michael-A-Kuykendall/shimmy | Historical: replaced by released 2.6.7 |
| airframe | 0.4.5 | Michael-A-Kuykendall/airframe | Historical: replaced by released 0.4.6 |
| shimmytok | 0.8.5 | Michael-A-Kuykendall/shimmytok | Keep version; no demonstrated new tokenizer change |
| libfse | 0.1.4 | airframe/crates/libfse | Keep version; Apache-2.0 additional grant applies |
| airframe-observe | 0.1.4 | airframe/crates/airframe_observe | Keep version unless integration proves otherwise |
| shimmyjinja | 0.5.0 | Michael-A-Kuykendall/shimmyjinja | Keep version; no demonstrated new templating change |

Historical Shimmy 2.6.6 resolves Airframe 0.4.5 via Cargo and does **not** contain the Metal correction. Published Shimmy 2.6.7 resolves Airframe 0.4.6 and includes the qualified correction. Historical archives and certification remain unchanged.

## Exact qualified implementation and evidence

- Engine certification and source equivalence were independently accepted before publication; exact internal qualification identities are recorded in the maintainer's private evidence ledger.
- Combined acceptance result: seven numerical shards, 198/198 PASS, separately recorded in the maintainer's private ledger.
- Final 8K execution: [GitHub Actions run 42920826693](https://github.com/Michael-A-Kuykendall/shimmy/actions/runs/42920826693)
- Public [8K numerical test artifact](https://github.com/Michael-A-Kuykendall/shimmy/actions/runs/42920826693/artifacts/13820410710) includes the archived verification record.
- Tested environment: **Apple M1 (Virtual)**, Apple Paravirtual Metal GPU, macOS **15.8.2**.
- Comparisons: **198/198 PASS** across seven numerical shards; these are comparison rows and repeated generated positions, **not 198 independent prompts**.
- Frozen FP32 full-logit limits: max absolute **0.1**, RMS **0.03**; worst observed max absolute **0.0027675629**, RMS **0.0006875207**; all tested greedy decisions match.
- Combined acceptance includes state, reset, cancellation, recovery and actual HTTP JSON/SSE/usage/stop gates.
- Final untraced 8K ordinary production path completed; original 8K stall is historical and still lacks established cause.
- **Physical M4: NOT_RUN**. Qwen3.5-9B on M4: NOT_RUN. Intel/AMD/other GPU families are not certified by this run.

## Production corrections included in the release

1. Metal queue completion after large model initialization and before output-head allocation: addresses all-zero Qwen3-1.7B embeddings.
2. macOS outer dense prefill bound to 64 tokens: passes 12 tested Llama full-logit comparisons after native Metal timeout.
3. No speculative 8K kernel/scheduler correction: a subsequent untraced execution passed the original 8K contract. Historical intermittent stall remains unexplained.

Neither change expands supported architectures or changes frozen numerical gates.

## Open maintenance and release boundaries

- Watchdog terminal cleanup warning remains a separate open issue; it is not the MiniCPM-V/Vision GPU cleanup investigation.
- WSL active-generation device loss/destructor instability remain outside this Apple M1 correctness result.
- macOS Paste tool maintenance warning, experimental kernels, INT4 KV, extended-context Llama, M4 physical hardware, other Qwen3.5 combinations remain unverified.
- Shimmy #218 M4 Max original reproduction must **not** be closed solely on this M1 run.
- Shimmy #214 binding-limit fix is recorded in 2.6.3, but reported INT4/Qwen3.5 configuration still needs separate confirmation.
- Shimmy 2.6.7 and Airframe 0.4.6 crates, GitHub Releases, versioned GHCR 2.6.7, Homebrew and Scoop were published and verified. The document is an evidence record; it does not itself perform those operations.

## Completed publication verification

The accepted source-to-package equivalence, four native distribution targets, actual installed CLI, container and package-manager gates, and release artifacts were verified before publication. Physical M4 and other unexecuted hardware remain explicitly NOT_RUN. No additional model or numerical certification is claimed.

Source of truth for model/fixture names and existing published guarantees: [SUPPORTED_MODELS](../SUPPORTED_MODELS.md) and [CERTIFICATION](../CERTIFICATION.md).