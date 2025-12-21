# @shimmy/theme-validator

Static analysis CLI to validate Shimmy theme source code against the frozen Frontend Contract.

- No runtime/backend calls — pure source inspection
- Checks: discovery usage, no hardcoded ports, ModelChooser + Chat presence, selection→chat wiring, metrics hooks

Usage
- Build: npm install && npm run build
- Run: npx shimmy-validate-theme --theme-path ../../32bit-interface

Options
- --theme-path <dir>   Required path to the theme root
- --json <file>        Optional report output path
- --strict             Treat warnings as failures (non-zero exit)
