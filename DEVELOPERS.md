# Developer guide — Shimmy 2.6.6

Read [architecture](docs/ARCHITECTURE.md), [configuration](docs/CONFIGURATION.md),
[API](docs/API.md) and [contribution policy](CONTRIBUTING.md) first.

## Local checks

```bash
cargo fmt --check
cargo test --locked --lib -- --test-threads=1
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --no-default-features --features fast --lib -- --test-threads=1
cargo build --locked --release
```

Default features compile Airframe 0.4.5. The fast profile excludes inference;
legacy feature names are compatibility stubs. Use locked official dependencies
for release verification. Temporary source patches are development aids, not
proof that public registry consumers resolve the approved packages.

## Manual use

```bash
shimmy list
shimmy generate MODEL_NAME --prompt "Hello" --max-tokens 8
shimmy --ctx 512 --kv-quant f32 serve --model-path /path/to/model.gguf --bind 127.0.0.1:11435
```

Replace model names/paths with exact fixtures. Generation takes a positional
name; there is no generate --name or generic run/chat command.
Real numerical/state/server checks need the exact combined candidate and
reference conditions. See [certification](docs/CERTIFICATION.md).
Do not reopen passing math tests for documentation-only edits with explicit
source equivalence; changed executable paths require targeted checks.

Only maintainer-approved source/package hashes may be promoted publicly.
Follow [release process](RELEASE_PROCESS.md); do not use hosted Actions to
debug local failures or generate unapproved artifacts.
