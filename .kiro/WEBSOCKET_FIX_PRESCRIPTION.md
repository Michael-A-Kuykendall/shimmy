# WebSocket Connection Fix Prescription

**Issue**: WebSocket connection closes after first message/response  
**Root Cause**: Client (theme) is closing the connection, not the server  
**Status**: Requires investigation and fix

---

## Problem Analysis

### Observed Behavior
```
[BACKEND] handle_get_models called
[BACKEND] list_models returned 9 models
[BACKEND] Returning models_response, response length: 3646
[BACKEND] Response received, first 100 chars: {"models":[...
[BACKEND] is_direct_response = true
[BACKEND] Sending direct response (not wrapped)
WebSocket close received: Some(CloseFrame { code: 1001, reason: "" })
WebSocket connection handler exiting
```

### Analysis
- Backend successfully processes the request
- Backend successfully sends the response
- **Client closes the connection** (CloseFrame code 1001 = "Going Away")
- Backend then exits the connection handler

### Root Cause
The **theme/client** is closing the WebSocket connection after receiving the response, not the backend.

---

## Fix Prescription

### Phase 1: Verify Backend Behavior (DONE)
✅ Backend WebSocket handler is correct
✅ Backend keeps connection open in loop
✅ Backend sends response and continues waiting for next message
✅ Backend only closes on explicit client close or error

### Phase 2: Fix Theme WebSocket Connection (REQUIRED)

The theme's WebSocket connection is closing after receiving the first response. This could be due to:

1. **Error in message handling** - The theme might be throwing an error when processing the response
2. **Connection timeout** - The theme might have a timeout that's closing the connection
3. **Explicit close** - The theme might be explicitly closing after receiving a response

#### Investigation Steps

1. **Check browser console for errors**
   - Look for JavaScript errors when response is received
   - Check if there's an exception being thrown

2. **Check theme WebSocket event handlers**
   - Verify `onmessage` handler doesn't throw errors
   - Verify `onerror` handler isn't being triggered
   - Verify `onclose` handler isn't being called prematurely

3. **Check for explicit close calls**
   - Search for `socket.close()` in theme code
   - Verify no automatic close after response

#### Likely Culprits

**In `useWebSocket.ts`**:
- The `onmessage` handler might be throwing an error
- The `onerror` handler might be closing the connection
- The `onclose` handler might be closing prematurely

**In component message handlers**:
- The custom event listener might be throwing an error
- The response processing might be failing

### Phase 3: Fix Implementation

#### Step 1: Add Error Logging to useWebSocket
```typescript
ws.onmessage = (event) => {
  try {
    const message = JSON.parse(event.data)
    console.log('📨 Received message:', message)
    
    // Dispatch custom event for message handling
    window.dispatchEvent(
      new CustomEvent('shimmy-websocket-message', { 
        detail: message 
      })
    )
  } catch (error) {
    console.error('❌ Failed to parse WebSocket message:', error)
    console.error('Raw data:', event.data)
  }
}

ws.onerror = (error) => {
  console.error('❌ WebSocket error:', error)
  setError('WebSocket connection error')
}

ws.onclose = (event) => {
  console.log('🔌 WebSocket disconnected:', event.code, event.reason)
  console.log('Close event details:', {
    code: event.code,
    reason: event.reason,
    wasClean: event.wasClean
  })
  // ... rest of onclose handler
}
```

#### Step 2: Add Error Logging to Component Message Handlers
```typescript
// In ModelChooser, Chat, Metrics components
const handleMessage = (event: CustomEvent) => {
  try {
    const message = event.detail
    console.log('📨 Component received message:', message)
    
    switch (message.type) {
      case 'models_response':
        console.log('✅ Models response received:', message)
        // ... handle response
        break
      // ... other cases
    }
  } catch (error) {
    console.error('❌ Error handling message:', error)
  }
}
```

#### Step 3: Verify No Explicit Close
Search for all instances of `socket.close()` in theme code and verify they're only called intentionally.

#### Step 4: Test with Logging
1. Rebuild theme with logging
2. Open browser DevTools
3. Click "Refresh Models"
4. Check console for:
   - Message received log
   - Any errors
   - Close event details

### Phase 4: Root Cause Determination

Once logging is in place, check:

1. **Is the message being received?**
   - If yes: message handling is working
   - If no: network issue or parsing error

2. **Is there an error when processing the message?**
   - If yes: fix the error in the handler
   - If no: continue to next check

3. **Is the close event being triggered?**
   - If yes: check the close code and reason
   - If no: connection is staying open

4. **What is the close code?**
   - 1000: Normal closure (intentional)
   - 1001: Going away (client disconnecting)
   - 1002: Protocol error
   - 1006: Abnormal closure (connection lost)
   - Other: Check WebSocket close codes

---

## Expected Behavior After Fix

1. Theme connects to backend via WebSocket
2. Theme sends `get_models` request
3. Backend receives request and sends response
4. **Theme receives response and processes it**
5. **Connection stays open**
6. Theme can send another request
7. Backend receives and responds
8. Connection stays open indefinitely until:
   - User closes browser
   - User navigates away
   - Explicit close is called
   - Network error occurs

---

## Testing Checklist

- [ ] Add console logging to useWebSocket.ts
- [ ] Add console logging to component message handlers
- [ ] Rebuild theme
- [ ] Open browser DevTools
- [ ] Click "Refresh Models"
- [ ] Check console for:
  - [ ] "📨 Received message" log
  - [ ] "✅ Models response received" log
  - [ ] No error messages
  - [ ] No close event triggered
- [ ] Verify models display in UI
- [ ] Verify connection stays open
- [ ] Send another request to verify connection is persistent

---

## Files to Modify

### Theme Files
- `theme-generator/themes/shimmy-default/src/hooks/useWebSocket.ts` - Add logging
- `theme-generator/themes/shimmy-default/src/components/ModelChooser.tsx` - Add logging
- `theme-generator/themes/shimmy-default/src/components/Chat.tsx` - Add logging
- `theme-generator/themes/shimmy-default/src/components/Metrics.tsx` - Add logging

### Backend Files
- None (backend is working correctly)

---

## Conclusion

The backend WebSocket handler is correctly implemented and keeps the connection open. The issue is that the **client (theme) is closing the connection** after receiving the first response.

The fix requires:
1. Adding comprehensive logging to identify where the close is happening
2. Fixing the code that's causing the close
3. Verifying the connection stays open for multiple messages

Once the theme keeps the connection open, all functionality (models, chat, tools, metrics) will work correctly.

