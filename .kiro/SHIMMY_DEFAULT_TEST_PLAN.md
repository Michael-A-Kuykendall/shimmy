# Shimmy Default Theme - Test & Fix Plan

**Date**: 2025-11-25  
**Objective**: Identify and fix the chat connection issue  
**Approach**: Systematic testing of each component in the stack

---

## PHASE 1: VERIFY ORCHESTRATOR STARTUP

### Test 1.1: Start Orchestrator
```bash
# In VS Code terminal:
cargo build --release --bin shimmy --features llama,console,http-adapter
# Then run:
SHIMMY_ORCH_SKIP_THEMES=1 cargo run --release --bin shimmy -- serve --bind auto
```

**Expected Output**:
```
✅ Shimmy server starting
🟡🟡🟡 [CLI] get_bind_address() called with bind='auto'
🟢🟢🟢 [PORT_MGR] OS allocated ephemeral port: XXXXX
🚀 Starting Shimmy server on 127.0.0.1:XXXXX
📡 HTTP Discovery on 127.0.0.1:11430
✅ Backend registered with HTTP discovery
```

**Verify**:
- [ ] Shimmy starts on ephemeral port (e.g., 62740)
- [ ] HTTP Discovery starts on 11430
- [ ] Backend registers successfully
- [ ] Port file exists: `~/.shimmy/port` contains the ephemeral port

### Test 1.2: Verify Port File
```bash
cat ~/.shimmy/port
# Should output: 62740 (or whatever port was allocated)
```

**Verify**:
- [ ] File exists
- [ ] Contains valid port number
- [ ] Port matches the one in logs

---

## PHASE 2: VERIFY DISCOVERY ENDPOINT

### Test 2.1: Query Discovery HTTP
```bash
# Get the port from the file
PORT=$(cat ~/.shimmy/port)

# Query discovery
curl -s http://127.0.0.1:11430/api/discovery | jq .
```

**Expected Output**:
```json
{
  "discovery_port": 11430,
  "last_updated": "2025-11-25T23:45:44.480975900+00:00",
  "epoch": 1,
  "backends": [
    {
      "id": "shimmy-62740",
      "url": "http://127.0.0.1:62740",
      "port": 62740,
      "models": [
        {
          "name": "phi3-mini",
          "display_name": "Phi 3 Mini",
          ...
        }
      ],
      "capabilities": ["streaming", "websocket"],
      "health": {
        "healthy": true,
        "last_check": "..."
      },
      "started_at": "...",
      "pid": 12345
    }
  ]
}
```

**Verify**:
- [ ] Discovery returns valid JSON
- [ ] `backends` array is not empty
- [ ] Backend `port` matches ephemeral port
- [ ] Backend `url` is correct
- [ ] `capabilities` includes "websocket"
- [ ] Models list is populated

---

## PHASE 3: VERIFY HTTP ENDPOINTS

### Test 3.1: Query Models Endpoint
```bash
PORT=$(cat ~/.shimmy/port)
curl -s http://127.0.0.1:$PORT/api/models | jq .
```

**Expected Output**:
```json
{
  "models": [
    {
      "name": "phi3-mini",
      "display_name": "Phi 3 Mini",
      ...
    }
  ]
}
```

**Verify**:
- [ ] HTTP endpoint responds
- [ ] Models list is populated
- [ ] Each model has required fields

### Test 3.2: Query Metrics Endpoint
```bash
PORT=$(cat ~/.shimmy/port)
curl -s http://127.0.0.1:$PORT/api/metrics | jq .
```

**Expected Output**:
```json
{
  "cpu_percent": 5.2,
  "memory_mb": 512,
  "tokens_per_second": 0,
  ...
}
```

**Verify**:
- [ ] HTTP endpoint responds
- [ ] Metrics are valid numbers
- [ ] No NaN or undefined values

---

## PHASE 4: VERIFY WEBSOCKET ENDPOINT

### Test 4.1: Connect to WebSocket
```bash
# Install wscat if not already installed:
npm install -g wscat

# Get the port
PORT=$(cat ~/.shimmy/port)

# Connect to WebSocket
wscat -c ws://127.0.0.1:$PORT/ws/console
```

**Expected Behavior**:
```
Connected (press CTRL+C to quit)
>
```

**Verify**:
- [ ] WebSocket connection succeeds
- [ ] No connection refused error
- [ ] No timeout error

### Test 4.2: Send get_models Message
```bash
# In the wscat prompt, type:
{"type":"get_models"}
# Then press Enter
```

**Expected Response**:
```json
{
  "type": "models_response",
  "models": [
    {
      "name": "phi3-mini",
      ...
    }
  ]
}
```

**Verify**:
- [ ] Server responds to get_models
- [ ] Response contains models list
- [ ] Response is valid JSON

### Test 4.3: Select Model
```bash
# In the wscat prompt, type:
{"type":"select_model","model_name":"phi3-mini"}
# Then press Enter
```

**Expected Response**:
```json
{
  "type": "model_selected",
  "model_name": "phi3-mini",
  "success": true
}
```

**Verify**:
- [ ] Server responds to select_model
- [ ] Response indicates success

