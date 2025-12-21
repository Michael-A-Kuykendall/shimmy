# Shakedown Report: shimmy-default

**Date**: November 29, 2025  
**Status**: IN PROGRESS

---

## Scene 1: Setup & Baseline ✅ PASS
- Backend running via orchestrator
- Theme dev server running on port 8080
- Discovery service responding on port 11430
- Browser console clean (no red errors, only React Router warnings)

## Scene 2: Model Discovery ✅ PASS
- Models auto-load when WebSocket connects
- 9 models displayed correctly
- Model metadata complete (name, parameters, quantization, context length, size, type)
- No hardcoded ports in network trace

**Fixes Applied**:
1. ModelChooser useEffect now uses ref to track if models requested
2. useWebSocket sendMessage now uses socketRef to avoid stale closure

## Scene 3: Model Selection ✅ PASS
- Click on model card works
- Selection message sent via WebSocket
- Chat interface appears after selection
- Model name displayed in header

## Scene 4: Chat Streaming ❌ FAIL - BACKEND ISSUES

### Issues Found:

1. **Long response time** (~1.5 minutes to get first response)
   - Backend is slow to start generating

2. **Garbage/looping output** - Model produces repetitive garbage:
   ```
   User request: Hello, how are you? Assistant: Hello, how are you?
   User request: Hello, how are you? Assistant: Hello, how are you?
   (repeats endlessly)
   ```

### Root Cause Analysis:

The backend's `forward_to_shimmy_inference_streaming` function (console/src/websocket/mod.rs:1069) has a flawed prompt construction:

```rust
let system_message = format!("{}\nUser request: {}", tool_primer, message);
```

This creates a prompt like:
```
TOOL USAGE EXAMPLES:
User: "Read the README file"
Assistant: I'll read the README file for you.
<tool_call>read_file(path="README.md")</tool_call>
... (huge tool primer) ...

User request: Hello
```

Problems:
1. **Tool primer in user message** - The tool primer should be in system message, not concatenated with user input
2. **No proper chat template** - Models need ChatML or similar formatting
3. **Model mismatch** - starcoder2 is a CODE model, not a CHAT model - it sees the pattern and just completes it by repeating

### Backend Fix Needed:

The prompt construction in `console/src/websocket/mod.rs` needs to:
1. Put tool primer in system message only
2. Keep user message clean
3. Use proper chat templates for each model type
4. Filter out non-chat models from the model list

## Scene 5: Tool Execution ⏳ NOT STARTED

## Scene 6: Metrics Display ⏳ NOT STARTED
- Metrics panel visible
- Shows "Loading metrics..."

## Scene 7: Error Handling ⏳ NOT STARTED

## Scene 8: Performance & Stability ⏳ NOT STARTED

## Scene 9: Visual & UX ⏳ NOT STARTED

## Scene 10: Security & Wiring ⏳ NOT STARTED

---

## Fixes Applied During Shakedown

### Fix 1: Models not auto-loading
**Root Cause**: useEffect dependency on `[socket, isConnected]` didn't re-run when `isConnected` changed because `socket` reference was the same.

**Solution**: Use a ref (`hasRequestedModels`) to track if models have been requested, only depend on `[isConnected]`.

### Fix 2: sendMessage had stale socket reference
**Root Cause**: `useCallback` with `[socket]` dependency captured stale socket value (null) when called immediately after connection.

**Solution**: Use `socketRef` ref that always holds current socket, use in `sendMessage` callback with no dependencies.

---

## Files Modified

- `theme-generator/templates/react-vite/src/components/ModelChooser.tsx.ejs`
- `theme-generator/templates/react-vite/src/hooks/useWebSocket.ts.ejs`
- `theme-generator/themes/shimmy-default/src/components/ModelChooser.tsx`
- `theme-generator/themes/shimmy-default/src/hooks/useWebSocket.ts`

---

## Screenshots

- `chat-test-before-send.png` - Chat interface ready
- `chat-test-sending.png` - Message being sent
- `chat-test-after-send.png` - "Generating" indicator visible
- `models-socketref-fix.png` - 9 models displayed

---

## Next Steps

1. Wait for chat response to complete
2. Verify response displays correctly
3. Continue with Scene 5 (Tool Execution)
4. Complete remaining scenes
