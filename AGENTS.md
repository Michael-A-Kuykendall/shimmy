# Agent Instructions — Shimmy

## Repository Architecture

```
shimmy-private/              ← THIS REPO — public-facing CLI/server product (private working copy)
public remote: shimmy.git    ← https://github.com/Michael-A-Kuykendall/shimmy.git
airframe = { version = "0.2" }  ← PUBLIC crates.io dep — Airframe is Shimmy's GPU engine library
```

- **Shimmy is the product. Airframe is Shimmy's GPU engine library.**
  All user-facing value ships through Shimmy. Airframe is a Rust library dependency
  of Shimmy and has no binary, CLI, or server. Both are MIT-licensed and public.
- `cargo build` (default features) compiles the full GPU engine — airframe is downloaded from crates.io.

## Repository Push Policy

- Two remotes exist:
  - `origin` → `https://github.com/Michael-A-Kuykendall/shimmy-private.git` (private working copy)
  - `public` → `https://github.com/Michael-A-Kuykendall/shimmy.git` (public GitHub repo users see)
- In the submodule context (`shimmy_integration/` inside airframe workspace): push with `git push private <branch>`.
- In the standalone context (this repo — `/home/michael/repos/airframe-workspace/shimmy` after the 2026-08-11 Windows→Linux migration; the old `C:/Users/micha/repos/shimmy-private` standalone copy was deleted): push with `git push origin <branch>` (private) or `git push public <branch>` (public).
- Do not push unless explicitly requested by the user.
- To publish to the public shimmy repo, push to the `public` remote.

## Test Failures

**Zero tolerance. No exceptions.**

`cargo test` must finish with 0 failures before any task is considered done.
There is no such thing as a "pre-existing" failure. Fix it before moving on.

**ALWAYS use `--release` for `cargo clippy` and `cargo build`.** Debug-profile builds
produce ~18GB of artifacts in `target/debug/` that waste disk space. Never run bare
`cargo clippy` or `cargo build` without `--release`.

## Architecture (v2.0)

- **Engine**: wgpu/WebGPU WGSL pipeline via Airframe (crates.io: `airframe = "0.1"`). Replaces llama.cpp entirely.
- **Server**: OpenAI-compatible (`/v1/chat/completions`, `/v1/completions`), Ollama-compat (`/api/generate`, `/api/tags`), LM Studio discovery.
- **No Python in default path.** Default build is `airframe` + `huggingface` features.
- **WGSL quant coverage**: F32, F16, Q4_0, Q8_0, Q4_K(M/S), Q5_K(M/S), Q6_K.
- **wgpu 2 GB buffer cap**: Known limit for models with tensors >2 GB. Deferred to v2.1.

## Feature Flags

```toml
default = ["airframe", "huggingface"]  # Full GPU build; use --no-default-features --features huggingface for CPU-only
airframe = ["dep:airframe"]            # Airframe native GPU engine (from crates.io)
gpu = ["airframe", "huggingface"]      # GPU-optimized build
full = ["airframe", "huggingface", "mlx"]
fast / coverage = ["huggingface"]      # CI-safe, no GPU hardware required
# Deprecated stubs (llama.cpp removed in v2.0):
llama = []  llama-cuda = []  llama-vulkan = []  llama-opencl = []
```

## Scope Control

- Console (`crates/console/`) is scaffolded but unimplemented. Keep isolated from runtime release changes.
- Vision work is deferred. Keep on dedicated branches.
- Launch scope is architecture/runtime path only.

## Release Process

Load the `deploy` skill before cutting a release.
Releases are coordinated with Airframe via `scripts/deploy.sh` in the
workspace root (see workspace AGENTS.md for the full deploy process).
One command handles version bumps, commits, tags, crates.io publish,
and GitHub Releases for both repos. Never bump versions or tag manually.

## Branch model

- **Single live branch: `main`.** No `master` branch exists on any remote.
- All work merges into `main` locally; push main + tag to `origin` (public) and
  `private` (working copy). No cloud PRs, no cloud merges.

## What NOT To Do

- Do NOT add an `airframe/` submodule inside this repo.
- Do NOT use a path dep for airframe — it is on crates.io as `airframe = { version = "0.1", optional = true }`.
- Do NOT push without explicit user request.
- Do NOT mix vision or console feature work into launch-critical runtime changes.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:970c3bf2 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   bd dolt push
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->

<!-- BEGIN BEADS CODEX SETUP: generated by bd setup codex -->
## Beads Issue Tracker

Use Beads (`bd`) for durable task tracking in repositories that include it. Use the `beads` skill at `.agents/skills/beads/SKILL.md` (project install) or `~/.agents/skills/beads/SKILL.md` (global install) for Beads workflow guidance, then use the `bd` CLI for issue operations.

### Quick Reference

```bash
bd ready                # Find available work
bd show <id>            # View issue details
bd update <id> --claim  # Claim work
bd close <id>           # Complete work
bd prime                # Refresh Beads context
```

### Rules

- Use `bd` for all task tracking; do not create markdown TODO lists.
- Run `bd prime` when Beads context is missing or stale. Codex 0.129.0+ can load Beads context automatically through native hooks; use `/hooks` to inspect or toggle them.
- Keep persistent project memory in Beads via `bd remember`; do not create ad hoc memory files.

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.
<!-- END BEADS CODEX SETUP -->
