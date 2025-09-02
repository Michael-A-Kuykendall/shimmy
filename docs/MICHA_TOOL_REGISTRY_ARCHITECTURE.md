# Micha's Personal Tool Registry & Context System Architecture

## Problem Statement
- Multiple powerful tools (punch, shimmy, rustchain, etc.) scattered across repositories
- No centralized discovery mechanism for AI agents to understand available tools
- Repeated context explanation required for each new agent session
- Need for automated context database population for VS Code extension (context-light)

## Architecture Recommendations

### Approach 1: Centralized Tool Registry (RECOMMENDED)

#### Location Options:
1. **`C:\Users\micha\.micha-tools\registry.json`** (User profile - always accessible)
2. **`C:\Users\micha\repos\micha-toolchain\`** (Dedicated repo - version controlled)
3. **Environment variable `MICHA_TOOLS_REGISTRY`** (Flexible configuration)

#### Registry Schema:
```json
{
  "version": "1.0",
  "updated": "2025-08-28T22:55:00Z",
  "tools": {
    "punch": {
      "name": "Punch Discovery",
      "path": "C:\\Users\\micha\\repos\\punch-discovery",
      "type": "cli",
      "description": "Advanced codebase analysis and profiling tool",
      "capabilities": [
        "repository analysis",
        "code metrics",
        "dependency mapping",
        "architecture insights"
      ],
      "api_endpoints": [],
      "cli_commands": [
        "punch discover",
        "punch analyze",
        "punch profile"
      ],
      "documentation": "C:\\Users\\micha\\repos\\punch-discovery\\README.md",
      "status": "active",
      "last_verified": "2025-08-28T22:55:00Z"
    },
    "shimmy": {
      "name": "Shimmy AI Inference Server",
      "path": "C:\\Users\\micha\\repos\\shimmy",
      "type": "server",
      "description": "Local-first AI inference with OpenAI compatibility",
      "capabilities": [
        "local LLM inference",
        "OpenAI API compatibility",
        "model auto-discovery",
        "hot model swapping",
        "tool calling framework"
      ],
      "api_endpoints": [
        "GET /health",
        "POST /api/generate",
        "POST /v1/chat/completions",
        "GET /api/models",
        "POST /api/models/discover"
      ],
      "cli_commands": [
        "shimmy serve",
        "shimmy list",
        "shimmy discover"
      ],
      "default_port": 11435,
      "documentation": "C:\\Users\\micha\\repos\\shimmy\\README.md",
      "status": "active",
      "last_verified": "2025-08-28T22:55:00Z"
    },
    "rustchain": {
      "name": "RustChain AI Agent Framework",
      "path": "C:\\Users\\micha\\repos\\rustchain-community",
      "type": "framework",
      "description": "AI agent mission execution and orchestration",
      "capabilities": [
        "mission-driven development",
        "agent orchestration",
        "command execution",
        "verification systems"
      ],
      "api_endpoints": [],
      "cli_commands": [
        "rustchain run",
        "rustchain mission",
        "rustchain interactive"
      ],
      "documentation": "C:\\Users\\micha\\repos\\rustchain-community\\README.md",
      "status": "active",
      "last_verified": "2025-08-28T22:55:00Z"
    }
  },
  "integrations": {
    "context_light": {
      "enabled": true,
      "database_path": "${project_root}/.context-light/tools.db",
      "auto_populate": true
    },
    "claude_copilot": {
      "enabled": true,
      "instruction_injection": true,
      "global_context_file": "C:\\Users\\micha\\.micha-tools\\claude-context.md"
    }
  }
}
```

### Approach 2: VS Code Extension Integration

#### Context-Light Extension Architecture:
```
context-light/
├── extension.ts          # Main extension entry
├── registry/
│   ├── tool-scanner.ts   # Auto-discovery of tools
│   ├── database.ts       # SQLite operations
│   └── sync.ts          # Registry synchronization
├── databases/
│   ├── {project}.db     # Per-project context database
│   └── global-tools.db  # Global tool registry
└── templates/
    ├── claude-context.md
    └── copilot-instructions.md
```

#### Database Schema:
```sql
-- Global tool registry
CREATE TABLE tools (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    type TEXT NOT NULL, -- cli, server, framework, library
    description TEXT,
    capabilities TEXT, -- JSON array
    api_endpoints TEXT, -- JSON array
    cli_commands TEXT, -- JSON array
    documentation_path TEXT,
    status TEXT DEFAULT 'active',
    last_verified DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Per-project tool usage
CREATE TABLE project_tools (
    project_id TEXT,
    tool_id TEXT,
    usage_frequency INTEGER DEFAULT 0,
    last_used DATETIME,
    custom_config TEXT, -- JSON
    PRIMARY KEY (project_id, tool_id)
);

-- Context injection templates
CREATE TABLE context_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    template TEXT NOT NULL,
    target TEXT NOT NULL, -- claude, copilot, general
    auto_inject BOOLEAN DEFAULT true
);
```

## Implementation Strategy

### Phase 1: Registry Foundation (1-2 days)
1. **Create central registry file** at `C:\Users\micha\.micha-tools\registry.json`
2. **Build discovery scanner** that finds tools in `C:\Users\micha\repos\`
3. **Auto-populate initial registry** with punch, shimmy, rustchain

### Phase 2: Context Integration (2-3 days)
1. **Develop VS Code extension skeleton** for context-light
2. **Implement SQLite database layer** with tool registry sync
3. **Create context injection templates** for Claude/Copilot

### Phase 3: Auto-Discovery & Maintenance (1-2 days)
1. **Registry update automation** (scan for new tools weekly)
2. **Health checking system** (verify tool availability)
3. **Context template generation** (auto-update instructions)

## Benefits of This Approach

### For AI Agents:
- **Instant tool awareness** via registry lookup
- **Standardized capability discovery** across all tools
- **Consistent API/CLI interface documentation**
- **Automated context injection** eliminates repetitive explanations

### For Development Workflow:
- **Single source of truth** for tool inventory
- **Version-controlled tool metadata** (if using repo approach)
- **Project-specific tool usage tracking** via context-light
- **Automated documentation updates** when tools change

### For Context-Light Extension:
- **Per-project databases** with relevant tool subset
- **Usage analytics** to prioritize context injection
- **Smart template selection** based on project type
- **Real-time registry synchronization**

## Next Steps Recommendation

1. **Start with Approach 1** - create the centralized registry first
2. **Use punch tool to analyze your existing repositories** and auto-populate registry
3. **Build context-light extension incrementally** with registry integration
4. **Inject registry-based context into all claude.md files** across projects

This gives you immediate benefits (centralized tool awareness) while building toward the full context-light vision.

Would you like me to implement the initial registry creation and scanner?
