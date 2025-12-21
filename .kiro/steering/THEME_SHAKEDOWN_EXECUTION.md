# Theme Shakedown Execution Protocol
**Status**: AUTHORITATIVE WORKFLOW  
**Date**: November 26, 2025  
**Purpose**: Performant, repeatable theme validation using AI-as-user (Playwright + OCR + Chat)

---

## THE CORE PRINCIPLE

**I am a user with tools**:
- 🖱️ **Click**: Playwright `tester.js click`
- ⌨️ **Type & Send**: Playwright `tester.js type-send`
- 👀 **See**: Playwright `tester.js screenshot` + `read_image` (OCR)
- 📖 **Read**: Source code, console logs, API responses
- ⚡ **Speed**: 2 days (AI-driven) vs 2-3 weeks (human copy-paste loop)

**Goal**: Execute this checklist autonomously, fix non-blocking issues inline, stop on blocking issues.

---

## PHASE 1: MODEL CHOOSER VERIFICATION

### Step 1.1: Screenshot Initial State
```
tester.js screenshot http://localhost:8080 initial-state.png
read_image initial-state.png mode=ocr
```

**Verify**:
- [ ] Page loads without red console errors
- [ ] Model chooser UI visible
- [ ] All models from `/api/models` are displayed as cards

### Step 1.2: Validate Model Card Data
For each model card visible:
- [ ] Model name matches `/api/models` response
- [ ] Memory requirement displayed (e.g., "~4GB")
- [ ] Parameter count displayed (e.g., "7B")
- [ ] Quantization displayed (e.g., "Q4_K_M")
- [ ] All data is accurate and pulled from backend

**Tool**: Compare OCR text from screenshot to `/api/models` API response

### Step 1.3: Verify No Hardcoded Ports
```
grep -r "11435\|11434" theme-generator/themes/shimmy-default/src/
```

**Verify**:
- [ ] Zero hardcoded port references
- [ ] All communication via discovery service (11430)

---

## PHASE 2: MODEL SELECTION & CHAT ACTIVATION

### Step 2.1: Select a Model
```
tester.js click http://localhost:8080 "[model-card-selector]"
```

**Verify**:
- [ ] Click succeeds
- [ ] Selection confirmation message appears
- [ ] Chat input field becomes enabled

### Step 2.2: Screenshot Chat State
```
tester.js screenshot http://localhost:8080 chat-ready.png
read_image chat-ready.png mode=ocr
```

**Verify**:
- [ ] Chat input visible and enabled
- [ ] No red console errors
- [ ] Model name shown as active

---

## PHASE 3: CHAT FUNCTIONALITY

### Step 3.1: Rust Programming Question
```
tester.js type-send http://localhost:8080 "[input-selector]" "[send-selector]" "Explain how Rust's ownership system works"
```

**Wait for response**, then screenshot:
```
tester.js screenshot http://localhost:8080 rust-response.png
read_image rust-response.png mode=ocr
```

**Verify**:
- [ ] Response appears in chat
- [ ] Response is coherent and about Rust
- [ ] No errors in console

### Step 3.2: File Access Question
```
tester.js type-send http://localhost:8080 "[input-selector]" "[send-selector]" "What files and folders do you have access to in this project?"
```

**Wait for response**, then screenshot and OCR:

**Verify**:
- [ ] Response indicates full file system access
- [ ] Response mentions project structure
- [ ] No errors

### Step 3.3: Second Programming Question
```
tester.js type-send http://localhost:8080 "[input-selector]" "[send-selector]" "How would you refactor this code for better performance?"
```

**Verify**:
- [ ] Response is coherent
- [ ] Shows understanding of code optimization

---

## PHASE 4: TOOL EXECUTION TESTING

For each tool in the registry, execute non-destructively:

### Tool 1: `get_cwd`
```
tester.js type-send http://localhost:8080 "[input-selector]" "[send-selector]" "What is the current working directory?"
```

