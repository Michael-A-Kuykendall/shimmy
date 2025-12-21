# Orchestrator Migration & Reset.sh Deprecation Plan

Status: In-progress
Last updated: 2025-11-22

Summary
-------
This document describes the steps to migrate developer workflows from the legacy `legacy shell bootstrap (removed; see migration-audit for archived copies)` script to the new Rust-based orchestrator (`shimmy dev`). The CLI now runs the orchestrator lifecycle by default; the legacy `legacy shell bootstrap (removed; see migration-audit for archived copies)` script has been removed from the repository. CI and developer workflows should use `shimmy dev` / `shimmy verify` instead.

Rollout goals
-------------
- Ensure `shimmy dev` is feature-parity with `legacy shell bootstrap (removed; see migration-audit for archived copies)` for development and CI flows.
- Ensure determinism and reliability for CI by removing dependency on external binaries in tests.
- Provide a clear deprecation timeline; `legacy shell bootstrap (removed; see migration-audit for archived copies)` will be removed and no longer used as a fallback.

Steps
-----
1. Code & Tests (done)
   - Implement Rust orchestrator lifecycle (done).
   - Add deterministic test harness & FakeSupervisor so tests don't spawn external processes (done).
   - Add and harden tests to validate lifecycle behavior across success/failure paths (done).

2. CLI & Scripts (done)
   - Make `shimmy dev` use orchestrator lifecycle by default (done).
   - `legacy shell bootstrap (removed; see migration-audit for archived copies)` removed from canonical usage; update automation and CI to call `shimmy dev` instead.

3. Docs & Runbooks (in-progress)
   - Update `DEV_COMMAND_GUIDE.md`, `legacy shell bootstrap (removed; see migration-audit for archived copies)` header, and other runbooks to emphasize `shimmy dev` as the recommended workflow.
   - Add migration guidance for developers who rely on `legacy shell bootstrap (removed; see migration-audit for archived copies)` shortcuts.

4. CI & Validation (in-progress)
   - Ensure orchestrator tests run and pass in CI across targeted platforms (Linux + Windows).
   - Add gating rules to CI that require orchestrator tests to be green before merging orchestrator-related changes.

5. Monitoring & Support Window (post-merge)
   - Announce default change and recommended migration to the team via release notes and developer channels.
   - Monitor CI runs for 2-4 weeks. Roll back CI workflow changes if regressions appear; rely on runbooks for rollback guidance rather than reintroducing `--legacy-reset`.

6. Formal Deprecation (TBD)
   - After successful validation period, mark `legacy shell bootstrap (removed; see migration-audit for archived copies)` as deprecated in docs and provide a removal timeline (e.g., 1-2 release cycles).

Acceptance criteria
-------------------
- Orchestrator tests (unit + integration) pass in CI across configured matrix for 10 consecutive builds.
- Orchestrator lifecycle verified in developer smoke tests (documented steps) and produces `logs/orchestrator-report.json` reliably.
 - Developers adopt `shimmy dev` in default workflows. `legacy shell bootstrap (removed; see migration-audit for archived copies)` has been removed and is not a supported fallback.

Notes
-----
- `legacy shell bootstrap (removed; see migration-audit for archived copies)` has been removed from the repository root and must not be used in developer or CI workflows.
- The orchestrator lifecycle is now the canonical, recommended developer and CI flow; do not reintroduce `--legacy-reset` in normal operations.
