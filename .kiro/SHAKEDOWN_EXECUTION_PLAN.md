# Shakedown Execution Plan - Scenes 3-10

**Status**: Ready to execute  
**Backend**: Running on port 59933  
**Theme**: Running on port 8080  
**Discovery**: Running on port 11430  

---

## Execution Strategy

### Current Status
✅ Scene 1: Setup & Baseline - PASS
✅ Scene 2: Model Discovery - PASS (models loading correctly)

### Remaining Scenes (3-10)

#### Scene 3: Model Selection
**Objective**: Verify model selection works and enables chat

**Steps**:
1. Request models (already done)
2. Click on a model card
3. Verify selection message sent through WebSocket
4. Verify chat input becomes enabled
5. Verify model marked as selected in UI

**Success Criteria**:
- ✅ Model selection message sent
- ✅ Chat input enabled
- ✅ Model shows as selected
- ✅ No errors in console

#### Scene 4: Chat Streaming (CRITICAL)
**Objective**: Verify chat messages stream correctly

**Steps**:
1. Send a chat message
2. Watch tokens stream back
3. Verify tokens arrive in order
4. Verify no duplication
5. Verify response displays in chat

**Success Criteria**:
- ✅ Message sent successfully
- ✅ Tokens stream back
- ✅ Tokens in correct order
- ✅ No token duplication
- ✅ Response displays in chat history

#### Scene 5: Tool Execution
**Objective**: Verify tools execute correctly

**Steps**:
1. Send request to use a tool
2. Verify tool executes
3. Verify results display in chat
4. Test multiple tools

**Success Criteria**:
- ✅ Tool executes
- ✅ Results display
- ✅ No errors

#### Scene 6: Metrics Display
**Objective**: Verify metrics display and update

**Steps**:
1. Verify metrics panel visible
2. Check metrics values
3. Send messages and verify metrics update
4. Verify values match backend

**Success Criteria**:
- ✅ Metrics panel visible
- ✅ Values display correctly
- ✅ Real-time updates work
- ✅ Values match backend

#### Scene 7: Error Handling
**Objective**: Verify error handling is graceful

**Steps**:
1. Send empty message
2. Send very long message
3. Send special characters
4. Simulate connection issues

**Success Criteria**:
- ✅ Empty message shows error
- ✅ Long message sends
- ✅ Special characters display
- ✅ Connection recovers

#### Scene 8: Performance & Stability
**Objective**: Verify performance under load

**Steps**:
1. Send 10+ messages in sequence
2. Monitor CPU/memory
3. Verify no slowdown
4. Verify connection stays open

**Success Criteria**:
- ✅ No slowdown
- ✅ CPU < 5% idle
- ✅ Memory stable
- ✅ Connection stays open

#### Scene 9: Visual & UX
**Objective**: Verify visual consistency

**Steps**:
1. Check layout at 1920x1080
2. Check layout at 1366x768
3. Check layout at 1024x768
4. Verify colors consistent
5. Verify animations smooth

**Success Criteria**:
- ✅ Layout stable at all resolutions
- ✅ Colors consistent
- ✅ Animations smooth

#### Scene 10: Security & Wiring
**Objective**: Verify security and architecture

**Steps**:
1. Grep for hardcoded ports
2. Verify discovery service used
3. Verify WebSocket protocol
4. Check backend logs

**Success Criteria**:
- ✅ No hardcoded ports
- ✅ Discovery service used
- ✅ WebSocket protocol correct
- ✅ Backend logs clean

---

## Next Action

Execute Scene 3: Model Selection

**Command**: Click on a model card and verify selection

