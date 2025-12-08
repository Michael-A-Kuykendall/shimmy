# ⚠️ CRITICAL SERVER RULE: NEVER cancel background servers with Ctrl+C! Use `&` or separate terminals!
# If you start a server (shimmy serve, python -m http.server, etc.) and then cancel it, IT WON'T RUN ANYMORE.
# Either use trailing `&` for background OR use different terminal tabs. You've done this mistake 12+ times today!

# 📋 CURRENT STATUS - December 8, 2025

## Active Work: Systematic Issue Resolution 🎯

### IMMEDIATE PRIORITY: Fix All Open Issues (12 remaining)
**Complete Workflow for Each Issue:**
1. **READ AND UNDERSTAND FULL ISSUE + ALL CONVERSATIONS** - Every sub-point, every user comment, every technical detail
2. **Create Feature Branch**: `git checkout -b fix/issue-NNN-description`
3. **Implement Solution**: Make minimal, targeted changes based on full understanding
4. **Add Regression Tests**: Create tests to prevent this issue from recurring
5. **Add PPT/Invariant Tests**: If needed for core functionality validation
6. **Test Thoroughly**: Run local tests + regression suite
7. **Pass Release Gates**: `./scripts/dry-run-release.sh` must pass all gates
8. **Create PR**: Push branch, create PR with detailed description
9. **Merge & Monitor**: Merge PR, ensure CI passes, handle any issues
10. **Write Human Response**: Respond to all users in natural human voice (not AI), sign as "Mike"

### Current Open Issues Priority Order:
1. **#146**: Docker image publishing (HIGH - affects container users)
2. **#114**: Distribution pipeline failures (HIGH - affects all installations)
3. **#144**: MLX default on Apple Silicon (MEDIUM - macOS UX)
4. **#143**: uvx support (MEDIUM - Python packaging)
5. **#145**: Multi-modal support (LOW - major feature)
6. **#153**: Swagger docs (LOW - docs)
7. **#137**: Quickstart/demo (LOW - docs)
8. **#135**: README examples (LOW - docs)
9. **#141**: OpenAI response.create (QUESTION)
10. **#150**: Server model loading (QUESTION)
11. **#151**: How shimmy works (QUESTION)

### Development Workflow Rules:
- **NEVER work on main**: Always create feature branches
- **Read FULL context first**: Every issue conversation, every technical detail
- **Test before commit**: `./scripts/dev-test.sh` or `cargo test`
- **Release gates mandatory**: `./scripts/dry-run-release.sh` before PR
- **Clean commits**: `cargo fmt`, `cargo clippy -- -D warnings`
- **Detailed PRs**: Include issue link, reproduction steps, test results
- **Human responses**: Write responses in natural human voice, not AI-speak

### Quality Gates:
- ✅ Code compiles and runs
- ✅ All tests pass (unit + regression)
- ✅ Dry-run release passes all 6 gates
- ✅ No warnings in builds
- ✅ Issue fully resolved per user requirements

### Completed Issues (5 total):
- ✅ #152: Docker build failure
- ✅ #142: AMD GPU detection on Windows
- ✅ #113: OpenAI API frontend compatibility
- ✅ #147: Sharded model file grouping
- ✅ #139: Unicode streaming fixes

### Shelved Issues:
- ❌ #127: MLX smoke test (too complex, awaiting new library)

---
Keep this file concise; update as issues are resolved.

---

# Copilot / AI Assistant Operating Guide for Shimmy

This file teaches any AI assistant how to work effectively inside this repository. Keep replies lean, perform actions directly, and favor incremental verified changes.

## CRITICAL RULES - NEVER VIOLATE

### 1. NEVER Print Fake Validation
**WRONG**: `echo "✅ Build successful"` or `printf "All tests passing"`
**RIGHT**: Actually check: `ls -lh target/release/shimmy.exe && echo $? && ./shimmy --version`

