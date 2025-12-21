# @shimmy/comm (Internal Sidecar)

Internal, theme-agnostic communication channel for Shimmy final shakedown.

## Commands (stdin REPL)
- /discover        Find running shimmy via discovery ports
- /models          List available models
- /use <model>     Select model for subsequent prompts
- /say <text>      Stream a prompt (tokens stderr or tagged lines)
- /metrics         Snapshot metrics and deltas
- /history         Print session turns
- /persist on|off  Toggle disk transcript (.shimmy-comm-sessions)
- /check tool <t>  Non-destructive tool probe prompt
- /exit            Quit

No theme modifications required; relies on existing HTTP + WebSocket endpoints.
