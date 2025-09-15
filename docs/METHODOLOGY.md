# Engineering Methodology

Shimmy was built **spec-first**, **test-driven**, and **AI‑assisted**. This document records the exact loop, the quality gates, and where to find proofs.

---

## Development Loop

1. **Define a contract/spec**  
   Example: “Implement `/v1/chat/completions` with streaming (Server‑Sent Events) and match the response schema.”

2. **Generate a candidate implementation**  
   AI tools scaffold code; every line is reviewed before commit. Nontrivial changes are tied to a spec (issue or PR description) and include tests.

3. **Validate with properties & invariants**  
   - Property‑based tests: see [`docs/ppt-invariant-testing.md`](./ppt-invariant-testing.md).  
   - Runtime invariants: assertions on protocol, state, and memory safety expectations.  
   - Tests live under `/tests` and run in CI on Linux/macOS/Windows.

4. **CI Gates**  
   Every PR runs:  
   - DCO sign‑off  
   - Build matrix (Linux/macOS/Windows)  
   - Unit + property tests  
   - Static checks / duplicate issue detection  
   - Release workflow dry‑run (where applicable)

5. **Iterate until green**  
   Code merges only when all gates pass. Releases are signed/tagged and changelogged.

---

## Quality Practices

- **Property Testing**: Exercise edge cases beyond example‑based tests.  
- **Runtime Invariants**: Fail fast when correctness assumptions are violated.  
- **Benchmarks**: Reproducible scripts and environment in [`docs/BENCHMARKS.md`](./BENCHMARKS.md).  
- **OpenAI Compat**: Supported endpoints/fields in [`docs/OPENAI_COMPAT.md`](./OPENAI_COMPAT.md).  
- **Security Defaults**:  
  - Binds to `127.0.0.1` by default.  
  - External model files are **trust‑on‑first‑use**; optional SHA‑256 verification and allow‑list paths are available/planned.  
  - Prefer running with least privilege; avoid exposing ports publicly without auth.

---

## Philosophy

- **Spec first, code second** — logic/contracts drive implementation.  
- **Tests > syntax** — correctness is proven with properties/invariants.  
- **AI is a tool; process is the product** — the methodology scales teams.  
- **Forever‑free core** — MIT license; contributions via Issues/PRs are welcome.

---

## Quick Links

- Property/invariant guide: [`docs/ppt-invariant-testing.md`](./ppt-invariant-testing.md)  
- Tests: [`/tests`](../tests)  
- CI: GitHub Actions → _CI status badge in README_  
- Benchmarks: [`docs/BENCHMARKS.md`](./BENCHMARKS.md)  
- OpenAI Compatibility: [`docs/OPENAI_COMPAT.md`](./OPENAI_COMPAT.md)