- Never use echo/printf to print success messages
- Always verify with actual commands (ls, grep, test exit codes, run the binary)
- If you can't verify it, say "I cannot verify this yet" - don't fake it

### 2. NEVER Use `!` in Bash Commands
**WRONG**: `echo "Build finished!"` or `rg "println!" src/`
**RIGHT**: `printf "%s\n" "Build finished"` or `rg 'println\!' src/`

- Bash interprets `!` as history expansion (event not found error)
- Use printf instead of echo when printing messages with !
- **ALWAYS escape ! in regex patterns**: Use `'println\!'` not `"println!"`
- This happens constantly - CHECK EVERY COMMAND with ! before running

### 3. ALWAYS Use `&` for Background Processes
**WRONG**: Long-running commands without `&` (blocks terminal)
**RIGHT**: `command args &` (runs in background, keeps terminal available)

- Use `&` for servers, builds, uploads, or any long-running process
- This prevents blocking the terminal and allows continued work
- Essential for workflow efficiency on expensive compute instances

### 4. ZERO TOLERANCE FOR WARNINGS
**RULE**: Fix ALL warnings immediately when encountered - never proceed with warnings present
**ACTION**: Stop and fix each warning properly (understand the issue, implement correct solution)

- Warnings indicate poor software engineering that must be corrected
- No warnings allowed in any build output - achieve completely clean builds
- Fix warnings at their source, only suppress if genuinely unavoidable (like auto-generated code)
- This is non-negotiable - warnings = incomplete work that must be finished

### 3. Python Command is `py` NOT `python3`
**WRONG**: `python3 script.py`
**RIGHT**: `py script.py`

- Windows uses `py` launcher, not `python3`
- Check yourself before running Python commands

### 3. Read Documentation BEFORE Trial-and-Error
**WRONG**: Try random commands, see what works
**RIGHT**: `fetch_webpage` to get official docs, then execute correct command

- Your training data is 2+ years old
- APIs change, flags change, behavior changes
- Read current docs FIRST, especially for: cargo, git, build tools
- One doc lookup saves 10 failed attempts

## Mission
Shimmy is a single-binary local inference shim (GGUF + optional LoRA) exposing simple HTTP/SSE/WebSocket endpoints plus a CLI. Goal: fast, frictionless local LLM token streaming that can front other tools (e.g. punch-discovery, RustChain) and act as a drop‑in development aide.

## Core Components
- `src/engine/llama.rs`: llama.cpp backend via `llama-cpp-2` (feature `llama`).
- `src/api.rs`: `/api/generate` (POST, JSON) with optional SSE streaming and `/ws/generate` WebSocket streaming.
- `src/server.rs`: axum server wiring.
- `src/templates.rs`: prompt template families (ChatML, Llama3, OpenChat).
- `src/model_registry.rs`: simple in-memory registry (now single model).
- `src/cli.rs` + `src/main.rs`: CLI commands (serve, list, probe, bench, generate).

## Build & Run
- Non-backend (stub): `cargo run -- list` (no llama feature).
- Real backend: `cargo run --features llama -- probe phi3-lora`.
- Serve: `cargo run --features llama -- serve --bind 127.0.0.1:11435` (choose free port if conflict).
- Generate (CLI quick test): `cargo run --features llama -- generate --name phi3-lora --prompt "Say hi" --max-tokens 32`.
- HTTP JSON (non-stream): `POST /api/generate {"model":"phi3-lora","prompt":"Say hi","stream":false}`.
- SSE stream: same body with `"stream":true`; tokens arrive as SSE `data:` events, `[DONE]` sentinel.
- WebSocket: connect `/ws/generate`, first text frame = same JSON body, then token frames, final `{ "done": true }`.

Environment variables:
- `SHIMMY_BASE_GGUF` (required path to base model gguf)
- `SHIMMY_LORA_GGUF` (optional adapter)

