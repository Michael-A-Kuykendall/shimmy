# COPILOT INSTRUCTIONS - SHIMMY (STRICT ONE-PAGE VERSION)

**THIS FILE IS LAW. NO EXCEPTIONS. NO CREATIVITY. FOLLOW EXACTLY.**

---

## 🚨 ABSOLUTE RULE #1: PROCESS CONTROL

**EVERY BUILD/KILL/START/TEST = run_task ONLY.**

- ❌ NO `cargo build` in terminal
- ❌ NO `npm run dev` in terminal  
- ❌ NO custom shell scripts
- ❌ NO timing hacks ("sleep 3")

**✅ ALWAYS use run_task:**
```
COMPILE → cargo build --release --bin shimmy --features llama,console,http-adapter
KILL → taskkill //F //IM shimmy.exe && taskkill //F //IM node.exe && sleep 3
START_SHIMMY → ./shimmy.exe serve --bind auto (background)
WAIT_SHIMMY → poll http://127.0.0.1:11430/api/discovery for 30s
START_THEME → npm run dev in 32bit-interface (background)
RUN_ALL → KILL → START_SHIMMY → WAIT_SHIMMY → START_THEME (sequence)
```

**DECISION TREE (ONLY OPTIONS):**
```
User asks to "build":         → run_task(COMPILE)
User asks to "start":         → run_task(RUN_ALL)
User asks to "restart":       → run_task(RUN_ALL)
User asks to "stop":          → run_task(KILL)
User asks to "test":          → run_task(RUN_ALL) then verify
Need to change code:          → edit file → run_task(COMPILE) → run_task(RUN_ALL)
System seems broken:          → run_task(KILL) → run_task(RUN_ALL)
Anything else in doubt:       → Prefer a single-path plan and proceed with reasonable defaults; ASK USER only if a critical blocker (destructive operation, missing credential, or policy decision) prevents safe progress.
```

---

## 🚨 ABSOLUTE RULE #2: CONTEXT MANAGEMENT

**BEFORE EVERY ACTION:**
1. Read todo list (manage_todo_list with read operation)
2. Check if task is "in-progress" - continue it, don't restart
3. If starting new work: mark as in-progress FIRST
4. After completion: mark completed IMMEDIATELY

**BANNED BEHAVIORS:**
- ❌ Trial and error
- ❌ "Let me try X and see what happens"
- ❌ Freestyle debugging
- ❌ Hoping it works

**REQUIRED BEHAVIORS:**
- ✅ Understand existing state before changing anything
- ✅ Read copilot-instructions FIRST if confused
- ✅ Ask user if unsure about intent
- ✅ Document what you changed and why

---

## 🚨 ABSOLUTE RULE #3: CURRENT STATE (Fresh Session Memory)

**ESSENTIAL FACTS (read ESSENTIAL_STATE.md if it exists):**

**What's built:**
- Shimmy binary: `target/release/shimmy.exe` (9.8MB)
- HTTP discovery service on port 11430 (spawning works now)
- Theme dev server infrastructure ready
- All Rust code compiles, no errors

**What's working:**
- ✅ IPC discovery (backends register via Windows Named Pipes)
- ✅ HTTP discovery endpoint (`/api/discovery` returns backend list)
- ✅ Shimmy starts with NO model loaded (user picks model from UI)
- ✅ Binary spawns correctly with --bind auto

**What's broken/incomplete:**
- ❌ Theme npm dev server may hang (UNKNOWN CAUSE - restart with RUN_ALL)
- ⏳ Theme not yet tested with shimmy discovery

**Current branch:** `feature/discovery-service`

---

## 🚨 ABSOLUTE RULE #4: WHEN STUCK

**IF SYSTEM UNRESPONSIVE:**
```
1. run_task(KILL)                 # Force kill everything
2. run_task(RUN_ALL)              # Full restart sequence
3. If STILL broken → ASK USER
```

**IF CODE WON'T COMPILE:**
```
1. Read error message EXACTLY
2. Search workspace for offending function/type
3. Fix one thing at a time
4. run_task(COMPILE) to verify
5. If can't fix → ASK USER
```

**IF NEED NEW INFO:**
```
Before asking anything additional, the assistant should:
1. Attempt a single-path plan when safe and well-scoped.
2. Run any non-destructive, verifiable checks or tests that reduce ambiguity.
3. If those checks reveal a blocking condition, ask exactly one clear question focused on the blocker (e.g., "Do you want me to proceed with destructive DB migration X? [yes/no]").

Ask the user only when the condition is unsafe to infer.
```

---

## 🚨 ABSOLUTE RULE #5: COMMUNICATION

**Format:**
- Status (1 sentence)
- What Changed (bullets only)
- Next Steps (2-3 bullets)

**NO:**
- Verbose explanations
- Markdown theater
- Performance theater
- "Let me try X..." (you already know you can't)

**YES:**
- Direct facts
- What actually happened
- What's next

---

## TASKS AVAILABLE

All defined in `.vscode/tasks.json`. These are the ONLY valid options:
- `COMPILE`
- `KILL`  
- `START_SHIMMY`
- `WAIT_SHIMMY`
- `START_THEME`
- `RUN_ALL`

Use `run_task(id, workspaceFolder)` with exact task ID.

---

## MODEL FILES

Located in `models/` directory:
- `phi3-mini.gguf` - 0 bytes (⚠️ DON'T USE - empty file)
- `Phi-3-mini-4k-instruct-q4.gguf` - 2.3GB (working)
- `gpt-oss-20b-f16.gguf` - 13GB (working)
- `phi-3.5-moe-f16.gguf` - 79GB (working)

**Shimmy starts with NO model. User picks from UI.**

---

## FRESH SESSION CHECKLIST

When starting new session:
- [ ] Read THIS FILE first (2 minutes)
- [ ] Read ESSENTIAL_STATE.md if exists (1 minute)
- [ ] Check todo list (30 seconds)
- [ ] Continue from where user left off
- [ ] If unsure about state → run_task(KILL) then run_task(RUN_ALL)

---

**END OF INSTRUCTIONS. EVERY RULE IS NON-NEGOTIABLE.**
