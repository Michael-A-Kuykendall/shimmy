# Theme Shakedown Roadmap
**Status**: Production Theme Validation Plan  
**Approach**: AI-as-User with Tools (Screenshots, Chat, Read-Image)  
**Timeline**: This week (shimmy-default + 32-bit), next week (8 more themes)

---

## Phase 1: Shimmy-Default Theme (This Week)

### What It Is
- Auto-generated from Rust schema
- Should be created automatically from REST endpoints
- Baseline for all other themes

### Shakedown Steps

1. **Schema Validation**
   ```
   [ ] Run theme-validator against shimmy-default
   [ ] Verify all required fields present
   [ ] Fix any schema errors
   [ ] Re-validate until ✅ passes
   ```

2. **Load & Visual Verification**
   ```
   [ ] Start backend (cargo run --bin shimmy --features console)
   [ ] Load theme in browser
   [ ] Take screenshot
   [ ] Read screenshot to verify UI renders
   [ ] Check for layout issues, missing elements
   ```

3. **Discovery Integration**
   ```
   [ ] Chat: "What models are available?"
   [ ] Verify response shows models
   [ ] Take screenshot
   [ ] Verify model list displays correctly
   ```

4. **Model Selection**
   ```
   [ ] Select a model from UI
   [ ] Take screenshot
   [ ] Verify selection is reflected
   [ ] Chat to confirm model is selected
   ```

5. **Chat Streaming**
   ```
   [ ] Send: "Hello, what is 2+2?"
   [ ] Watch tokens stream in real-time
   [ ] Take screenshot of streaming response
   [ ] Verify tokens arrive in order
   [ ] Verify response completes cleanly
   ```

6. **Tool Execution**
   ```
   [ ] Send: "Use the read_image tool"
   [ ] Verify tool executes
   [ ] Take screenshot of tool result
   [ ] Verify tool output displays correctly
   ```

7. **Error Handling**
   ```
   [ ] Send empty message → verify error
   [ ] Select invalid model → verify error
   [ ] Disconnect WebSocket → verify recovery
   [ ] Take screenshots of each error state
   ```

8. **Performance & Stability**
   ```
   [ ] Send 10+ messages in sequence
   [ ] Take screenshot after each
   [ ] Verify no slowdown or memory issues
   [ ] Check metrics endpoint
   [ ] Verify tokens_per_second is populated
   ```

9. **Document Results**
   ```
   [ ] Create shakedown report
   [ ] List what works (✅)
   [ ] List what's broken (❌)
   [ ] List what needs fixing (⚠️)
   [ ] Recommend fixes
   ```

10. **Lock It Down**
    ```
    [ ] Create PPT invariant tests
    [ ] Add regression tests
    [ ] Mark as "production ready"
    [ ] Move to next theme
    ```

### Success Criteria
- ✅ Schema validation passes
- ✅ Theme loads without errors
- ✅ All features work as expected
- ✅ Chat streaming works end-to-end
- ✅ Tools execute correctly
- ✅ Error handling is graceful
- ✅ Performance is acceptable
- ✅ PPT tests written and passing

### Output
- `SHAKEDOWN_REPORT_SHIMMY_DEFAULT.md` - Detailed findings
- `tests/shimmy_default_invariants.rs` - PPT tests
- `tests/shimmy_default_regression.rs` - Regression tests

---

## Phase 2: 32-Bit Theme (This Week)

### What It Is
- Already created, needs modification
- Likely has schema issues
- Needs validation and fixes

### Shakedown Steps

1. **Schema Validation**
   ```
   [ ] Run theme-validator against 32-bit
   [ ] Identify schema errors
   [ ] Document what's wrong
   [ ] Fix each issue
   [ ] Re-validate until ✅ passes
   ```

2. **Compare to Shimmy-Default**
   ```
   [ ] Load both themes side-by-side (screenshots)
   [ ] Identify visual differences
   [ ] Verify differences are intentional
   [ ] Document any issues
   ```

3. **Run Same Shakedown as Shimmy-Default**
   ```
   [ ] Follow steps 2-8 from Phase 1
   [ ] Take screenshots at each step
   [ ] Verify all features work
   [ ] Document any theme-specific issues
   ```

4. **Document Results**
   ```
   [ ] Create shakedown report
   [ ] List fixes applied
   [ ] List what works/broken/needs fix
   ```

5. **Lock It Down**
   ```
   [ ] Create PPT invariant tests
   [ ] Add regression tests
   [ ] Mark as "production ready"
   ```

### Success Criteria
- ✅ Schema validation passes
- ✅ Theme loads without errors
- ✅ All features work as expected
- ✅ Visual differences from shimmy-default are intentional
- ✅ PPT tests written and passing

### Output
- `SHAKEDOWN_REPORT_32BIT.md` - Detailed findings
- `tests/32bit_invariants.rs` - PPT tests
- `tests/32bit_regression.rs` - Regression tests

