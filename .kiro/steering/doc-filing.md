# Document Filing Convention

## Two-Directory Rule

Every markdown document belongs in exactly one of two places:

| Directory | Purpose | Goes in git? |
|-----------|---------|--------------|
| `docs/` | Public-facing user documentation | ✅ Yes |
| `docs/internal/` | Dev process, AI sessions, audits, plans | ❌ No (gitignored) |

## What goes in `docs/` (public)

- User-facing guides: README, quickstart, API reference, CONFIGURATION, EXAMPLES
- Model documentation: CHAT_TEMPLATES, QUANTIZATION, TROUBLESHOOTING
- Release notes: `docs/releases/`
- Language translations: `docs/zh-CN/`, `docs/zh-TW/`
- Architecture docs intended for external contributors

**Test:** Would a user or external contributor need this to use or contribute to the project? → `docs/`

## What goes in `docs/internal/` (private, never committed)

- AI session handoffs and handoff briefs
- Archaeology/recovery/reconstruction plans
- Corruption audits, session forensic notes
- Execution plans, roadmaps, checklists (development process)
- Clippy remediation plans, refactor plans
- Test plans for specific hardware (A100, RTX 3060)
- Regression workstreams, model-specific debugging notes
- Any file named `HANDOFF_*`, `*_AUDIT*`, `*_RECONSTRUCTION*`, `*_SPIKE*`
- Whitepapers that expose internal architecture decisions before public release

**Test:** Does this file contain internal process, AI-generated session context, or architectural decisions not yet public? → `docs/internal/`

## Gitignore entries (already present)

Both repos have `docs/internal/` in `.gitignore`. Never remove this entry.

```
# Internal developer docs — never committed
docs/internal/
```

## Quick filing checklist

Before creating or saving any markdown document:

```
[ ] Is this for a user or contributor? → docs/
[ ] Is this an AI handoff, session note, execution plan, or audit? → docs/internal/
[ ] Does it contain internal architecture not yet public? → docs/internal/
[ ] Is it a whitepaper or roadmap that might need public release later? → docs/internal/ for now, promote to docs/ when ready
```

## Airframe repo: same convention

`c:\Users\micha\repos\airframe\docs\` → public architecture docs  
`c:\Users\micha\repos\airframe\docs\internal\` → internal (gitignored)  
`c:\Users\micha\repos\airframe\artifacts\` → AI chat artifacts, offload docs (gitignored)
