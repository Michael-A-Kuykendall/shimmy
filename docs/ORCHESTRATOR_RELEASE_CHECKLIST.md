# Orchestrator Release Checklist & Acceptance Criteria

Purpose
-------
This checklist ensures the orchestrator implementation is production-ready and safe to roll out as the default developer and CI orchestration mechanism.

Acceptance criteria
-------------------
- Orchestrator unit + integration tests pass in CI for 10 consecutive runs across the configured matrix.
- The `logs/orchestrator-report.json` artifact is produced by the lifecycle run and validates against the orchestrator JSON schema (existence of keys: status, shimmy_port, discovery_url, theme_url, verification.run/status).
- PRs touching orchestrator code must include isolated tests for new behavior and avoid spawning external processes (use FakeSupervisor / test helpers).
- Coverage: orchestrator package should demonstrate >= 60% coverage across its files (measured on Linux via tarpaulin or cargo-llvm-cov; CI will enforce progress, not immediate hard block at low coverage).
- CLI backward compatibility: the legacy `legacy shell bootstrap (removed; see migration-audit for archived copies)` bootstrap has been removed; `shimmy dev` is the canonical orchestrator.

Release steps
-------------
1. Ensure all orchestrator tests pass on CI and coverage upload succeeds.
2. Verify sample local run using `shimmy dev <theme>` produces `logs/orchestrator-report.json` and that discovery and theme readiness succeed.
3. Run canonical verification: `shimmy dev <theme> --verify` and ensure the orchestrator-native verification pipeline runs and produces a summary (written to `logs/orchestrator-report.json`).
4. Update release notes describing the default orchestrator behavior and how to opt-out of verification (e.g., `--no-build` or `--no-browser`) — `--legacy-reset` is no longer supported.
5. Monitor CI for 2 weeks. Any failures in orchestrator-related tests should disable merge gating and be triaged immediately.

Post-release metrics & checks
----------------------------
- Add a small CI job that runs `shimmy dev <theme> --verify` on a reproducible theme and uploads `logs/orchestrator-report.json` artifact for inspection.
- Add a weekly smoke run in CI that ensures `shimmy dev` still works end-to-end with the current default theme.

Failure rollback
----------------
If new orchestrator changes introduce regressions, gates will be removed and PRs will revert offending commits; fallback procedures should reference archived scripts only for emergency inspection rather than reintroducing legacy flags.
