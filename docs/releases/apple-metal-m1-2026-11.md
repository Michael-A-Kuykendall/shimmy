# Apple M1 Metal certification — unreleased candidate reconciliation

**Status:** Certified source candidate; **NOT a published Shimmy or Airframe release**. Prepared 2026-11-11.

## Proposed integration versions

Shimmy **2.6.7** consumes Airframe **0.4.6**. These versions are proposed and unpublished; the other four components retain their existing published versions. New source/archive identities and distribution acceptance must be verified before independent audit and human GO. No existing tag or archive is moved.

## Publication baseline

| Crate | Published version | Repository | Disposition |
| --- | --- | --- | --- |
| shimmy | 2.6.6 | Michael-A-Kuykendall/shimmy | Downstream new release needed to consume corrected engine |
| airframe | 0.4.5 | Michael-A-Kuykendall/airframe | New engine release needed |
| shimmytok | 0.8.5 | Michael-A-Kuykendall/shimmytok | Keep version; no demonstrated new tokenizer change |
| libfse | 0.1.4 | airframe/crates/libfse | Keep version; Apache-2.0 additional grant applies |
| airframe-observe | 0.1.4 | airframe/crates/airframe_observe | Keep version unless integration proves otherwise |
| shimmyjinja | 0.5.0 | Michael-A-Kuykendall/shimmyjinja | Keep version; no demonstrated new templating change |

Published Shimmy 2.6.6 resolves Airframe 0.4.5 via Cargo. The later private Metal correction is **not incorporated by Cargo version alone**. Never rewrite an old tag, archived crate, or its historical certification claim.

## Exact qualified implementation and evidence

- Engine source and tree identities: recorded in the maintainer's private qualification ledger; release integration must independently verify source equivalence.
- Combined acceptance result: seven numerical shards, 198/198 PASS, separately recorded in the maintainer's private ledger.
- Final 8K execution: [GitHub Actions run 42920826693](https://github.com/Michael-A-Kuykendall/shimmy/actions/runs/42920826693)
- Public [8K numerical test artifact](https://github.com/Michael-A-Kuykendall/shimmy/actions/runs/42920826693/artifacts/13820410710) includes the archived verification record.
- Tested environment: **Apple M1 (Virtual)**, Apple Paravirtual Metal GPU, macOS **15.8.2**.
- Comparisons: **198/198 PASS** across seven numerical shards; these are comparison rows and repeated generated positions, **not 198 independent prompts**.
- Frozen FP32 full-logit limits: max absolute **0.1**, RMS **0.03**; worst observed max absolute **0.0027675629**, RMS **0.0006875207**; all tested greedy decisions match.
- Combined acceptance includes state, reset, cancellation, recovery and actual HTTP JSON/SSE/usage/stop gates.
- Final untraced 8K ordinary production path completed; original 8K stall is historical and still lacks established cause.
- **Physical M4: NOT_RUN**. Qwen3.5-9B on M4: NOT_RUN. Intel/AMD/other GPU families are not certified by this run.

## Production changes isolated in the candidate

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
- No new crate, binary, GitHub release, Docker image, package-manager formula, or installer has been published by this document.

## Release handoff / mandatory checks

1. Transfer and reconcile the **exact qualified engine tree** into public Airframe, with independent diff and artifact verification.
2. Choose and publish an available new Airframe version, then update Shimmy's dependency and lockfile.
3. Rebuild actual platform artifacts and run release-install/real inference smoke checks on the precise distributables.
4. Synchronize accurate English/Chinese documentation, version histories, package notices and issue updates.
5. Publish only after independent package/source provenance acceptance. Preserve M4 as NOT_RUN.

Source of truth for model/fixture names and existing published guarantees: [SUPPORTED_MODELS](../SUPPORTED_MODELS.md) and [CERTIFICATION](../CERTIFICATION.md).