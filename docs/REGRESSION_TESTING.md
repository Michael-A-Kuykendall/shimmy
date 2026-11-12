# Regression testing — Shimmy 2.6.7

Use the exact integrated sources and locked dependency graph. Keep unit/protocol,
numerical, state, real HTTP and platform evidence distinct.

```bash
cargo test --locked --lib -- --test-threads=1
cargo test --locked --no-default-features --features fast --lib -- --test-threads=1
cargo clippy --locked --all-targets -- -D warnings
```

These are quality checks, not sufficient model certification. Numerical checks
need exact fixture hashes/IDs and independent FP32 logits under original
max-absolute 0.1/RMS 0.03 bounds. Real HTTP checks must assert generated content,
JSON/chat SSE, usage/stops/finish, warm-up, errors and state isolation.
Record omitted modes as NOT_RUN; do not substitute mocks or greedy-only output.

Documentation-only edits do not justify expensive GPU matrices when executable
source and verified dependencies are proven equivalent. Changed help/diagnostic
strings need targeted CLI/build/server verification, not new math claims.
See [certification](CERTIFICATION.md), [methodology](METHODOLOGY.md) and
[release gates](../RELEASE_GATES_CHECKLIST.md).
