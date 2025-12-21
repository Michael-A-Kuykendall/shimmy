# Shimmy Theme Shakedown Specification

**Version**: 1.0  
**Date**: 2025-11-26  
**Status**: Active  
**Scope**: Autonomous end-to-end validation of shimmy-default theme using tools and visual inspection

---

## Specification Overview

This specification defines the complete shakedown process for validating shimmy themes. The process is **tool-driven** and **autonomous**, using browser automation, screenshots, and console inspection to diagnose and fix issues without human-in-the-loop copying of logs.

**Key Principle**: If a problem exists, clicking the UI and inspecting the console/network tab will reveal it immediately. The agent uses tools to capture this state and fix issues autonomously.

---

## Phase 1: Stack Initialization

### Requirement 1.1: Backend Startup
**Specification**: The Shimmy backend must start on an ephemeral port and register with HTTP discovery.

**Acceptance Criteria**:
- Backend starts on ephemeral port (e.g., 62740)
- Port file written to `~/.shimmy/port`
- HTTP discovery starts on port 11430
- Backend registers with discovery service
- Backend responds to `/api/models` HTTP request
- Backend responds to `/api/metrics` HTTP request

**Tool Validation**:
- Execute: `cargo run --release --bin shimmy -- serve --bind auto`
- Verify: Port file exists and contains valid port
- Verify: `curl http://127.0.0.1:11430/api/discovery` returns backends array
- Verify: `curl http://127.0.0.1:{port}/api/models` returns models list

**Success Criteria**: All HTTP endpoints respond with valid JSON

---

### Requirement 1.2: WebSocket Endpoint Availability
**Specification**: The WebSocket endpoint must be accessible on the same port as HTTP.

**Acceptance Criteria**:
- WebSocket endpoint exists at `/ws/console`
- Endpoint is on same port as HTTP (ephemeral port)
- WebSocket accepts connections
- WebSocket responds to `get_models` message

**Tool Validation**:
- Execute: `wscat -c ws://127.0.0.1:{port}/ws/console`
- Send: `{"type":"get_models"}`
- Verify: Response contains models array

**Success Criteria**: WebSocket connection succeeds and responds to messages

---

### Requirement 1.3: Theme Startup
**Specification**: The React theme must build and start on port 8080.

**Acceptance Criteria**:
- Theme builds without errors
- Theme starts on port 8080
- Vite dev server is ready
- No build warnings or errors

**Tool Validation**:
- Execute: `cd theme-generator/themes/shimmy-default && npm run dev`
- Verify: Output shows "ready in XXX ms"
- Verify: No error messages in output

**Success Criteria**: Theme is running and accessible at http://localhost:8080

---

## Phase 2: Discovery & Connection

### Requirement 2.1: Theme Discovery
**Specification**: The theme must discover the backend via HTTP discovery endpoint.

**Acceptance Criteria**:
- Theme loads in browser
- Theme queries `/api/discovery` endpoint
- Discovery returns backend information
- Theme extracts backend port from discovery response
- No CORS errors in browser console

**Tool Validation**:
- Open browser to http://localhost:8080
- Take screenshot of initial load
- Inspect browser console for discovery messages
- Inspect Network tab for discovery HTTP request (should be 200)
- Verify response contains backends array with port

**Success Criteria**: Discovery HTTP request succeeds (200 status) and returns valid backend info

---

### Requirement 2.2: WebSocket Connection
**Specification**: The theme must establish WebSocket connection to backend.

**Acceptance Criteria**:
- Theme attempts WebSocket connection to discovered port
- WebSocket connection succeeds (101 status)
- WebSocket connection is established and ready
- No connection refused or timeout errors
- Browser Network tab shows 101 status for ws:// connection

**Tool Validation**:
- Inspect Network tab for ws:// connection
- Verify status is 101 (Switching Protocols)
- Verify connection shows "Connected" state
- Check browser console for connection errors

**Success Criteria**: WebSocket connection shows 101 status and is in Connected state

---

### Requirement 2.3: Models List Display
**Specification**: After successful connection, theme must display available models.

**Acceptance Criteria**:
- ModelChooser component displays
- Models list is populated
- Each model card shows name and description
- No errors in browser console
- Models are clickable

**Tool Validation**:
- Take screenshot of ModelChooser
- Verify models are visible
- Inspect console for any errors
- Verify no "undefined" or "null" values in UI

**Success Criteria**: ModelChooser displays with populated models list

---

## Phase 3: Model Selection & Navigation

### Requirement 3.1: Model Selection
**Specification**: User must be able to select a model and receive confirmation.

**Acceptance Criteria**:
- User clicks on model card
- `select_model` message sent to backend
- Backend responds with `model_selected` message
- Theme receives confirmation
- No errors in console

**Tool Validation**:
- Click on first model card
- Inspect Network tab for WebSocket message
- Verify message type is `select_model`
- Verify response type is `model_selected`
- Check console for any errors

**Success Criteria**: Model selection message sent and response received

---

### Requirement 3.2: Navigation to Chat
**Specification**: After model selection, theme must navigate to chat page.

**Acceptance Criteria**:
- URL changes to `/chat`
- Chat component loads
- Chat input field is visible and enabled
- No navigation errors
- No console errors

**Tool Validation**:
- Verify URL is `/chat`
- Take screenshot of chat page
- Verify input field is present and focused
- Check console for any errors

**Success Criteria**: URL is `/chat` and Chat component is displayed

---

## Phase 4: Chat Functionality

### Requirement 4.1: Send Chat Message
**Specification**: User must be able to send a chat message to the backend.

**Acceptance Criteria**:
- User types message in input field
- User presses Enter or clicks Send
- Message appears in chat history
- `chat_request` message sent to backend
- No errors in console

**Tool Validation**:
- Type test message: "Hello, how are you?"
- Press Enter
- Verify message appears in chat
- Inspect Network tab for WebSocket message
- Verify message type is `chat_request`

**Success Criteria**: Message appears in chat and is sent to backend

---

### Requirement 4.2: Receive Streaming Response
**Specification**: Backend must stream response tokens to theme.

**Acceptance Criteria**:
- Backend sends `chat_token` messages
- Tokens stream one at a time
- Response appears in chat in real-time
- Response is coherent (not gibberish)
- Generation completes with `generation_complete` message
- No errors in console

**Tool Validation**:
- Monitor Network tab for WebSocket messages
- Verify multiple `chat_token` messages received
- Verify response text appears in chat
- Verify response is readable and coherent
- Verify `generation_complete` message received

**Success Criteria**: Response streams and completes successfully

---

## Phase 5: Metrics Validation

### Requirement 5.1: Metrics Display
**Specification**: Metrics panel must display real-time metrics.

**Acceptance Criteria**:
- Metrics panel is visible on chat page
- CPU% displays valid number (0-100)
- Memory% displays valid number (0-100)
- Tokens/sec displays valid number (≥0)
- No NaN or undefined values
- Values are realistic (not hardcoded)

**Tool Validation**:
- Take screenshot of metrics panel
- Verify all metrics are visible
- Verify values are numbers (not NaN/undefined)
- Verify values are in valid ranges

**Success Criteria**: All metrics display valid numbers

---

### Requirement 5.2: Metrics During Generation
**Specification**: Metrics must update in real-time during response generation.

**Acceptance Criteria**:
- Tokens/sec increases during generation
- CPU% increases during generation
- Memory% stable or incre                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    