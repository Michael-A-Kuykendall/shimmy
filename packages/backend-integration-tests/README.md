# @shimmy/theme-validator

Shimmy Theme Validator ensures themes adhere to the Frontend Contract (model discovery, chooser, selection, and basic chat).

Status: Phase 1 (model chooser + discovery + basic chat)

## Install & Run (from this package)

```bash
npm i
npm run build
node dist/index.js --theme-path ../../32bit-interface --timeout 15000
```

CLI Options:
- `--theme-path <path>`: Path to theme root (default: ../../32bit-interface)
- `--timeout <ms>`: Per-network operation timeout (default: 10000)
- `--ports <csv>`: Port list to probe for shimmy (default: 11435,11434,11436,11430,11431,11432,11433)
- `--json <file>`: Write JSON report to file

Exit codes:
- 0 = All mandatory checks PASS
- 1 = A mandatory check FAILed or internal error

See THEME_VALIDATOR_SPEC.md at repo root for detailed rules.
