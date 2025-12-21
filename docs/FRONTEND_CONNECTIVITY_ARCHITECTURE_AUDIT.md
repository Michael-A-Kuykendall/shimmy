# Frontend Connectivity Architecture Audit

Date: 2025-11-07
Branch: feature/discovery-service
Owner: Shimmy (local inference shim)

## Purpose
Document all layers involved in connecting themes to Shimmy; map dependencies; evaluate alternatives (IPC, HTTP, WS shim); stress-test for multi-instance dev (5–11 concurrent instances); and recommend a simplified, robust blueprint to freeze the frontend contract and guide the theme validator.

---

## System Overview
- Shimmy core: Axum-based server + llama backend
- Protocols today:
  - WebSocket: `/ws/console`, `/ws/generate` (token streaming)
  - HTTP: `/api/models`, `/api/generate`, `/api/metrics`, token endpoints
  - Discovery:
    - IPC Named Pipe (Windows) `\\.\pipe\shimmy.discovery` (authoritative)
    - HTTP Discovery Service (browser fallback) binding 11430–11439 today
- Themes: standalone frontends (32bit, cyberpunk, retro) using discovery-client
- Theme Validator: Node/TS CLI that asserts contract conformance

---

## Dependency Graph (Mermaid)
```mermaid
flowchart TD
  subgraph Themes
    T32[32bit Theme]
    TCP[Cyberpunk Theme]
    TR[Retro Theme]
    VAL[Theme Validator]
  end

  subgraph Discovery
    IPC[IPC Discovery (Named Pipe)]
    HDISC[HTTP Discovery Service]
  end

  subgraph Shimmy Server
    WS[/WebSocket Endpoints\n/ws/console, /ws/generate/]
    HTTP[/HTTP API\n/api/models, /api/generate, /api/metrics/]
    ADAPT[HTTP → WS Adapter (proposed canonical path)]
    DISP[WS Dispatcher (single-source logic)]
  end

  subgraph Engine
    LLAMA[llama.cpp backend]
    REG[Model Registry]
    HIST[History (ReDB)]
    TOK[Token Meter (ReDB)]
  end

  T32 -->|discover| IPC
  TCP -->|discover| IPC
  TR  -->|discover| IPC
  VAL -->|discover| IPC
  VAL -->|fallback| HDISC
  T32 -->|fallback| HDISC

  IPC -->|assign port + ws uri| WS
  HDISC -->|returns port + ws uri| WS

  WS --> DISP --> LLAMA
  DISP --> REG
  DISP --> HIST
  DISP --> TOK

  HTTP --> ADAPT --> DISP
```

---

## End-to-End Flows

### 1) Canonical WS Flow (Preferred)
1. Theme (or Validator) requests discovery via IPC.
2. Discovery returns chosen bind (host:port) and WS routes.
3. Theme connects to `/ws/console`:
   - Fetch models (message: `list_models`)
   - Select model (message: `select_model`)
   - Start chat/generation (message: `generate`)
4. Tokens stream over WS; metrics are surfaced via periodic `metrics` events.

### 2) Browser Fallback via HTTP Discovery + WS
1. Theme can’t access IPC; hits HTTP discovery service on a fixed port (see Recommendation).
2. Receives the bound host:port and WS routes.
3. Continues with the WS flow as above.

### 3) HTTP API via Adapter (Legacy/Optional)
- `/api/models`, `/api/generate` exist only as thin adapters that translate request/response to WS dispatcher calls, ensuring single logic path.
- Streaming via SSE remains supported by translating WS token frames to SSE `data:` events.

---

## Multi‑Instance Strategy
- Requirement: 5–11 concurrent Shimmy instances during development.
- Approach:
  - IPC as primary discovery: the leader assigns ephemeral bind ports per instance.
  - HTTP Discovery: one fixed port (e.g., 11430) acting as a registry that enumerates currently active instances and resolves a target by PID/name/session.
  - Themes can select instance by `SHIMMY_INSTANCE_ID` (env or query) when needed; default is "first available".
- Benefits:
  - No hardcoded app ports per instance
  - Browsers don’t need privileged port juggling
  - Cross-platform compatibility (Windows Named Pipe, Unix domain socket)

---

## Options Audit

### A) Keep Dual Logic (Current)
- WS endpoints for console/chat + independent HTTP handlers calling engine directly.
- Pros: Already works; simple mental model for HTTP clients.
- Cons: Two codepaths, higher maintenance, drift risk; validator + contract have to cover both.

### B) WS Canonical + HTTP→WS Adapter (Recommended)
- Single dispatcher for all operations; HTTP endpoints just translate.
- Pros: One source of truth; easier to evolve; consistent streaming semantics; future control frames (cancel, temp) unified.
- Cons: Small adapter layer work; need clear dispatcher API.

### C) HTTP‑Only (Dump WS)
- All streaming via SSE or chunked HTTP.
- Pros: Simpler for some clients.
- Cons: Loses mid‑stream bi‑directional control; harder to extend; long‑lived connections less flexible.

### D) Pure IPC + Embedded WS (No HTTP at all)
- Themes always connect via IPC → negotiated WS; no public HTTP endpoints.
- Pros: Minimal surface area; less ambiguity.
- Cons: Browsers can’t use IPC; would require helper/bridge always.

---

## Discovery Architecture Choices
- IPC First: authoritative, no port guessing, solid for multi‑instance.
- HTTP Discovery: keep but consolidate to a single fixed port (11430) instead of a range; maps instance IDs to host:port.
- Remove port numbers from theme config; discovery returns full URIs.

---

## Contract Implications
- Frontend Contract becomes WS‑first: message schemas (list_models, select_model, generate, metrics, errors).
- HTTP Contract is explicitly a shim: same payload shapes as WS where applicable; streaming mapped to SSE.
- Validator:
  - Declares mode (ipc or http) in `shimmy.theme.json`.
  - Verifies no hardcoded ports; discovery used; WS messages and event shapes validated.

---

## Security & Robustness
- Validate discovery requests (path traversal, malformed IDs).
- Timeouts and retries for discovery and WS connect.
- Backpressure and token throttling surfaced via WS control frames.
- Database (ReDB) locks: ensure clean startup/shutdown; add recovery note to docs.

---

## Performance Notes
- WS vs SSE throughput close; WS preferred for control flexibility.
- HTTP→WS adapter overhead negligible compared to generation cost.
- Multi‑instance only affects discovery path, not per‑token latency.

---

## Recommendation
- Adopt Option B: WS Canonical + HTTP→WS Adapter.
- Consolidate discovery to: IPC primary + single fixed HTTP discovery port (11430) as fallback.
- Freeze message schemas and update FRONTEND_CONTRACT.md accordingly.
- Update Validator to be contract‑centric with mode manifest and WS message checks.

### Rationale
- Single logic path reduces bugs and drift.
- Supports heavy multi‑instance dev without manual port wrangling.
- Keeps browser compatibility via minimal, well‑defined shim.

---

## Next Steps
1. Update contract doc to WS‑first and single HTTP discovery port.
2. Introduce WS dispatcher trait and rewire HTTP handlers to the adapter.
3. Update validator for manifest + WS message tests.
4. Add doc for multi‑instance selection + discovery registry.
5. Ship small tests (unit for adapter, validator checks for message shapes).
