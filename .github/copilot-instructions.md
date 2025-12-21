# Shimmy AI Assistant Instructions

## CRITICAL: Terminal Rules
**NEVER pipe cat to head/tail - hangs forever on Windows/Git Bash.**
```bash
# WRONG: cat file | head -100
# RIGHT: head -100 file
```

## Execution Gate
User controls all runtime. Do not start servers, orchestrators, or long builds unless explicitly requested. Only execute stop-all/build/start when the user asks.

## Stack Commands
The assistant does not run these unless the user requests.
```bash
cargo build --release --bin shimmy --features llama,console,http-adapter
./target/release/shimmy dev shimmy-default  # starts backend + theme
./target/release/shimmy stop-all
```

## Ports
- 11430: Discovery (fixed)
- 8080: Theme dev
- Ephemeral: Backend API

## Theme Testing (Playwright)
```bash
node theme-tester/tester.js screenshot http://localhost:8080 test.png
node theme-tester/tester.js click http://localhost:8080 '<selector>' after.png
```

## Policy
- Do not run services or long builds proactively. The user will request stop-all, compile, and start.
- Make focused code changes and prepare for testing; then notify the user when ready.
- Align to existing specs; avoid protocol invention unless gaps are confirmed.
