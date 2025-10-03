# 🚨 ACTIVE FIX TRACKER - Issue #72: GPU Backend Not Working

## Problem Summary
- **Reporter**: D0wn10ad
- **Issue**: `--gpu-backend` flag (auto/vulkan/opencl) is ignored; all layers assigned to CPU
- **Version**: 1.5.6 (built from source)
- **Build**: `cargo build --release --no-default-features --features huggingface,llama-opencl,llama-vulkan`
- **GPU**: Works with standalone llama.cpp on same hardware (Vulkan confirmed working)
- **Evidence**: All 29 layers show `load_tensors: layer N assigned to device CPU, is_swa = 0`

## Root Cause Analysis
The `--gpu-backend` CLI flag is accepted but not passed through to llama.cpp backend initialization.
Need to trace: CLI arg → model loading → llama.cpp params.

## Files to Investigate
1. `src/cli.rs` - Check if `--gpu-backend` is parsed and stored
2. `src/engine/llama.rs` - Check if GPU backend param is used during model load
3. `src/model_registry.rs` - Check if GPU config is passed to engine
4. Check llama-cpp-2 crate docs for proper GPU initialization

## Fix Plan
1. ✅ Understand issue from logs and user report
2. ✅ Locate where `--gpu-backend` CLI flag is defined (src/cli.rs line 28)
3. ✅ Trace flag through to model loading code (FOUND THE BUG!)
4. ✅ Find llama-cpp-2 GPU initialization API (`with_n_gpu_layers()`)
5. ✅ Wire GPU backend selection into model load
   - ✅ Modified `LlamaEngine::new_with_backend()` to accept GPU backend
   - ✅ Parse CLI gpu_backend string to GpuBackend enum
   - ✅ Pass n_gpu_layers to model params based on backend
   - ✅ Build successful with llama-vulkan feature
6. [ ] Test with vulkan/opencl/auto settings
7. [ ] Add regression test to prevent future breakage
8. [ ] Commit, push, respond to issue

## Bug Root Cause (CONFIRMED)
- CLI parses `--gpu-backend` ✅
- `LlamaEngine` has `gpu_backend` field ✅  
- **BUT**: `gpu_backend` field is NEVER USED in model loading ❌
- **AND**: CLI value is NEVER PASSED to engine constructor ❌
- Model loads with default params → no GPU layers → CPU only

## Testing Requirements
- Build with `--features llama-vulkan,llama-opencl`
- Verify `shimmy gpu-info` shows backends enabled
- Verify `shimmy serve --gpu-backend vulkan` assigns layers to GPU (not CPU)
- Check logs show `load_tensors: layer N assigned to device Vulkan/OpenCL`

---

# Copilot / AI Assistant Operating Guide for Shimmy

This file teaches any AI assistant how to work effectively inside this repository. Keep replies lean, perform actions directly, and favor incremental verified changes.

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
