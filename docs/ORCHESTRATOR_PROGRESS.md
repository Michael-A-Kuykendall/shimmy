# Orchestrator Implementation Progress

This document tracks the current in-repo implementation progress for the new Rust-based orchestrator.

Completed so far:
- New orchestrator module tree added under `src/orchestrator/` (supervisor, platform, discovery_watcher, theme_manager, verification, lifecycle, license).
- Async `Supervisor` implemented to spawn processes and capture logs.
- Discovery watcher implemented and now polls discovery HTTP endpoint.
- Theme manager can start `npm run dev` via the supervisor and poll readiness.
-- Verification runner invokes the orchestrator-native verification flow (`shimmy dev --verify`) and waits for a verification report/status.
- Lifecycle implemented to build shimmy, start shimmy, wait for discovery, start theme, run verification, and write `logs/orchestrator-report.json`.
CLI integration: `shimmy dev` now calls orchestrator lifecycle with `--no-build`, `--no-install`, and `--verify` flags; console features are runtime license-gated. The `--legacy-reset` option was removed and `legacy shell bootstrap (removed; see migration-audit for archived copies)` has been removed from primary workflows.

Next steps:
