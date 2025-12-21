# Shimmy Theme Fix Plan
**Status**: Diagnostic & Recovery Plan  
**Date**: November 26, 2025  
**Goal**: Fix orchestrator port discovery issue and get theme working end-to-end

---

## Phase 1: Diagnostic Verification (CURRENT)

### 1.1 Verify Orchestrator Port Registration
- [ ] Check what port the backend is actually listening on
- [ ] Check what port HTTP discovery is listening on
- [ ] Verify discovery response contains correct WebSocket port
- [ ] Confirm backend WebSocket endpoint is accessible

### 1.2 Verify Theme Configuration
- [ ] Confirm vite.config.ts has port 8080
- [ ] Verify discovery endpoint URL in useDiscovery.ts
- [ ] Check WebSocket connection logic in useWebSocket.ts
- [ ] Verify proxy configuration in vite.config.ts

### 1.3 Verify Backend Registration
- [ ] Check if backend is properly registering with discovery service
- [ ] Verify discovery response format matches theme expectations
- [ ] Confirm port information in discovery response is accurate

---

## Phase 2: Fix Discovery Port Issue

### 2.1 Root Cause Analysis
- [ ] Identify why discovery returns wrong port (55641 vs 57899)
- [ ] Check orchestrator lifecycle for port registration logic
- [ ] Verify discovery service state management

### 2.2 Fix Backend Port Registration
- [ ] Update orchestrator to correctly register backend port with discovery
- [ ] Ensure discovery response includes correct WebSocket port
- [ ] Test discovery endpoint returns accurate information

### 2.3 Verify Fix
- [ ] Restart backend and check discovery response
- [ ] Confirm port matches actual backend port
- [ ] Test theme can connect to WebSocket

---

## Phase 3: Theme Component Fixes

### 3.1 Fix ModelChooser Component
- [ ] Fix WebSocket send timing (wait for OPEN state)
- [ ] Add proper connection state handling
- [ ] Implement retry logic for failed requests
- [ ] Test model list loads and displays

### 3.2 Fix Chat Component
- [ ] Implement message sending through WebSocket
- [ ] Add streaming response handling
- [ ] Display messages in chat history
- [ ] Test end-to-end chat flow

### 3.3 Fix Metrics Component
- [ ] Connect to metrics endpoint
- [ ] Display real-time metrics
- [ ] Update metrics on message send

---

## Phase 4: End-to-End Testing

### 4.1 Shakedown Checklist
- [ ] Backend starts on correct port
- [ ] Discovery service responds with correct port
- [ ] Theme loads on port 8080
- [ ] Discovery hook connects successfully
- [ ] WebSocket connects to correct port
- [ ] Model list loads and displays
- [ ] Can select a model
- [ ] Can send a chat message
- [ ] Response streams back correctly
- [ ] Metrics display and update

### 4.2 Validation
- [ ] All components render without errors
- [ ] No console errors
- [ ] WebSocket stays connected
- [ ] Chat works end-to-end

---

## Current Status (Updated)

### Fixed Issues
1. ✅ **Port Configuration**: vite.config.ts changed from 5173 to 8080
2. ✅ **Discovery Port**: Now correctly on 11430 (after killing stale process)
3. ✅ **Backend Registration**: Backend now registers with discovery on correct port

### Known Issues
1. **WebSocket Connection**: Still not connecting (logs cut off after discovery success)
2. **ModelChooser Error**: "Failed to execute 'send' on 'WebSocket': Still in CONNECTING state"
3. **Port Mismatch**: Discovery returns backend port, but WebSocket may not be connecting

### What's Working
- Backend running on 51505
- HTTP Discovery running on 11430 (correct port)
- Theme loads on 8080
- Discovery hook connects successfully
- Discovery returns correct backend information
- Vite dev server on correct port

### What's Broken
- WebSocket connection (logs cut off, no connection messages)
- Model chooser can't send messages
- Chat functionality blocked

---

## Execution Order

1. **Verify current state** (diagnostic)
2. **Fix discovery port registration** (backend)
3. **Test discovery response** (verification)
4. **Fix theme components** (frontend)
5. **Run end-to-end shakedown** (validation)

