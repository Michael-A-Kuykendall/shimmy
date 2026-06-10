# Shimmy Console — Where It Lives

**Consolidated:** 2026-06-10  
**Status:** Canonical implementation is in the airframe repo, NOT here.

---

## FULL WORKSTREAM DOC: `docs/internal/CONSOLE_WORKSTREAM_2026-06-10.md`

Read that first. It has everything.

## DO NOT BUILD A NEW CONSOLE HERE

The shimmy-console implementation lives in:

```
airframe/crates/console/     ← THIS IS THE ONE
branch: release/v0.2.2-clean
```

Compiles clean as of 2026-06-10. Zero errors, zero warnings.

---

## What Lives in THIS Repo (shimmy)

### `shimmy/console/` — Dead Skeleton
The `console/` workspace crate in this repo is a **dead skeleton** on the `feature/console` branch.
It has `embedded_server.rs` scaffolding and stub commands. Do not extend it.
It is kept only for git history. The real work is in airframe.

### `shimmy/src/` — Feature Stubs (Intentionally Preserved)
The `#[cfg(feature = "console")]` stubs in `src/main.rs` and `src/cli.rs` are
**intentionally kept** as the future integration point. When shimmy-console is ready
to ship as part of the public shimmy binary, these stubs get wired to the
`shimmy-console-lib` crate (published from airframe/crates/console).

Do not remove these stubs. Do not extend them here. They are placeholders only.

---

## Integration Plan (Future)

When console is ready to go public:
1. Publish `airframe/crates/console` as `shimmy-console-lib` on crates.io
2. Wire the `#[cfg(feature = "console")]` stubs in shimmy to call it
3. Ship as `shimmy console [theme]` subcommand

---

## Theme System

- Default theme: **`arcade`** (approved 2026-06-10, replaces any prior "amiga" references)
- Themes live in `~/.shimmy/themes/{name}/`
- User config: `~/.shimmy/config.toml`

---

## Prior Console Branches in This Repo

| Branch | What it was | Status |
|---|---|---|
| `feature/console` | Dead console skeleton + Phase 1 tools | Superseded — signposted |
| `shimmy-console` | Old recovery branch | Ancient — ignore |

---

## Patent Notice

This software implements Fused Semantic Execution (FSE).  
Patent pending by Michael A. Kuykendall. All rights reserved.
