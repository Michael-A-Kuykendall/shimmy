# Shimmy Theme Testing Protocol
**Status**: CRITICAL ARCHITECTURAL GUIDANCE  
**Audience**: AI Agent (me) - This is how I test themes  
**Last Updated**: November 26, 2025

---

## THE CORE PRINCIPLE

**"Shakedown" = AI Agent Acts as User**

NOT: Fragile JavaScript test scripts that parse output  
NOT: Automated assertions on DOM elements  
NOT: Scripted test sequences

YES: I (the AI agent) use the actual tools available to interact with the theme like a real user would, take screenshots, read them, verify behavior, and iterate.

---

## What "Shakedown a Theme" Means

### Definition
A **shakedown** is a comprehensive manual validation where I:

1. **Act as a real user** - Use the theme's UI like a human would
2. **Use available tools** - Chat with local AI, take screenshots, read images
3. **Verify each feature** - Check that features work as designed
4. **Find errors autonomously** - Use my tools to investigate issues
5. **Iterate and fix** - Don't wait for human feedback, act on what I find
6. **Document results** - Record what works, what doesn't, what needs fixing

### Why This Approach

- ✅ **Faster**: I can test 10x faster than human copy-paste iteration
- ✅ **Comprehensive**: I see what you see (via screenshots) and can act (via tools)
- ✅ **Autonomous**: No human in the loop waiting for feedback
- ✅ **Real**: Tests actual user workflows, not fragile scripts
- ✅ **Scalable**: Same process works for all 10 themes

---

## The Shakedown Process (For Each Theme)

### Phase 1: Setup & Validation
```
1. Verify theme loads (screenshot)
2. Verify schema validation passes (theme-validator tool)
3. Verify all required fields present
4. Document baseline state
```

### Phase 2: Feature Testing
```
For each feature in the theme:
  1. Use the feature (click, type, interact)
  2. Take screenshot to verify state
  3. Read screenshot to confirm visual state
  4. Chat with AI to test backend integration
  5. Verify tool execution if applicable
  6. Document result (✅ works / ❌ broken / ⚠️ needs fix)
```

### Phase 3: Tool Integration Testing
```
For each tool the theme exposes:
  1. Trigger the tool from chat
  2. Verify tool executes correctly
  3. Verify results display in theme
  4. Take screenshot to confirm
  5. Document result
```

### Phase 4: Error Handling
```
For each error scenario:
  1. Trigger the error (invalid input, network issue, etc.)
  2. Verify error message displays
  3. Verify theme doesn't crash
  4. Verify recovery is possible
  5. Document result
```

### Phase 5: Performance & Stability
```
1. Run multiple chat messages (10+)
2. Verify no memory leaks or slowdown
3. Verify WebSocket stays connected
4. Verify metrics update correctly
5. Document results
```

### Phase 6: Summary Report
```
Document:
- ✅ What works
- ❌ What's broken
- ⚠️ What needs fixing
- 📋 Recommended fixes
- 🎯 Next steps
```

---

## Tools I Have Available

### For Interaction
- **Chat with AI**: Send messages to local AI, get responses
- **WebSocket**: Direct connection to backend for streaming
- **HTTP**: Discovery and metrics endpoints

### For Verification
- **Screenshot**: Capture current theme state
- **Read Image**: Analyze screenshots to verify visual state
- **Grep/Search**: Find issues in code or logs
- **File Operations**: Read theme files, check configuration

### For Debugging
- **Process Control**: Start/stop backend
- **Log Reading**: Check backend logs for errors
- **Diagnostics**: Run theme validator, check schema compliance

---

## Shakedown Checklist Template

For each theme, I will create a checklist like this:

```markdown
# Shimmy-Default Theme Shakedown

## Setup & Validation
- [ ] Theme loads without errors
- [ ] Schema validation passes
- [ ] All required fields present
- [ ] Discovery works
- [ ] WebSocket connects

## Feature Testing
- [ ] Model selection works
- [ ] Chat input accepts text
- [ ] Messages stream correctly
- [ ] Tokens arrive in order
- [ ] Generation completes cleanly

## Tool Integration
- [ ] read_image tool works
- [ ] Tool results display
- [ ] Tool errors handled gracefully
- [ ] Multiple tools can execute

## Error Handling
- [ ] Invalid model → error message
- [ ] Empty prompt → error message
- [ ] Connection drop → graceful recovery
- [ ] Invalid tool call → error message

## Performance
- [ ] 10+ messages without slowdown
- [ ] WebSocket stays connected
- [ ] Metrics update correctly
- [ ] No memory leaks

## Summary
- ✅ Works: [list]
- ❌ Broken: [list]
- ⚠️ Needs Fix: [list]
```

---

## Theme Validation Workflow

### Step 1: Validate Against Schema
```bash
# Use theme-validator tool
theme-validator validate shimmy-default
# Output: ✅ Valid or ❌ Invalid (with specific errors)
```

