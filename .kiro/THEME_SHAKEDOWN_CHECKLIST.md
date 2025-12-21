# Theme Shakedown Checklist

**Purpose**: Manual checklist for shaking down each of the 10 themes  
**Method**: AI acts as a REAL USER using screenshot/click/type tools  
**NO SCRIPTS**: Do NOT create JavaScript test files

---

## Pre-Flight

- [ ] Orchestrator running: `cargo run --bin shimmy -- dev <theme-name> --no-build`
- [ ] Wait for "lifecycle complete" message
- [ ] Discovery responding: `curl.exe http://127.0.0.1:11430/api/discovery`

---

## Scene 1: Theme Loads

- [ ] Take screenshot: `node theme-tester/tester.js screenshot http://localhost:8080 scene1-load.png`
- [ ] Read screenshot to verify theme loaded
- [ ] Check for any error messages visible

---

## Scene 2: Models Display

- [ ] Verify model cards are visible in screenshot
- [ ] Count models displayed (should be 9)
- [ ] Note any missing metadata

---

## Scene 3: Model Selection

- [ ] Click on a model card (prefer small model like tinyllama)
- [ ] Take screenshot after click
- [ ] Verify selection is reflected in UI
- [ ] Verify chat input becomes available

---

## Scene 4: Chat Streaming (CRITICAL)

- [ ] Type a message: "Hello, what is 2+2?"
- [ ] Click send button
- [ ] **WAIT** - this can take several minutes for model to load and respond
- [ ] Take screenshot periodically to check progress
- [ ] Verify response appears in chat
- [ ] Note response quality

---

## Scene 5: Tool Execution

- [ ] Send message asking about tools
- [ ] Wait for response
- [ ] Take screenshot
- [ ] Verify tool information displayed

---

## Scene 6: Metrics Display

- [ ] Look for metrics panel in UI
- [ ] Take screenshot
- [ ] Note what metrics are shown

---

## Scene 7: Error Handling

- [ ] Try sending empty message
- [ ] Take screenshot of error state
- [ ] Verify graceful handling

---

## Scene 8: Performance

- [ ] Send multiple messages
- [ ] Note response times
- [ ] Check for UI slowdown

---

## Scene 9: Visual/UX

- [ ] Check layout consistency
- [ ] Note any visual issues
- [ ] Verify readability

---

## Scene 10: Security/Wiring

- [ ] Grep for hardcoded ports in theme code
- [ ] Verify discovery service used
- [ ] Check backend logs for errors

---

## Post-Flight

- [ ] Update shakedown report
- [ ] Note any issues found
- [ ] Mark theme as PASS/FAIL
- [ ] Move to next theme

---

## Themes to Shakedown

1. [ ] shimmy-default (CURRENT)
2. [ ] 32-bit theme
3. [ ] Theme 3
4. [ ] Theme 4
5. [ ] Theme 5
6. [ ] Theme 6
7. [ ] Theme 7
8. [ ] Theme 8
9. [ ] Theme 9
10. [ ] Theme 10

---

## REMEMBER

- NO JavaScript test scripts
- Use screenshot/click/type tools ONLY
- Act like a REAL USER
- WAIT for responses (even minutes)
- Trust your eyes (screenshots), not scripts
