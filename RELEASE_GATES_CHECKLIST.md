# Release gates — Shimmy 2.6.7 and future coordinated releases

The accepted 2.6.7 release is the exact coordinated six-component source/archive set (only Airframe and Shimmy were newly published),
not a successful generic build or old workflow checklist.

- Default engine and no-default-features fast graphs: locked resolution,
  official sources, approved security versions and applicable audit findings.
- Formatting, strict Clippy, relevant default/fast unit and protocol tests;
  fresh packaged-source release build and clean external consumer resolution.
- Exact versions, upstream dependency checksums, license/patent/attribution
  contents, README/changelog/migration/errata and all local links.
- Source-equivalence proof to the frozen inference baseline. Re-execute numerical
  tests only if a changed executable/dependency path requires targeted regression.
- Real server JSON/chat SSE, content, usage, finish/stop behavior, typed errors,
  cancellation recovery, warm-up and request isolation on declared fixtures.
- Accurate support matrix: unavailable modes/platforms remain NOT_RUN. WSL
  destruction/device-loss issues remain open. No INT4 or experimental promotion.
- Clean public-source diffs exclude private audits, credentials, model weights,
  agent state and local workspace material.
- Exact publication request/archive/source/binary hashes, live registry/tag/
  ancestry checks and explicit human authorization.

## Local quality commands

```bash
cargo fmt --check
cargo test --locked --lib -- --test-threads=1
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --no-default-features --features fast --lib -- --test-threads=1
cargo clippy --locked --no-default-features --features fast --all-targets -- -D warnings
cargo build --locked --release
```

These commands are quality checks, not the entire release acceptance gate.
There is no current huggingface feature or constitutional binary-size promise.
The fast build excludes the inference engine; it is not CPU numerical evidence.

No public push, tag, hosted Actions, upload or distribution is authorized by
checking this list. See [release process](RELEASE_PROCESS.md).
