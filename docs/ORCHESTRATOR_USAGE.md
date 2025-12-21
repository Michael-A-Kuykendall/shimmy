Shimmy Orchestrator — Usage
===========================

Quick examples (developer):

- Start everything using the Rust orchestrator (preferred):

  ```bash
  shimmy dev 32bit-interface --verify
  ```

- Start without rebuilding the shimmy binary (faster when you already built):

  ```bash
  shimmy dev 32bit-interface --no-build
  ```

- Legacy bootstrap: `legacy shell bootstrap (removed; see migration-audit for archived copies)` has been removed. Use `shimmy dev` as the canonical orchestrator. If you need historical context for the legacy script, consult the project's migration notes or `migration-audit` (if present).

Notes:
-- The `--verify` flow will run the orchestrator-native verification pipeline (no external shell scripts) and write `logs/orchestrator-report.json`.
- Console commands (chat/edit/analyze) are gated behind the `console` cargo feature at compile-time and a runtime license gate (set `SHIMMY_LICENSE_KEY` or `SHIMMY_LICENSE_TEST=1` for tests).
