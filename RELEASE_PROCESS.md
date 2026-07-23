# Shimmy Release Process

This workspace uses a **coordinated release script** to bump, publish, tag, and
release both Shimmy and Airframe in concert. You do not manually tag or push
either repo.

## The tool

`scripts/release-coordinated.sh` in the `airframe-workspace` handles everything:
bumping Cargo.toml, `cargo publish` (airframe), git commit + tag, `gh release
create`, and workspace allowance management (`.cargo/config.toml` patch,
`Cargo.lock` resolution against crates.io, remote push policy).

**Usage:**
```bash
# Preview
./scripts/release-coordinated.sh --airframe-version 0.2.11 --shimmy-version 2.3.1 --dry-run

# Execute
./scripts/release-coordinated.sh --airframe-version 0.2.11 --shimmy-version 2.3.1
```

## Prerequisites

- `gh` authenticated with `repo` + `workflow` scopes
- `cargo` with crates.io credentials (`~/.cargo/credentials.toml`)
- Both repos as siblings in the workspace (`airframe/`, `shimmy/`)
- `CARGO_REGISTRY_TOKEN` secret set in both GitHub repos

## What the script does

1. Bump airframe `Cargo.toml`, commit, tag, push
2. `cargo publish -p airframe-observe` (tolerates already-published)
3. `cargo publish -p airframe`
4. `gh release create` for airframe
5. Save shimmy `.cargo/config.toml` patch, delete it
6. Bump shimmy's airframe dep, `cargo update -p airframe` (against crates.io)
7. Restore patch, commit `Cargo.toml` + `Cargo.lock`, tag, push
8. `gh release create` for shimmy

## After the script runs

- Confirm crates.io: `cargo search airframe`
- Confirm GitHub Releases on both repos
- Update `CHANGELOG.md` in both repos (not automated yet)

## CI

Shimmy uses a minimal CI (`.github/workflows/ci.yml`):
- **Build + Test** on push/PR to `main` (no GPU needed, uses `--features fast`)
- **Release** on `v*` tag (creates GitHub Release automatically)

See `AGENTS.md` in the workspace root for the full release protocol.