### Test 4.4: Send Chat Message
```bash
# In the wscat prompt, type:
{"type":"chat_request","message":"Hello, how are you?"}
# Then press Enter
```

**Expected Behavior**:
```
# Server sends multiple messages:
{"type":"chat_token","token":"I"}
{"type":"chat_token","token":"'m"}
{"type":"chat_token","token":" "}
{"type":"chat_token","token":"doing"}
...
{"type":"generation_complete"}
```

**Verify**:
- [ ] Server responds with tokens
- [ ] Tokens stream one at a time
- [ ] Generation completes
- [ ] No errors

---

## PHASE 5: VERIFY THEME STARTUP

### Test 5.1: Start Theme
```bash
# In a new terminal:
cd theme-generator/themes/shimmy-default
npm run dev
```

**Expected Output**:
```
VITE v4.5.14  ready in 299 ms
➜  Local:   http://localhost:8080/
```

**Verify**:
- [ ] Theme starts on port 8080
- [ ] No build errors
- [ ] Vite server is ready

---

## PHASE 6: VERIFY THEME DISCOVERY

### Test 6.1: Open Theme in Browser
```bash
# Open browser to:
http://localhost:8080
```

**Expected Behavior**:
- [ ] Page loads
- [ ] Shows "Discovering Shimmy Backend..." message
- [ ] After a few seconds, shows ModelChooser component
- [ ] Models list is populated

**If it fails**:
- [ ] Open browser DevTools (F12)
- [ ] Go to Console tab
- [ ] Look for error messages
- [ ] Check Network tab for failed requests

### Test 6.2: Check Browser Console
```javascript
// In browser DevTools console, type:
console.log('Discovery test')
// Should see output

// Check for discovery errors:
// Look for messages like:
// "🔍 Attempting discovery at: http://127.0.0.1:11430/api/discovery"
// "✅ Discovery successful:"
// or error messages
```

**Verify**:
- [ ] Discovery request is made
- [ ] Discovery response is received
- [ ] No CORS errors
- [ ] No timeout errors

### Test 6.3: Check Network Tab
```
In DevTools Network tab:
1. Look for request to: http://127.0.0.1:11430/api/discovery
   - Status should be 200
   - Response should contain backends array
   
2. Look for WebSocket connection to: ws://127.0.0.1:XXXXX/ws/console
   - Status should be 101 (Switching Protocols)
   - Should show messages being sent/received
```

**Verify**:
- [ ] Discovery HTTP request succeeds (200)
- [ ] WebSocket connection succeeds (101)
- [ ] WebSocket shows message traffic

---

## PHASE 7: VERIFY THEME CHAT

### Test 7.1: Select Model
```
In browser:
1. Wait for ModelChooser to load
2. Click on first model card
3. Should see "Switched to MODEL_NAME" message
4. Should navigate to /chat page
```

**Verify**:
- [ ] Model selection works
- [ ] Chat page loads
- [ ] Chat input is enabled

### Test 7.2: Send Chat Message
```
In browser:
1. Type: "Hello, how are you?"
2. Press Enter or click Send button
3. Should see message appear in chat
4. Should see response streaming in
```

**Verify**:
- [ ] Message appears in chat
- [ ] Response streams back
- [ ] No errors in console
- [ ] Metrics update during generation

---

## TROUBLESHOOTING MATRIX

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| Port file doesn't exist | Shimmy didn't write it | Check server.rs for port file write code |
| Discovery returns empty backends | Backend didn't register | Check server.rs registration code |
| Discovery returns wrong port | Port file is stale | Delete ~/.shimmy/port and restart |
| WebSocket connection refused | WebSocket endpoint not mounted | Check server.rs for /ws/console route |
| WebSocket connection timeout | Firewall or network issue | Check if port is accessible |
| get_models fails | Backend not responding | Check shimmy logs for errors |
| Theme shows "Discovery Failed" | Discovery HTTP failed | Check browser console for CORS errors |
| Theme shows "Disconnected" | WebSocket connection failed | Check browser Network tab for 101 status |
| Chat message doesn't send | WebSocket not connected | Check browser console for connection errors |
| Chat response doesn't stream | Backend not sending tokens | Check shimmy logs for generation errors |

---

## QUICK REFERENCE: COMMANDS

```bash
# Start shimmy
cargo run --release --bin shimmy -- serve --bind auto

# Check port file
cat ~/.shimmy/port

# Query discovery
curl http://127.0.0.1:11430/api/discovery | jq .

# Query models
PORT=$(cat ~/.shimmy/port)
curl http://127.0.0.1:$PORT/api/models | jq .

# Connect to WebSocket
PORT=$(cat ~/.shimmy/port)
wscat -c ws://127.0.0.1:$PORT/ws/console

# Start theme
cd theme-generator/themes/shimmy-default
npm run dev

# Open theme in browser
# http://localhost:8080

# Check logs
tail -f logs/shimmy-*.log
tail -f logs/theme-*.log
```

---

## NEXT STEPS

1. **Run Phase 1-4 tests** to verify backend is working
2. **If all pass**: Run Phase 5-7 tests to verify theme
3. **If any fail**: Use troubleshooting matrix to identify issue
4. **Once identified**: Implement fix and re-test
5. **Once working**: Document the fix and move to next theme

