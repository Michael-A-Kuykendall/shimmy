# 📋 CURRENT STATUS - Oct 6, 2025

## Active Work: MoE Model Testing & Documentation Mission 🚀

### MISSION: MoE CPU Offloading Multi-Model Testing & Documentation
- **Status**: ACTIVE - Phase 1 (GPT-OSS) COMPLETE ✅
- **Goal**: Test, convert, and document 3-5 MoE models with CPU offloading
- **Current Phase**: Complete GPT-OSS documentation → Find alternatives → Convert & test → Publish

### Phase 1: GPT-OSS 20B - COMPLETE ✅
- **HuggingFace**: https://huggingface.co/MikeKuykendall/gpt-oss-20b-moe-cpu-offload-gguf
- **Performance**: 99.9% VRAM savings (2MB vs 15GB), 32 experts, 4 active/token
- **Status**: Successfully uploaded 81.5GB with professional documentation
- **Architecture**: 24 layers, 131K context, F16 precision, sliding window attention
- **Memory**: CPU expert offloading working perfectly in shimmy feat/moe-cpu-offload

### Phase 2: Documentation & Benchmarking - IN PROGRESS
- **White Paper**: Create comprehensive MoE CPU offloading research document
- **Benchmarks**: Speed tests, memory usage, performance metrics
- **Documentation**: Technical specifications, usage patterns, comparative analysis

### Phase 3: Alternative Models - PENDING
- **Target**: Find 2-4 additional MoE models suitable for GH200 instance
- **Process**: Convert to GGUF, test CPU offloading, benchmark performance
- **Criteria**: Must have proper MoE expert tensors (unlike Mixtral issues we found)
- **Goal**: Professional HuggingFace uploads for each working model

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
