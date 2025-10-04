# 📋 CURRENT STATUS - Code Cleanup & Audit Phase

**Status as of 3:40 PM, Oct 3, 2025:**

## What We're Doing Right Now
Comprehensive codebase audit and cleanup before cutting release tag for Issue #72.

## What We Just Completed
1. ✅ Fixed Issue #72 - GPU backend flag properly wired to model loading
2. ✅ All 13 GPU backend regression tests passing  
3. ✅ Removed unused specs (001, 002, 004) and dead code (ModelCache, RouteManager)
4. ✅ Fixed all clippy errors - build clean
5. ✅ Updated all "sub-20MB" references back to "sub-5MB" (actual: 4.8MB)
6. ✅ Toned down AI marketing language in README
7. ✅ Committed cleanup: f4a12f7

## Current Phase
Running comprehensive audit:
- Removing stale log files from root
- Cleaning up test artifacts
- Verifying directory structure
- Running full regression suite

## Next Steps
1. Complete regression tests
2. Final commit of audit cleanup
3. Cut release tag (v1.5.7 or v1.6.0)
4. Respond to Issue #72 with tag info
5. Wait for @D0wn10ad verification before closing

## Key Files Modified
- Updated binary size references: 4.8MB (142x smaller than Ollama)
- Constitution restored to sub-5MB limit
- README cleaned of AI fluff
- Cargo.toml, ROADMAP.md, CLAUDE.md all updated

---

# 🚨 ACTIVE FIX TRACKER - Issue #72: GPU Backend Not Working ✅ RESOLVED

## Problem Summary
- **Reporter**: D0wn10ad
- **Issue**: `--gpu-backend` flag (auto/vulkan/opencl) was ignored; all layers assigned to CPU
- **Version**: 1.5.6 (built from source)
- **Build**: `cargo build --release --no-default-features --features huggingface,llama-opencl,llama-vulkan`
- **GPU**: Works with standalone llama.cpp on same hardware (Vulkan confirmed working)
- **Evidence**: All 29 layers showed `load_tensors: layer N assigned to device CPU, is_swa = 0`

## Root Cause (CONFIRMED)
- CLI parsed `--gpu-backend` ✅
- `LlamaEngine` had `gpu_backend` field ✅  
- **BUT**: `gpu_backend` field was NEVER USED in model loading ❌
- **AND**: CLI value was NEVER PASSED to engine constructor ❌
- Model loaded with default params → no GPU layers → CPU only

## Fix Implementation ✅ COMPLETE
1. ✅ Added `LlamaEngine::new_with_backend(Option<&str>)` constructor
2. ✅ Implemented `GpuBackend::from_string()` parser with helpful error messages
3. ✅ Implemented `GpuBackend::detect_best()` with priority: CUDA > Vulkan > OpenCL > CPU
4. ✅ Implemented `GpuBackend::get_gpu_layers()` returning 999 for GPU, 0 for CPU
5. ✅ Modified model loading to call `.with_n_gpu_layers(n_gpu_layers)`
6. ✅ Wired CLI `--gpu-backend` through all engine instantiation points (serve, generate, gpu-info)
7. ✅ Added detection checks (vulkaninfo, clinfo) for runtime validation

## Testing & Verification ✅ ALL PASSING
- ✅ Build successful with `--features huggingface,llama-opencl,llama-vulkan`
- ✅ All 13 GPU backend regression tests passing (gpu_backend_tests.rs + gpu_layer_verification.rs)
- ✅ Test suite validates backend selection, CLI flag respect, multi-backend auto-detection
- ✅ Added Issue #72 to release gate (`scripts/run-regression-tests.sh` Phase 5)
- ✅ Manual verification: `shimmy gpu-info --gpu-backend vulkan` shows "Vulkan" backend selected

## Commits
- cc82cec9 - Core fix: Wire --gpu-backend CLI flag through to model loading
- 40790a2f - Add GPU layer configuration support  
- 28e07340 - Add Issue #72 regression tests to release gate

## Status: READY FOR USER VERIFICATION
- All tests passing
- Release gate updated  
- Need user to test with actual model loading and verify logs show GPU layer assignment

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

## RustChain Mission-Driven Development

Shimmy development now follows a mission-driven approach using RustChain AI agent framework:

### Mission Management Structure
- `docs/mission-stacks/hopper/` - Upcoming missions (priority ordered)
- `docs/mission-stacks/current/` - Active mission and related submissions
- `docs/mission-stacks/done/` - Completed missions (archived)

### Mission Workflow
1. **Mission Planning**: Create comprehensive YAML missions in hopper/ with:
   - Clear verification criteria and tests
   - Gated epic structure with dependencies
   - Specific deliverables and acceptance criteria
   - Integration points with existing codebase

2. **Mission Execution**: 
   - Move mission from hopper/ to current/ when starting
   - AI assistant executes mission using RustChain
   - Create submissions for needed corrections if verification fails
   - All related files stay in current/ during active work

3. **Mission Completion**:
   - Verify mission passes all defined tests
   - Move completed mission and outputs to done/
   - Update project status and next mission priority

### Mission Standards
- Each mission MUST have verifiable completion criteria
- Include build tests, functional tests, and integration checks  
- Missions should be granular but coherent (1-3 day scope)
- Dependencies clearly defined between missions
- Champion LLM (llama32-champion) provides domain expertise

### AI Assistant Mission Responsibilities
- Execute RustChain missions in order of priority
- Run verification tests and QA each mission
- Create corrective submissions when missions fail verification
- Pause for user input only when mission requirements unclear
- Report mission completion status and next recommended actions

### Champion LLM Integration
- Use llama32-champion model for shimmy-specific analysis
- Leverage champion's training on user's development patterns
- Champion provides architecture guidance and implementation strategy
- Regular champion consultation on complex technical decisions

This mission-driven approach ensures systematic, verified progress toward shimmy's goals of becoming a robust local-first AI serving solution.
