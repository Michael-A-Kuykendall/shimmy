# Shimmy Console Architectural Deep Dive

## System Overview
Shimmy Console is a local AI development platform combining inference server capabilities with agentic tooling and themeable interfaces. Built in Rust with Node.js themes.

## Layered Architecture

### Layer 1: Core Inference (Shimmy Base)
- **Purpose**: Provide fast, local AI inference
- **Components**:
  - Engine adapters (Llama.cpp primary)
  - Model registry and loading
  - Basic REST API
- **Tech**: Rust, axum, tokio

### Layer 2: Console Extension
- **Purpose**: Add agentic development features
- **Components**:
  - Tool system (16 pre-built + snap-in)
  - Session management (history, metrics)
  - Discovery/orchestration
- **Tech**: Rust feature flag

### Layer 3: Frontend Ecosystem
- **Purpose**: User interfaces and themes
- **Components**:
  - Frontend contract (WebSocket protocol)
  - Theme generator (internal)
  - Theme validator (external)
  - 10 pre-made themes + custom
- **Tech**: React/Vite, WebSocket

### Layer 4: User Experience
- **Purpose**: Seamless theme switching with context
- **Components**:
  - Model chooser
  - Chat interface with streaming
  - Tool integration
  - Metrics dashboard
- **Tech**: Theme-specific (React, etc.)

## Data Flow Architecture
1. **Server Startup**: Shimmy runs on 11435, registers with discovery
2. **Theme Launch**: CLI starts theme dev server on 8080
3. **Connection**: Theme WebSocket connects to `/ws/console`
4. **Initialization**: get_models → select_model
5. **Interaction**: generate requests → streaming responses + metrics
6. **Tools**: Agent executes tools via console backend
7. **Persistence**: Session history in REDB, metrics tracked

## Strengths
- **Local First**: No cloud dependency, private
- **Extensible**: Snap-in tools/themes
- **Multi-Frontend**: Terminal to web
- **Tool Parity**: Matches commercial AI assistants
- **Themeable**: Full customization

## Current State Assessment
We're very close to a working system. The core plumbing (WebSocket, streaming, tools) is solid. The theme system with contract is well-designed. Metrics integration is partially implemented.

## Improvement Ideas (For Future Iteration)
1. **Unified State Management**: Add a central state store (e.g., Redux-like) across themes for seamless context switching.
2. **Enhanced Metrics**: Expand ShimmyMetrics to include tool usage, session duration, error rates.
3. **Tool Orchestration**: Add workflow engine for chaining tools (e.g., "analyze code then git commit").
4. **Theme Hot-Reload**: Auto-reload themes on changes during dev.
5. **Security Hardening**: Sandbox tool execution, validate theme code.
6. **Performance**: Optimize WebSocket for low-latency streaming, add compression.
7. **Testing**: Comprehensive integration tests for theme contract compliance.
8. **Documentation**: Auto-generate API docs from contract.
9. **Plugin System**: Formalize snap-in architecture with manifests.
10. **AI Awareness**: Enhance context passing (file system awareness, project structure).

## Why We're Close
- Infrastructure is solid (Rust async, WebSocket streaming).
- Contract provides clean separation.
- Tools are functional.
- Themes can be generated/validated.

## Next Steps (Post-Shakedown)
1. Fix any remaining theme integration bugs.
2. Add missing metrics fields.
3. Polish tool UX.
4. Launch with shimmy-default theme.
5. Expand theme ecosystem.

The architecture is sound—focus on execution and polish.