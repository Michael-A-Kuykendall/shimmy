# Copilot Autonomy — Single-path Acceptance Criteria

Purpose: Define testable milestones and acceptance criteria so the assistant can propose a single-path plan and execute it without asking many branching questions.

How it works
- The assistant proposes one concrete plan with 2–5 milestones. Each milestone must include clear, automated acceptance criteria (tests, artifacts, or outputs).
- The assistant will not ask for approval for internal milestone steps. After a milestone finishes, the assistant reports results and waits for the user's approval to commit the change or move to the next milestone if the milestone is marked as 'requires user confirmation'.

Acceptance criteria template (per milestone)
- Description: short plain text
- Inputs: files or environment assumptions
- Actions: commands to run (e.g., run_task, cargo test)
- Outputs: tests, verify-report.json, screenshots — must be present
- Success conditions: explicit pass/fail conditions (e.g., `cargo test` passes for module X, verify-report.json validates against script/verify-report.schema.json)

Milestone examples
1) Stabilize validator output
  - Inputs: theme-validator/validator.js, scripts/verify-report.schema.json
  - Actions: run validator against a small discovery JSON fixture
  - Outputs: verify-report.json file & tests to validate schema
  - Success: schema validator passes and unit tests assert compliance

2) Theme spawn semantics
  - Inputs: ThemeManager, FakeSupervisor tests
  - Actions: unit/integration tests run in CI mode
  - Outputs: captured spawn args and supervisor logs
  - Success: tests assert ORCH_THEME_RUNNER handling and runner selection logic

3) CI shakedown job (Playwright + OCR)
  - Inputs: theme-tester, Playwright configs
  - Actions: recipes run on a CI container with browsers installed
  - Outputs: screenshots, verify-report.json, CI artifact upload
  - Success: browser tests pass and artifacts exist

Milestone approval policy
- For safe, reversible changes the assistant may commit after passing the milestones and including tests.
- For destructive or multi-repo operations the assistant must seek explicit user approval before committing.

If you'd like the assistant to be more conservative, add a `--confirm-every-step` flag in the TODO entry and the assistant will ask at every step.

---
This file is intentionally terse. It defines the minimal protocol the assistant will use to stop asking noisy branching questions and instead: plan, execute, produce evidence, then ask only at designated checkpoints.
