# Quick Reference - Shimmy Console Shakedown

**Status**: Complete quick reference for execution  
**Date**: November 26, 2025

---

## PORTS AT A GLANCE

```
Discovery Service:     11430 (fixed)
Shimmy Backend:        51505-51999 (dynamic)
Theme Dev Server:      8080-8099 (development)
```

---

## TOOLS AT A GLANCE

**17 Built-In Tools**:
- File ops: read_file, write_file, list_files, search_files
- Command: run_command
- Git: git_status, git_diff, git_commit, git_log
- Analysis: project_analysis, syntax_check, build_project, run_tests
- Docs: explain_command, get_help
- System: system_metrics
- Image: read_image

---

## 10-SCENE PLAYBOOK AT A GLANCE

1. Setup & Baseline (5 min)
2. Model Discovery (10 min)
3. Model Selection (5 min)
4. Chat Streaming (15 min) ← **BLOCKER**
5. Tool Execution (10 min)
6. Metrics Display (10 min)
7. Error Handling (10 min)
8. Performance & Stability (10 min)
9. Visual & UX (5 min)
10. Security & Wiring (5 min)

---

## TOOLS I HAVE

**Playwright**:
- `node theme-tester/tester.js screenshot <url> [filename]`
- `node theme-tester/tester.js click <url> <selector>`
- `node theme-tester/tester.js type-send <url> <input-selector> <send-selector> <text>`

**Backend APIs**:
- `GET /api/models`
- `GET /api/metrics`
- `GET /api/discovery` (port 11430)
- `WS /ws/generate`

**Code Tools**:
- Read/write files
- Grep search
- Process control
- Theme validator

---

## CURRENT STATE

✅ Backend running  
✅ Discovery service running  
✅ Theme loads on 8080  
✅ Chat streaming code fixed  
❌ Chat streaming NOT working end-to-end  
❌ WebSocket connection failing  

---

## IMMEDIATE TASK

Execute shakedown playbook for shimmy-default theme:
1. Start backend
2. Start theme
3. Execute Scene 1-10 in order
4. Find WebSocket connection issue
5. Fix it
6. Iterate until chat works

---

## GUARDRAILS

❌ NO fragile JavaScript tests  
❌ NO DOM parsing with regex  
❌ NO assuming things work  
❌ NO skipping screenshots  
❌ NO waiting for human feedback  

✅ Act as user with tools  
✅ Take screenshots  
✅ Read screenshots  
✅ Find errors autonomously  
✅ Iterate without human intervention  

---

## EXECUTION SHORTHANDS

- **GO** / **PROCEED**: Execute immediately, no acknowledgment
- **STOP**: Stop immediately
- **REPORT**: Report findings only

---

## THEMES TO SHAKEDOWN

1. Shimmy-Default (this week)
2. 32-Bit (this week)
3-10. 8 Additional Themes (next week)

---

## SUCCESS CRITERIA

- ✅ All 10 scenes pass
- ✅ No red errors in console
- ✅ Chat works end-to-end
- ✅ Tools execute correctly
- ✅ Metrics display accurately
- ✅ Performance acceptable
- ✅ PPT tests written and passing

---

## DOCUMENTS TO READ (IN ORDER)

1. CONTEXT_SUMMARY.md (15 min)
2. SHIMMY_CONSOLE_VISION.md (10 min)
3. SHIMMY_THEME_TESTING_PROTOCOL.md (10 min)
4. SHIMMY_SHAKEDOWN_PLAYBOOK.md (15 min)
5. WORKSPACE_LOCKDOWN.md (this session)

---

## REFERENCE DOCUMENTS

- TOOL_SPECIFICATION.md - Complete tool inventory
- ARCHITECTURE_AND_PORTS.md - Port/IPC architecture
- THEME_QA_CHECKLIST.md - 70+ validation checks
- THEME_SHAKEDOWN_ROADMAP.md - Timeline
- AI_AGENT_INSTRUCTIONS.md - My instructions

---

## VISION

Shimmy Console = Copilot for local AI development

- Any AI model (local or remote)
- Pluggable themes (10 launching)
- Pluggable tools (17 built-in + snap-ins)
- Concurrent multi-agent execution
- $10/month unlimited

---

## READY TO EXECUTE

Workspace is LOCKED IN.  
All specifications complete.  
All tools documented.  
All architecture clear.  
All guardrails in place.

**Status**: READY FOR GO