---

## Phase 3: Additional Themes (Next Week)

### Themes to Create & Shakedown
1. Dark Mode Theme
2. Minimal Theme
3. Developer Theme
4. Accessibility Theme
5. Mobile-Optimized Theme
6. High-Contrast Theme
7. Custom Theme Template
8. Community Theme

### Process for Each Theme
```
For each theme:
  1. Create theme (or use existing)
  2. Validate schema
  3. Fix schema errors
  4. Run shakedown (same as Phase 1)
  5. Document results
  6. Create PPT tests
  7. Mark as production ready
  
  Estimated time: 1-2 hours per theme
  Total: 8-16 hours for all 8 themes
```

### Success Criteria
- ✅ All 10 themes pass schema validation
- ✅ All 10 themes pass shakedown
- ✅ All 10 themes have PPT tests
- ✅ All 10 themes marked "production ready"

---

## Tools I Will Use

### For Interaction
- **Chat**: Send messages to local AI
- **WebSocket**: Direct backend connection
- **HTTP**: Discovery and metrics endpoints

### For Verification
- **Screenshot**: Capture theme state
- **Read-Image**: Analyze screenshots
- **Theme-Validator**: Validate schema
- **Grep/Search**: Find issues in code

### For Debugging
- **Process Control**: Start/stop backend
- **Log Reading**: Check for errors
- **File Operations**: Read theme files

---

## Shakedown Report Template

For each theme, I will create a report like this:

```markdown
# Shakedown Report: [Theme Name]

## Schema Validation
- Status: ✅ Pass / ❌ Fail
- Issues Found: [list]
- Fixes Applied: [list]

## Visual Verification
- Theme loads: ✅ Yes / ❌ No
- Layout correct: ✅ Yes / ❌ No
- Colors correct: ✅ Yes / ❌ No
- Issues: [list]

## Feature Testing
- Discovery: ✅ Works / ❌ Broken
- Model Selection: ✅ Works / ❌ Broken
- Chat Streaming: ✅ Works / ❌ Broken
- Tool Execution: ✅ Works / ❌ Broken
- Error Handling: ✅ Works / ❌ Broken
- Performance: ✅ Good / ❌ Issues

## Issues Found
- ❌ [Issue 1]: [Description] → [Fix]
- ❌ [Issue 2]: [Description] → [Fix]
- ⚠️ [Issue 3]: [Description] → [Recommendation]

## Fixes Applied
- ✅ [Fix 1]: [What was changed]
- ✅ [Fix 2]: [What was changed]

## PPT Tests Created
- `tests/[theme]_invariants.rs` - Invariant tests
- `tests/[theme]_regression.rs` - Regression tests

## Status
- ✅ Production Ready / ⚠️ Needs More Work / ❌ Not Ready

## Next Steps
- [Action 1]
- [Action 2]
```

---

## Timeline

### Week 1 (This Week)
- [ ] Monday: Shimmy-Default shakedown
- [ ] Tuesday: 32-Bit shakedown
- [ ] Wednesday: Create PPT tests for both
- [ ] Thursday: Document and lock down
- [ ] Friday: Prepare for next week

### Week 2 (Next Week)
- [ ] Monday-Friday: Create and shakedown 8 additional themes
- [ ] 1-2 hours per theme
- [ ] Create PPT tests as we go
- [ ] Lock down each theme before moving to next

### Week 3 (Following Week)
- [ ] All 10 themes production ready
- [ ] All PPT tests passing
- [ ] All regression tests in place
- [ ] Ready for MVP launch

---

## Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Shimmy-Default shakedown | ✅ Pass | TBD |
| 32-Bit shakedown | ✅ Pass | TBD |
| Schema validation | 10/10 themes | TBD |
| Shakedown reports | 10/10 themes | TBD |
| PPT tests | 10/10 themes | TBD |
| Regression tests | 10/10 themes | TBD |
| Production ready | 10/10 themes | TBD |

---

## Key Principles

1. **AI as User**: I act like a real user, not a script
2. **Tool-Based**: I use screenshots, chat, read-image to verify
3. **Autonomous**: No human in the loop for iteration
4. **Comprehensive**: Every feature gets tested
5. **Documented**: Every finding gets recorded
6. **Locked Down**: Every theme gets PPT tests

---

## What Success Looks Like

When this is done:
- ✅ 10 themes all production ready
- ✅ All themes validated against schema
- ✅ All themes passed shakedown
- ✅ All themes have PPT invariant tests
- ✅ All themes have regression tests
- ✅ MVP ready to launch
- ✅ Confident in quality

---

## If I Deviate From This Plan

You should call me out if I:
- Create fragile JavaScript test scripts
- Skip screenshot verification
- Assume things work without checking
- Wait for human feedback in the loop
- Don't follow the shakedown checklist
- Don't document findings properly

**This is the protocol. I will follow it exactly.**
