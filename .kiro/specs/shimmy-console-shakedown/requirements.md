# Shimmy Console - Theme Shakedown Specification

## Introduction

Shimmy Console is a Claude Code-level agentic development environment that supports any AI model (local or remote), pluggable themes, pluggable tools, and concurrent multi-agent execution. This specification defines the requirements for validating themes through the shakedown process.

The shakedown is a comprehensive end-to-end validation that proves a theme works correctly with the backend, handles all features, and is production-ready.

## Glossary

- **Theme**: A pluggable UI layer that connects to the Shimmy backend via WebSocket
- **Shakedown**: Comprehensive end-to-end validation of a theme using the 10-scene playbook
- **Sidecar API**: Backend HTTP/WebSocket endpoints that provide models, chat, tools, metrics, and discovery
- **WebSocket**: Unified connection protocol for all theme-to-backend communication
- **Tool**: Pluggable functionality (file ops, git, analysis, system, docs, image reading, etc.)
- **Model**: AI inference engine (local or remote)
- **Discovery Service**: HTTP service on port 11430 that returns backend connection info
- **Concurrent Execution**: Multiple Shimmy instances running simultaneously with independent port allocation

## Requirements

### Requirement 1: Theme Loading & Discovery

**User Story:** As a theme developer, I want my theme to load and discover the backend automatically, so that users don't need to manually configure ports.

#### Acceptance Criteria

1. WHEN a theme loads on localhost:8080 THEN the theme SHALL call the discovery service on port 11430 to get backend information
2. WHEN discovery returns backend info THEN the theme SHALL extract the WebSocket port and base URL
3. WHEN the discovery service is unavailable THEN the theme SHALL fall back to common ports (11434, 11435, 8000, 3000, 5000)
4. WHEN the theme receives discovery info THEN the theme SHALL establish a WebSocket connection to the backend
5. WHEN the WebSocket connection succeeds THEN the browser console SHALL show no red errors

### Requirement 2: Model Discovery & Selection

**User Story:** As a user, I want to see available models and select one, so that I can choose which AI to use.

#### Acceptance Criteria

1. WHEN the theme connects to the backend THEN the theme SHALL call GET /api/models to retrieve the model list
2. WHEN models are retrieved THEN the theme SHALL display model cards with name, memory requirements, and quantization info
3. WHEN a user clicks on a model THEN the theme SHALL send a selection message through WebSocket
4. WHEN a model is selected THEN the chat input SHALL become enabled
5. WHEN the model is selected THEN GET /api/models SHALL return the model with "active" field set to true

### Requirement 3: Chat Streaming

**User Story:** As a user, I want to send messages and see responses stream in real-time, so that I can interact with the AI naturally.

#### Acceptance Criteria

1. WHEN a user types a message and clicks send THEN the theme SHALL send the message through the WebSocket connection
2. WHEN the backend processes the message THEN tokens SHALL stream back through the WebSocket in real-time
3. WHEN tokens arrive THEN they SHALL appear in the chat in the order they were generated (no reordering)
4. WHEN tokens arrive THEN there SHALL be no token duplication
5. WHEN the response completes THEN the message SHALL appear in the chat history

### Requirement 4: Tool Execution

**User Story:** As a user, I want to use tools (file operations, git, analysis, system info, etc.), so that I can extend the AI's capabilities.

#### Acceptance Criteria

1. WHEN a user asks the AI to use a tool THEN the AI SHALL identify the tool and call it through the backend
2. WHEN a tool executes THEN the tool result SHALL display in the chat
3. WHEN a tool fails THEN an error message SHALL display in the chat
4. WHEN multiple tools are called THEN each tool result SHALL display correctly
5. WHEN a tool requires a license THEN the system SHALL validate the license before execution

### Requirement 5: Metrics Display

**User Story:** As a user, I want to see real-time metrics (CPU, memory, tokens/sec), so that I can monitor system performance.

#### Acceptance Criteria

1. WHEN the theme loads THEN a metrics panel SHALL display
2. WHEN the metrics panel displays THEN it SHALL show CPU usage, memory usage, and tokens/sec
3. WHEN the backend provides metrics via GET /api/metrics THEN the theme SHALL display values within tolerance (CPU ±5%, memory ±10%)
4. WHEN messages are sent THEN metrics SHALL update in real-time
5. WHEN multiple messages are sent THEN metrics SHALL remain stable (no memory leaks)

### Requirement 6: Error Handling

**User Story:** As a user, I want the theme to handle errors gracefully, so that I can recover from failures.

#### Acceptance Criteria

1. WHEN a user sends an empty message THEN the theme SHALL display an error message
2. WHEN a user sends a very long message (5000+ chars) THEN the theme SHALL send it successfully
3. WHEN a user sends special characters (emoji, quotes, etc.) THEN the theme SHALL display them correctly
4. WHEN the WebSocket connection drops THEN the theme SHALL attempt to reconnect
5. WHEN an error occurs THEN the browser console SHALL not show red errors

### Requirement 7: Performance & Stability

**User Story:** As a user, I want the theme to remain responsive and stable, so that I can use it for extended periods.

#### Acceptance Criteria

1. WHEN a user sends 10+ messages in sequence THEN the theme SHALL not slow down
2. WHEN messages are being processed THEN CPU usage SHALL remain below 5% idle
3. WHEN messages are being processed THEN memory usage SHALL remain stable
4. WHEN messages are being processed THEN the WebSocket connection SHALL remain open
5. WHEN the theme is used for extended periods THEN the browser console SHALL show no red errors

### Requirement 8: Visual & UX

**User Story:** As a user, I want the theme to look good and work at any resolution, so that I can use it on any device.

#### Acceptance Criteria

1. WHEN the theme loads at 1920x1080 THEN the layout SHALL be stable and readable
2. WHEN the theme loads at 1366x768 THEN the layout SHALL be stable and readable
3. WHEN the theme loads at 1024x768 THEN the layout SHALL be stable and readable
4. WHEN the theme displays THEN colors SHALL be consistent
5. WHEN the theme animates THEN animations SHALL be smooth (< 300ms)

### Requirement 9: Security & Wiring

**User Story:** As a developer, I want the theme to use the discovery service exclusively, so that there are no hardcoded ports or security issues.

#### Acceptance Criteria

1. WHEN the theme connects to the backend THEN it SHALL use the discovery service (port 11430)
2. WHEN the theme connects to the backend THEN it SHALL NOT use hardcoded ports (11435, 11434)
3. WHEN the theme communicates THEN it SHALL use WebSocket protocol (ws:// or wss://)
4. WHEN the theme sends data THEN it SHALL NOT include sensitive data in URLs
5. WHEN the backend processes requests THEN the backend logs SHALL show no errors

### Requirement 10: Concurrent Execution

**User Story:** As a developer, I want to run multiple Shimmy instances simultaneously, so that I can use multiple agents in parallel.

#### Acceptance Criteria

1. WHEN multiple Shimmy instances start THEN each instance SHALL get a unique port
2. WHEN multiple instances run THEN each instance SHALL have independent state
3. WHEN multiple instances run THEN each instance SHALL have independent tool execution
4. WHEN multiple instances run THEN each instance SHALL have independent metrics
5. WHEN multiple instances run THEN they SHALL not interfere with each other

## Success Criteria

A theme is **production ready** when:

- ✅ All 10 requirements pass
- ✅ All 10 scenes of the shakedown playbook pass
- ✅ No red errors in browser console
- ✅ Chat streaming works end-to-end
- ✅ All tools execute correctly
- ✅ Metrics display accurately
- ✅ Performance is acceptable
- ✅ PPT invariant tests written and passing