### Step 2: Fix Schema Issues
```
If validation fails:
1. Read error message
2. Identify missing/incorrect fields
3. Update theme files
4. Re-validate
5. Repeat until ✅ Valid
```

### Step 3: Shakedown the Theme
```
Once schema is valid:
1. Start backend
2. Load theme
3. Follow shakedown checklist
4. Document results
5. Fix any issues found
6. Re-test fixed features
```

### Step 4: Lock It Down
```
Once shakedown passes:
1. Create PPT invariant tests for this theme
2. Add regression tests
3. Mark as "production ready"
4. Move to next theme
```

---

## The Themes to Shakedown (In Order)

### Priority 1: Shimmy-Default (Auto-Generated)
- Status: Should be auto-generated from Rust schema
- Action: Validate schema, shakedown, lock down
- Timeline: This week

### Priority 2: 32-Bit Theme (Existing)
- Status: Already created, needs modification
- Action: Validate schema, identify issues, fix, shakedown
- Timeline: This week

### Priority 3-10: Additional Themes (To Create)
- Status: Not started
- Action: Create, validate, shakedown, lock down
- Timeline: Next week (1-2 hours per theme)

---

## What I Will NOT Do

❌ Create fragile JavaScript test scripts  
❌ Parse HTML/DOM with regex  
❌ Assume tests pass without verification  
❌ Skip manual verification steps  
❌ Wait for human feedback in the loop  
❌ Create brittle assertions on UI elements  
❌ Test without seeing actual screenshots  

---

## What I WILL Do

✅ Act as a real user interacting with the theme  
✅ Take screenshots to verify visual state  
✅ Read screenshots to confirm what I see  
✅ Chat with AI to test backend integration  
✅ Use tools to verify functionality  
✅ Find and document errors autonomously  
✅ Iterate and fix issues without human intervention  
✅ Create comprehensive shakedown reports  
✅ Lock down working themes with PPT tests  

---

## Memory & Context Management

I will maintain:

1. **This file** - The protocol (you're reading it)
2. **SHIMMY_CONSOLE_VISION.md** - Architecture principles
3. **Per-theme shakedown reports** - Results and findings
4. **PPT test files** - Locked-down invariants for each theme
5. **Theme validator output** - Schema compliance records

Before each shakedown, I will:
- [ ] Re-read this protocol
- [ ] Review SHIMMY_CONSOLE_VISION.md
- [ ] Check previous theme results
- [ ] Prepare shakedown checklist
- [ ] Verify tools are available

---

## Success Criteria

A theme is "production ready" when:

1. ✅ Schema validation passes
2. ✅ Shakedown checklist all green
3. ✅ No errors found during testing
4. ✅ All tools work correctly
5. ✅ Performance is acceptable
6. ✅ PPT invariant tests written and passing
7. ✅ Regression tests in place

---

## Example: How I'll Shakedown Shimmy-Default

```
1. Validate schema
   → theme-validator validate shimmy-default
   → If ❌, fix and re-validate

2. Start backend
   → cargo run --bin shimmy --features console

3. Load theme
   → Take screenshot
   → Verify UI loads

4. Test discovery
   → Chat: "What models are available?"
   → Verify response
   → Take screenshot

5. Test model selection
   → Select a model from UI
   → Take screenshot
   → Verify selection reflected

6. Test chat streaming
   → Send: "Hello, what is 2+2?"
   → Watch tokens stream
   → Take screenshot of response
   → Verify tokens arrived in order

7. Test tool execution
   → Send: "Use the read_image tool"
   → Verify tool executes
   → Take screenshot of result

8. Test error handling
   → Send empty message
   → Verify error message
   → Take screenshot

9. Test performance
   → Send 10+ messages
   → Verify no slowdown
   → Take screenshot of metrics

10. Document results
    → Create shakedown report
    → List what works/broken/needs fix
    → Recommend next steps
```

---

## This Is Non-Negotiable

This protocol is the **source of truth** for how I test themes. If I deviate from it:

- I'm creating fragile tests (bad)
- I'm not acting as a real user (bad)
- I'm not using my tools effectively (bad)
- I'm wasting your time (bad)

**You should call me out immediately if I:**
- Create JavaScript test scripts
- Try to parse DOM with regex
- Skip screenshot verification
- Assume things work without checking
- Wait for human feedback in the loop

---

## Questions I Will Ask Myself Before Each Shakedown

1. Am I acting as a real user would?
2. Can I see what the user sees (via screenshots)?
3. Can I act as the user would (via tools)?
4. Am I finding errors autonomously?
5. Am I iterating without human intervention?
6. Am I documenting everything?
7. Am I following the checklist?
8. Am I using the right tools?

If the answer to any is "no", I stop and re-read this protocol.

---

## Final Note

This is how we avoid the "nightmare" you had with the previous AI. Clear protocol, autonomous execution, tool-based verification, screenshot-based confirmation, and no human in the loop for iteration.

**I understand. I will follow this protocol exactly.**
