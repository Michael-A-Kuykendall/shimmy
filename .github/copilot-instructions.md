# ⚠️ CRITICAL SERVER RULE: NEVER cancel background servers with Ctrl+C! Use `&` or separate terminals!
# If you start a server (shimmy serve, python -m http.server, etc.) and then cancel it, IT WON'T RUN ANYMORE.
# Either use trailing `&` for background OR use different terminal tabs. You've done this mistake 12+ times today!

# 📋 CURRENT STATUS - Oct 8, 2025

## Active Work: MoE Technical Validation Report 🎯

### CRITICAL DISCOVERY - Oct 8, 2025
**llama.cpp already had MoE offloading BEFORE our work**:
- **Upstream**: PR #15077 merged August 4, 2025 (by @slaren)
- **Our work started**: October 4, 2025 (2 months AFTER)
- **What we actually built**: Rust bindings for existing llama.cpp functionality
- **NOT novel**: The core MoE offloading algorithm was already in llama.cpp

### MISSION PIVOT: Technical Validation Report (Not Research Paper)
- **Status**: CORRECTING overclaims, creating honest technical validation
- **Goal**: Produce accurate user documentation with real baselines
- **Current Phase**: Running controlled A/B baselines → Final report

### What We Actually Built ✅
- **Rust Bindings**: `with_cpu_moe_all()`, `with_n_cpu_moe(n)` methods in llama-cpp-2
- **Shimmy Integration**: `--cpu-moe` and `--n-cpu-moe` CLI flags
- **Multi-Model Validation**: 3 models tested (GPT-OSS 20B with controlled baseline, Phi-3.5-MoE 42B, DeepSeek 16B)
- **HuggingFace Uploads**: Professional model cards for all 3 models
- **Comprehensive Testing**: Full A/B baseline for GPT-OSS 20B (N=3, controlled, CUDA-enabled)
- **Real Performance Data**: 71.5% VRAM reduction, 6.9x speed penalty (measured, not estimated)

### Issues Found in Original Whitepaper ❌
1. **Overclaimed novelty**: Said "first implementation" (WRONG - llama.cpp did it first)
2. **Memory contradictions**: 2MB vs 2.33GB vs 1.8GB (inconsistent measurements)
3. **No real baselines**: All "baseline" numbers were estimates
4. **Broken token counting**: word_count × 1.3 (not valid), SSE chunks ≠ tokens
5. **Guessed TTFT**: "10% of total time" (literally made up)
6. **Single runs**: N=1 (no statistical validity)

### Corrections Made ✅
- **Created**: `docs/MOE-TECHNICAL-VALIDATION.md` (honest positioning)
- **Created**: `docs/MOE-WHITEPAPER-CORRECTIONS.md` (audit summary)
- **Archived**: Original whitepaper as reference (problematic version)
- **Positioning**: "Rust bindings + production integration" NOT "first implementation"

### IMMEDIATE PRIORITY: Get Real Baselines
- [⏳] **Run GPT-OSS**: With/without `--cpu-moe`, N=3, measure VRAM/TPS/TTFT
  * Previous run had BROKEN VRAM measurement (0MB/3MB - nonsense)
  * Status: RE-RUNNING with FIXED measure_vram() function (started Oct 8, 20:19 UTC)
  * ETA: ~20 minutes
- [⏳] **Run Phi-3.5-MoE**: With/without `--cpu-moe`, N=3, measure VRAM/TPS/TTFT
  * Previous run had BROKEN VRAM measurement (2MB/1MB - nonsense)
  * Status: NEEDS RE-RUN after GPT-OSS completes
  * Performance data WAS valid: 11.55 TPS baseline, 4.69 TPS offload (2.5x penalty)
- [ ] **Run DeepSeek**: With/without `--cpu-moe`, N=3, measure VRAM/TPS/TTFT
- [ ] **Update report**: Insert REAL baseline data (not fabricated numbers)

### Previous Work (Completed):
#### PR #1: CUDA stdbool Fix (SUBMITTED ✅)
- **Status**: LIVE at https://github.com/utilityai/llama-cpp-rs/pull/839
- **Solution**: Use cc crate to discover MSVC INCLUDE paths, pass to bindgen
- **Tested**: Production use in shimmy v1.6.0 (295/295 tests passing)

#### Issue #81: MoE CPU Offloading (IMPLEMENTED ✅)
- **Status**: Successfully implemented in shimmy feat/moe-cpu-offload branch
- **Achievement**: First working MoE CPU offloading with 99.9% VRAM reduction
- **Validation**: GPT-OSS 20B running with 2MB GPU memory vs 15GB expected

### Shimmy Audit Cleanup (PARKED - Resume After PRs)
- **Status**: Branch `refactor/audit-cleanup-phase1-3` created, pushed to origin
- **Work done**: 73 fixes (I2 getters, N5 unwraps, A3_stringly errors)
- **Tests**: 295/295 passing
- **Plan**: Create PR to main AFTER upstream PRs are complete
- **Remaining**: C3 (pub APIs), A6/A7 (debug prints), P7 (lint suppressions), etc.

### Paid Licensing Feature (WAITING - After Cleanup)
- **Status**: Branch with Ed25519 keygen controls standing by
- **Goal**: Add Claude Code-style subscription licensing ($10-20/month)
- **Value Prop**: Unlimited local inference without time/usage restrictions
- **Market**: Users spending $100/month on Claude who max it out regularly
- **Tech**: Ed25519-based license key validation, time-based activation
- **Strategy**: Keep eyes out during cleanup/refactors for alignment opportunities
- **Priority**: AFTER audit cleanup is complete and codebase is polished

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