**Verify**:
- [ ] Tool executes
- [ ] Returns valid path
- [ ] No errors

### Tool 2: `sys_info`
```
tester.js type-send http://localhost:8080 "[input-selector]" "[send-selector]" "Tell me about this system"
```

**Verify**:
- [ ] Tool executes
- [ ] Returns OS, architecture, hostname
- [ ] Data accurate

### Tool 3: `memory_info`
```
tester.js type-send http://localhost:8080 "[input-selector]" "[send-selector]" "How much memory is available?"
```

**Verify**:
- [ ] Tool executes
- [ ] Returns memory stats
- [ ] Values reasonable

### Tool 4-15: Remaining Tools
Execute each tool with non-destructive queries. Document any failures.

---

## PHASE 5: METRICS VALIDATION

### Step 5.1: Get Backend Metrics
```
curl http://127.0.0.1:11430/api/metrics | jq .
```

**Verify**:
- [ ] All 23 metrics present
- [ ] Values are reasonable

### Step 5.2: Screenshot Metrics Display
```
tester.js screenshot http://localhost:8080 metrics.png
read_image metrics.png mode=ocr
```

**Verify**:
- [ ] Metrics panel visible
- [ ] CPU, memory, TPS displayed
- [ ] Values match backend (within tolerance)

---

## ISSUE HANDLING

### Non-Blocking Issues
**Action**: Document in markdown, continue testing
```
## Issues Found (Non-Blocking)
- [ ] Issue 1: [description]
- [ ] Issue 2: [description]
```

### Blocking Issues
**Action**: STOP, fix code, rebuild, re-test
1. Identify root cause in source code
2. Fix in source (not theme)
3. Rebuild backend
4. Re-run from Phase 1

---

## OUTPUT

After completing all phases:

```markdown
# Shakedown Report: [theme-name]

## Status: ✅ PRODUCTION READY / ⚠️ NEEDS FIXES / ❌ BLOCKED

### Phase 1: Model Chooser
- ✅ All models displayed
- ✅ Data accurate
- ✅ No hardcoded ports

### Phase 2: Model Selection
- ✅ Selection works
- ✅ Chat input enabled

### Phase 3: Chat
- ✅ Rust question answered
- ✅ File access confirmed
- ✅ Programming question answered

### Phase 4: Tools
- ✅ get_cwd works
- ✅ sys_info works
- ✅ memory_info works
- [list all 15 tools]

### Phase 5: Metrics
- ✅ All 23 metrics present
- ✅ Display accurate

### Issues Found
[list any non-blocking issues]

### Fixes Applied
[list any fixes made]

### Next Steps
[recommendations]
```

---

## EXECUTION RULES

1. **Act as a user**: Use Playwright like a mouse, OCR like eyes
2. **Be autonomous**: Fix non-blocking issues, stop on blocking issues
3. **Be fast**: 2 days per theme, not 2-3 weeks
4. **Be thorough**: Test all 15 tools, all 23 metrics
5. **Document everything**: Issues, fixes, results
6. **Fix at source**: Problems in theme = fix in code, not theme
7. **WAIT FOR ORCHESTRATOR**: When running `shimmy dev <theme>`, WAIT UNTIL IT'S DONE. No time estimates. No checking. No polling. Just wait and read the terminal output when it completes. Whether it takes 30 seconds or 30 minutes - you wait.

---

## TOOLS AVAILABLE

- `tester.js screenshot` - Take screenshot
- `read_image` - OCR screenshot
- `tester.js click` - Click element
- `tester.js type-send` - Type and send
- `curl` - Query APIs
- `grep` - Search code
- `readFile` - Read source code

---

## SUCCESS CRITERIA

Theme is **production ready** when:
- ✅ Model chooser displays all models correctly
- ✅ Model selection works and enables chat
- ✅ Chat responds to programming questions
- ✅ File access confirmed
- ✅ All 15 tools execute non-destructively
- ✅ All 23 metrics display accurately
- ✅ No red console errors
- ✅ No hardcoded ports

