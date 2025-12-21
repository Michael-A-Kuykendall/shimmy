# Theme Primitive Validation (Manual, Per-Theme)

Use only the primitive Playwright helper (`theme-tester/tester.js`) plus OCR (`analyze-screenshot.js`). No flow-tester, no scripted selectors. Walk each theme logically.

## Tools
- One-shot actions: `node theme-tester/tester.js <action> <url> [selector]`
  - `screenshot <url> <file>`
  - `click <url> '<selector>' <file>` (captures screenshot after click)
  - `type <url> '<selector>' "text" <file>`
  - `type-send <url> '<inputSelector>' '<sendSelector>' "text" <file>`
- OCR: `node analyze-screenshot.js theme-tester/screenshots/<file>`
- All outputs land in `theme-tester/screenshots/` with console logs alongside.

## Per-Theme Checklist
1) **Page load & discovery**
   - Action: `screenshot http://localhost:8080 phase0.png`
   - Verify via OCR/logs: models visible, no console errors, WS connected if shown.
2) **Model selection**
   - Action: `click http://localhost:8080 '<model connect selector>' phase1.png`
   - Verify: navigation/route change to chat or status shows selected; WS still connected.
3) **Chat send**
   - Action: `type-send http://localhost:8080 '<chat input selector>' '<send button selector>' "Hello" phase2.png`
   - Verify: user message rendered; assistant response arrives/streams; no console errors.
4) **Metrics (if present)**
   - Action: `screenshot http://localhost:8080/chat metrics.png`
   - Verify: metrics populated and updating without errors.
5) **Log review**
   - Inspect `flow-*.log` (from tester actions) or console output emitted by tester; check for errors, WS disconnects, failed requests.

## Notes
- Adjust selectors per theme (read OCR/screenshot to choose). Do not embed selectors in scripts.
- If a step fails, retry after adjusting selectors; record the working selector in notes for that theme.
- No npm installs or rebuilds here; this is validation-only against a running theme/backend.
- If backend not running, skip actions and note as blocker.
