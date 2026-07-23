# Deploy / Release Skill

Load this skill before cutting a release of this crate.

## The tool

`scripts/release-coordinated.sh` in the parent `airframe-workspace` handles the
full airframe + shimmy coordinated release in one command. Do NOT manually
bump versions, tag, or publish — use the script.

## Usage

```bash
# From the workspace root:
echo y | ./scripts/release-coordinated.sh --airframe-version <ver> --shimmy-version <ver>
```

## What it does

1. Bumps both `Cargo.toml` files (version + dep)
2. Commits + tags + pushes
3. `cargo publish` to crates.io (both crates)
4. `gh release create` on GitHub (both repos)
5. Auto-generates CHANGELOG entries from release notes

## Prerequisites

- `gh` authenticated with `repo` + `workflow` scopes
- `cargo` with crates.io credentials (`~/.cargo/credentials.toml`)
- `CARGO_REGISTRY_TOKEN` secrets set in both GitHub repos
- Both repos clean (`git status --porcelain` empty)

## Idempotent

The script checks what's already done (tags, crates.io versions, GH releases)
and skips completed steps. Safe to re-run if anything fails mid-flight.
