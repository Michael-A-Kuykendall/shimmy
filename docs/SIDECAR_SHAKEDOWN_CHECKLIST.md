# Sidecar Theme Shakedown Checklist

_Intent: deterministic steps the AI follows every cycle before touching code. All boxes must be checked in order; if a step fails, stop and resolve before moving on._

## 0. Context Sync
- [ ] Re-read `MISSION_SIDECAR_SHAKEDOWN_V2.md` to refresh expectations for the 9 validation phases.
- [ ] Skim `FRONTEND_CONTRACT.md` for any schema / event changes since the last run.
- [ ] Confirm the working branch (`feature/discovery-service`) matches the task at hand.

## 1. Stack Preparation
- [ ] Run the `RESET` task (cargo build + backend/theme restart). Wait for the script to finish before touching anything else.
- [ ] Immediately run `CHECK_STACK` to verify shimmy port, discovery snapshot, theme HTML, and console log dump are all reachable.
- [ ] If either task surfaces errors, capture the log, fix the issue, and repeat `RESET` → `CHECK_STACK` until both pass.

## 2. Baseline Diagnostics
- [ ] Execute the deterministic Playwright smoke: `THEME_DIAGNOSTIC_PHASE2` task.
- [ ] If the diagnostic fails, log the phase + error string into the working notes before continuing.
- [ ] Archive the diagnostic output (copy into notes) for later comparison once fixes land.

## 3. Instrumentation Hooks
- [ ] Open the 32bit theme in the browser (http://localhost:8080) and confirm `_console_logs` is being appended in `localStorage`.
- [ ] Collect a fresh console log snapshot (e.g., via DevTools > Application > Local Storage > _console_logs) and paste it into the shakedown notes file.
- [ ] Use the lightweight test endpoints as needed:
	- `GET http://localhost:8080/__shimmy/test/screenshot` → returns `{ screenshot, logs }` without stealing focus.
	- `POST http://localhost:8080/__shimmy/test/select-model {"modelName":"phi3-mini"}` → simulates CONNECT.
	- `POST http://localhost:8080/__shimmy/test/send-chat {"message":"hello"}` → simulates chat send.
- [ ] (Optional fallback) Trigger the legacy theme tester screenshot flow (`node theme-tester/tester.js screenshot`) once per run if the new endpoint times out.

## 4. Phase-by-Phase Validation
Use this template per phase; do not advance until the current phase passes deterministically.

### Phase 1 – Discovery & Metadata
- [ ] Confirm `/api/discovery` response contains complete metadata fields (`size_bytes`, `parameter_count`, `quantization`, `context_length`, `model_type`).
- [ ] Validate ModelChooser renders one card per backend model with no `N/A` placeholders (unless field truly absent upstream).
- [ ] Check console logs for warnings/errors while the chooser mounts; resolve before moving on.

### Phase 2 – Model Selection Flow
- [ ] Click CONNECT for each model exposed; note which model (if any) fails to switch.
- [ ] Verify `useShimmy` emits the "✅ Switched" message and that chat input enables.
- [ ] Capture console errors for failed switches and triage root cause before advancing.

### Phase 3 – Chat & Streaming
- [ ] Send a user message; ensure assistant streaming occurs token-by-token.
- [ ] Confirm timestamps and ordering in `MessageList` are correct.
- [ ] Monitor console/logs for SSE/WS issues; resolve immediately.

### Phase 4 – Metrics Wiring
- [ ] Observe metrics panel updates (CPU, memory, TPS) during and after generation.
- [ ] Check for `NaN` / `undefined` values; confirm `/api/metrics` responses meet contract.

### Phase 5 – Tool Execution (if enabled)
- [ ] Invoke each exposed tool; confirm UI shows request + response.
- [ ] Ensure validator/test harness doesn’t report security issues (path traversal, etc.).

### Phase 6 – Advanced Chat Scenarios
- [ ] Run multi-turn conversation with at least three exchanges.
- [ ] Switch models mid-thread and verify state persists.

### Phase 7 – Network & Security
- [ ] Grep theme repo for forbidden hardcoded ports (`1143`, `11434`, `11435`, etc.).
- [ ] Confirm all network calls route through discovery-client endpoints.

### Phase 8 – UX & Polish
- [ ] Look for layout jumps, missing loaders, or flicker while switching models or sending chat.
- [ ] Ensure offline / error states present actionable messaging.

### Phase 9 – Data Integrity
- [ ] Ensure metadata + chat history survive across reconnects.
- [ ] Validate localStorage/sessionStorage doesn’t accumulate stale state that breaks reloads.

## 5. Reporting & Validator Sync
- [ ] Document each failure + fix attempt directly below the relevant checkbox (KEEP history).
- [ ] When a new failure mode appears, add a corresponding validator assertion so it can’t regress.
- [ ] Re-run `THEME_DIAGNOSTIC_PHASE2` (and broader validators as they land) to prove the fix sticks.

## 6. Handoff Criteria
- [ ] All checkboxes above are marked complete for the current theme run.
- [ ] Validator report shows ✅ for every phase exercised.
- [ ] Notes include links to console logs, screenshots, and diff summaries for each bugfix.

> _Always start the next session at Step 0. If context was lost, re-run RESET + CHECK_STACK before doing anything else._
