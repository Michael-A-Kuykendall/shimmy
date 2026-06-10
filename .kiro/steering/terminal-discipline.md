# Terminal Discipline — Three Hard Rules

## Rule 1: One General-Purpose Tab (reuse it)

There is always exactly ONE general-purpose terminal tab named **`general`**.
Use it for: cargo builds, git commands, file checks, quick lookups — anything that runs and completes.
Never open a new tab for something you can do in `general`.

## Rule 2: Name Every Background Tab to Its Purpose

If something MUST run in the background (a server, a long download),
open ONE named tab for it. The name must describe exactly what it does:
- `shimmy-server` — running shimmy serve
- `mixtral-download` — downloading a model
Never use generic names. Never open two tabs for the same purpose.

## Rule 3: Stop and Clean Up Before Opening Anything New

Before opening ANY new tab:
1. `list_processes` — see what is running
2. If count >= 2 background tabs → stop something first
3. Finished tab → `control_pwsh_process stop <id>` immediately

**Maximum background tabs: 2. Maximum total tabs: 3.**

---

## The Root Cause This Fixes

Every `control_pwsh_process start` opens a new Cygwin bash console.
Windows/Cygwin hard limit: 32 consoles.
Hitting it crashes all terminals.
Use `execute_pwsh` (blocking, self-cleaning) for 95% of work.

---

## Decision Guide

| Task | Tool |
|------|------|
| cargo build/test/clippy | `execute_pwsh` in `general` |
| git add/commit/push | `execute_pwsh` in `general` |
| shimmy serve (dev server) | `control_pwsh_process start` → named `shimmy-server` |
| Long model download | `control_pwsh_process start` → named `<model>-download` |

## Shimmy-Specific: Server Lifecycle

- Default port: `11435`
- Always check `/health` `uptime_seconds` after rebuild — if large, you're testing stale binary
- Stop → rebuild → restart. Never skip the stop.