## Conventions
- Keep public API minimal & stable (avoid breaking request/response shapes without versioning).
- Use owned `String` in token callbacks to avoid borrow lifetime headaches.
- Unsafe in `llama.rs` limited to context lifetime transmute; don’t expand without justification.
- Prefer additive changes; small focused patches.
- After editing Rust code: build (`cargo build --features llama`) to ensure no regressions.

## Adding Features (Playbook)
1. Outline contract (inputs, outputs, error cases) in commit message or PR body.
2. Add types & endpoint skeletons before wiring generation logic.
3. Add minimal tests (if introduced) or a benchmark harness stub.
4. Run build + (future) tests; fix warnings if trivial (e.g., unused_mut).
5. Update README / this file if external behavior changes.

## Error Handling
Return appropriate HTTP codes:
- 404 if model not found.
- 502 for backend load/generation failure.
- Keep body terse JSON when possible, e.g. `{ "error": "load failed" }`.

## Streaming Patterns
- SSE: single generation per HTTP request.
- WebSocket: future multi-ops (cancel, dynamic temperature) — plan to accept control frames (JSON with `{"stop":true}`) later.

## Performance Notes
- Generation latency dominated by model; SSE vs WS difference is small. Use WS for mid-stream control.
- Consider adding: token-per-second metrics, simple `/diag` enrichment, NDJSON alt streaming.

## Safe Refactors Checklist
- [ ] Build passes (`cargo build --features llama`).
- [ ] CLI still lists & probes model.
- [ ] `/api/generate` non-stream path works.
- [ ] SSE streaming path returns tokens + `[DONE]`.
- [ ] WebSocket path token frames + final `{done:true}`.

## Planned Enhancements (Open)
- NDJSON alternative streaming / unified event schema.
- Cancel / abort mid-generation (shared cancellation flag inspected each loop).
- Multi-model registry & dynamic load/unload.
- Metrics: per-request timing, token counts, throughput.
- Simple auth (token header) for remote usage.
- LoRA hot-swap (adapter reload without restart).
- Safer context lifetime (remove unsafe transmute via owned wrapper struct).

## Interaction Rules for AI Assistants
- Do work directly (create/edit files) instead of printing large blobs unless asked.
- After 3–5 file edits, pause and summarize delta.
- Avoid speculative large refactors; confirm intent.
- When blocked by missing info (paths, model file), explicitly request it once.
- Provide minimal command examples (avoid overlong logs) unless debugging.

## punch-discovery Synergy
Use Shimmy as a fast local model for intermediate drafts:
1. Run `punch discover / analyze` to produce structured insights.
2. Compress context (metrics + concise insight bullets) and send to Shimmy for patch drafting.
3. Validate & iterate; escalate only difficult cases to remote larger models.

## Minimal Prompt Template Guidance
- ChatML variant used when registry template = `chatml`.
- Provide `system` if you want role guidance; leave `messages` roles as `user` / `assistant` / `system` aligned with template expectations.

## Quality Gate (Manual Until Tests Added)
- Build success.
- Probe success (model loads quickly, < expected memory footprint for size).
- Sample generation returns text (≥1 token) within configured max_tokens.

## Adding Tests (Future)
Introduce a cargo feature `stub` to force deterministic token output; then assert API contract shapes & streaming sequence.

---
Keep this file concise; prune outdated sections when features land.

## Upstream Contribution Protocol

**CRITICAL**: When contributing to upstream projects (llama-cpp-rs, etc.):
1. **NO AI SHORTCUTS** - Every line must be real, working code
2. **NO STUBBING** - Never use "...existing code..." or placeholder comments
3. **VERIFY EVERYTHING** - Test in shimmy production first
4. **ACCURATE COMMIT MESSAGES** - Describe what code actually does, not what you intended
5. **REVIEW BEFORE PUSH** - User reviews every line before submission
6. **PATIENCE** - Better to take time and get it right than rush and embarrass ourselves
