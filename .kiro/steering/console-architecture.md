# Shimmy Console — Architecture

**Corrected:** 2026-06-10  
**Authoritative source:** `docs/internal/CONSOLE_WORKSTREAM_2026-06-10.md` — READ THIS FIRST.

---

## What `shimmy console` IS

A **browser-based themed UI**, shipped as a paid sub-feature of shimmy.

```
shimmy console [theme]
  → finds free port
  → spawns `shimmy serve` as a child process (no second terminal)
  → opens the themed web UI in the browser
  → UI connects over WebSocket to /ws/console on the local airframe engine
  → user picks a model (model chooser), chats, uses agentic tools
```

Free tier ships the **arcade** theme. Paid themes gated by license (URL TBD).

---

## What `shimmy console` IS NOT

It is **NOT** `airframe/crates/console/`. That airframe crate is a separate
**developer terminal tool** (chat REPL + tools). Same word, different product.
Do not use it as shimmy console. Do not build the browser product on top of it.

---

## Where Things Live

| Piece | Location | State |
|---|---|---|
| `/ws/console` WebSocket handler | `shimmy/src/api.rs` | ✅ committed |
| `/discover` endpoint | `shimmy/src/api.rs` | ✅ committed |
| `shimmy console [theme]` CLI command | `shimmy/src/cli.rs` | ⬜ to be wired |
| Embedded server spawn helpers | `shimmy/console/embedded_server.rs` | ✅ `find_free_port()`, `wait_for_ready()` |
| Arcade theme frontend | `C:\Users\micha\repos\arcade` (GitHub: `Michael-A-Kuykendall/arcade`) | ✅ renamed, scrubbed, model chooser exists |
| Local working copy of arcade | `shimmy/console/themes/arcade/` | gitignored, reference only |
| `#[cfg(feature="console")]` stubs | `shimmy/src/main.rs`, `cli.rs` | preserved — integration point |

---

## What's Left (next session)

1. Wire `Command::Console { theme }` in `cli.rs` — spawn serve, open browser
2. Build the theme chooser screen in arcade (extend existing styles; model chooser already exists)
3. Model discovery "Add folder" in UI → writes `~/.shimmy/config.toml` `model_dirs`
4. License gate stub (print URL, exit)
5. Test end-to-end with TinyLlama

See `docs/internal/CONSOLE_WORKSTREAM_2026-06-10.md` for the full plan and the
two-terminal prototype command to verify the stack works today.

---

## Theme

Default theme: **`arcade`** (was `amiga-ai-interface`, renamed everywhere, no "amiga" remains).

---

## Patent Notice

This software implements Fused Semantic Execution (FSE).  
Patent pending by Michael A. Kuykendall. All rights reserved.
