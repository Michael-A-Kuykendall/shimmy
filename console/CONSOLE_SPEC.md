# Shimmy Console — Local AI Development Assistant Specification

## Philosophy

Shimmy is a shim. The console is the agentic layer on top of `shimmy serve`.

**Goal:** Claude Code parity via incremental phases. The inference is done — this is feature work.

The console connects to `shimmy serve` at `/v1/chat/completions`, dispatches tool calls locally, injects results back into the conversation, and loops until the model produces a final text response. All tool execution is local, in-process, with user-level permissions.

---

## Current State — Phase 0 (COMPLETE)

All of the following are implemented and compiling:

**11 tools:**
- `read_file` — file_ops.rs
- `write_file` — file_ops.rs
- `list_files` — file_ops.rs
- `git_status` — git.rs
- `git_diff` — git.rs
- `git_commit` — git.rs
- `git_log` — git.rs
- `analyze_project` — analysis.rs
- `syntax_check` — analysis.rs
- `shell_command` — command.rs
- `system_info` — system.rs

**Infrastructure:**
- Agentic chat loop: sends to `/v1/chat/completions`, dispatches tool calls, injects results
- WebSocket stub at `/ws/console`
- License bypass (dev mode)

---

## MVP — Phase 1 (TARGET)

These items complete the minimum viable console:

### 1. `docs.rs` — ExplainCommandTool + GetHelpTool
- `explain_command`: runs `<command> --help 2>&1`, returns real help output. Falls back gracefully.
- `get_help`: static help listing all available tools. For `topic="general"` shows all; for a specific tool name shows that tool's details.

### 2. `image.rs` — ReadImageTool
- `read_image`: reads image file, returns base64 + MIME type + dimensions.
- Modes: `base64` (full data), `meta` (metadata only, no base64), `ocr` (placeholder, includes base64).
- PNG dimension parsing from header bytes (no image crate needed).
- MIME detection from magic bytes.

### 3. `loader.rs` — ToolManifest snap-in system
- Load `tools/manifest.json` (optional, tolerated if absent).
- Format: `{ "tools": { "tool_name": { "enabled": false } } }`
- Enable/disable tools by name.
- `apply_manifest()` removes disabled tools from registry.

### 4. `config.rs` — ConsoleConfig struct
- `base_url`: shimmy serve URL (default `http://127.0.0.1:11435`)
- `model`: model name (default `"default"`)
- `session_dir`: path to `~/.shimmy/sessions/`
- `tools_dir`: path to `~/.shimmy/tools/`
- `log_level`: string log level

### 5. `sessions.rs` — execute_session_command
- `list`: print all session files with timestamps
- `show <id>`: print session conversation
- `delete <id>`: remove session file

### 6. Session persistence
- Save/load conversations to `~/.shimmy/sessions/<id>.json`
- Session ID: UUID v4
- Format: `{ "id": "...", "created": "...", "messages": [...] }`

### 7. Workspace context injection
- Inject file tree + recent git log into system prompt on chat start
- Respects `.gitignore` (best-effort; uses `list_files` tool output)

---

## Phase 2 — Roadmap

- **Web search** via shimmy serve HTTP endpoint (not external API)
- **HuggingFace Hub model sourcing** — `hf://` protocol in serve
- **MCP server support** — pass `mcp_servers` to `/v1/chat/completions`
- **Streaming token display** in terminal (SSE parsing)
- **Multi-turn interactive REPL** mode (currently exits after one response)

---

## Phase 3 — Roadmap

- **Vision/image analysis** — real OCR via tesseract or cloud
- **Audio transcription**
- **Browser automation** via headless chromium
- **Artifact rendering** — HTML/React components in terminal

---

## Tool Inventory

| # | Name | File | Status | Description |
|---|------|------|--------|-------------|
| 1 | `read_file` | file_ops.rs | DONE | Read file contents with optional line range |
| 2 | `write_file` | file_ops.rs | DONE | Write content to file, create parent dirs |
| 3 | `list_files` | file_ops.rs | DONE | List directory contents |
| 4 | `git_status` | git.rs | DONE | `git status --porcelain` with parsed output |
| 5 | `git_diff` | git.rs | DONE | Git diff for file or entire repo |
| 6 | `git_commit` | git.rs | DONE | Stage all and commit with message |
| 7 | `git_log` | git.rs | DONE | Recent commit history (one-line format) |
| 8 | `analyze_project` | analysis.rs | DONE | Project structure, dependencies, type detection |
| 9 | `syntax_check` | analysis.rs | DONE | Syntax validation (multi-language) |
| 10 | `shell_command` | command.rs | DONE | Execute shell commands, return stdout/stderr |
| 11 | `system_info` | system.rs | DONE | OS, arch, cwd, home dir |
| 12 | `explain_command` | docs.rs | DONE | Run `--help` and return output |
| 13 | `get_help` | docs.rs | DONE | Static tool directory listing |
| 14 | `read_image` | image.rs | DONE | Base64/meta/OCR modes for image files |

---

## Claude Code Parity Matrix

| Claude Tool | Shimmy Equivalent | Status | Notes |
|-------------|-------------------|--------|-------|
| `bash_tool` | `shell_command` | ✅ Parity | Full stdout/stderr/exit code |
| `create_file` | `write_file` | ✅ Parity | Creates parent dirs |
| `str_replace` | — | ❌ Gap | Phase 2 candidate |
| `view` (file) | `read_file` | ✅ Parity | Line range supported |
| `view` (dir) | `list_files` | ✅ Parity | Flat listing (no depth limit yet) |
| `web_search` | — | ❌ Gap | Phase 2 via shimmy serve |
| `web_fetch` | — | ❌ Gap | Phase 2 via shimmy serve |
| `image_search` | — | ❌ Gap | Out of scope for Phase 1 |
| `present_files` | — | ❌ Gap | Terminal: no UI layer yet |
| `visualize:show_widget` | — | ❌ Gap | Phase 3 artifact rendering |
| Git commands | `git_status/diff/commit/log` | ✅ Partial | No branch/checkout/push yet |
| `explain_command` | `explain_command` | ✅ Parity | Uses real `--help` output |
| Image reading | `read_image` | ⚠️ Partial | OCR placeholder only |
| MCP integration | — | ❌ Gap | Phase 2 |

**Honest assessment:** shimmy console covers ~60% of Claude Code's day-to-day utility. The two biggest gaps are `str_replace` (surgical file editing) and web access. Both are roadmapped.

---

## Architecture

```
shimmy serve (GPU inference via airframe)
    ↓  /v1/chat/completions
console chat loop  (console/src/commands/chat.rs)
    ↓  tool_calls in response
ToolRegistry  (console/src/tools/mod.rs)
    ↓  Arc<dyn Tool>::execute()
tool implementations  (file_ops, git, analysis, command, system, docs, image)
    ↓  ToolResult { success, output, data }
injected back as "tool" role messages
    ↓
model sees results → next response
```

The loop runs up to `MAX_TOOL_ROUNDS = 10` before forcing a final response, preventing runaway agentic cycles.

### Key types

```rust
// Tool contract
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;  // JSON Schema
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError>;
}

// Args: flat HashMap<String, Value> with typed helpers
args.require_str("path")?   // error if missing
args.get_str("mode")        // Option<&str>
args.get_bool("flag", false)
args.get_i64("limit")

// Results
ToolResult::success(output)
ToolResult::success_with_data(output, json_data)
ToolResult::failure(output)
```

### Dependency policy

Minimal. No heavy crates added for Phase 1. `base64 = "0.21"` added for `read_image`. Everything else uses stdlib or existing Cargo.toml deps.
