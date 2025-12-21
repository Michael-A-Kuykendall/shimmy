# 🚀 Theme Shakedown Protocol - Quick Start Guide

## What Just Happened

You now have a **fully automated, end-to-end validation system** for Shimmy themes. The infrastructure successfully executed:

1. **Phase 0**: Theme loads → Backend discovered via HTTP → WebSocket connected
2. **Phase 1**: User clicks model button → Backend confirms selection → Navigates to chat
3. **Phase 2**: User types message → Sends via WebSocket → UI displays message

All in **under 2 minutes** with zero manual intervention.

---

## How to Validate Any Theme

### Option 1: Quick Single-Theme Test
```bash
cd /path/to/shimmy

# Start the stack with your theme
bash -c "./target/release/shimmy dev your-theme-name --verify --no-build || cargo run --release --bin shimmy -- dev your-theme-name --verify --no-build"

# Run the flow-tester
node theme-tester/flow-tester.js http://localhost:8080

# Results appear in theme-tester/screenshots/
```

### Option 2: Batch Multi-Theme Validation
Create `validate-themes.sh`:
```bash
#!/bin/bash
THEMES=("32bit" "amiga" "windows95" "macintosh" "commodore64")

for theme in "${THEMES[@]}"; do
  echo "🎨 Validating $theme..."
  
  # Reset stack with this theme
  # Start using the Rust orchestrator lifecycle instead of the legacy shell bootstrap
  bash -c "./target/release/shimmy dev \"$theme\" --verify --no-build || cargo run --release --bin shimmy -- dev \"$theme\" --verify --no-build" || { echo "❌ $theme failed to start"; continue; }
  
  # Run automated tests
  node theme-tester/flow-tester.js http://localhost:8080 || { echo "❌ $theme failed validation"; continue; }
  
  # Archive results
  mkdir -p validation-results/$theme
  cp theme-tester/screenshots/flow-*.{png,log} validation-results/$theme/
  
  echo "✅ $theme validated"
done
```

---

## Test Artifacts

After each run, check:

```
theme-tester/screenshots/
├── flow-phase0.png    ← Initial state (model selection)
├── flow-phase0.log    ← Discovery + WebSocket connection logs
├── flow-phase1.png    ← Chat interface (after model selection)
├── flow-phase1.log    ← Model selection confirmation
├── flow-phase2.png    ← Chat message visible
└── flow-phase2.log    ← Message transmission logs
```

### Reading the Logs

**Phase 0 Log** should show:
```
✅ Discovered Shimmy backend: http://127.0.0.1:XXXXX
✅ WebSocket connected
📨 WebSocket message received: models_response
```

**Phase 1 Log** should show:
```
✅ Model selected: [model-name]
✅ WebSocket connected (after navigation)
```

**Phase 2 Log** should show:
```
📤 Sending message: Hello
📨 Sending WebSocket message: {"type":"chat","message":"Hello"}
```

---

## What Gets Tested

| Aspect | What's Validated |
|--------|------------------|
| **UI Rendering** | Theme loads on port 8080, renders models |
| **Backend Discovery** | HTTP discovery endpoint works |
| **WebSocket Connection** | Persistent WS connection to backend |
| **Model Loading** | Models list received and displayed |
| **Navigation** | Route transitions work (/ → /chat) |
| **User Interaction** | Button clicks and form inputs work |
| **Message Transmission** | Chat messages sent successfully |
| **Console Health** | No JavaScript errors reported |

---

## Troubleshooting

### Theme Doesn't Start
```bash
tail -50 shimmy_startup.log  # Check backend
tail -50 theme_startup.log   # Check theme
```

### WebSocket Connection Fails
1. Check backend is running: `curl http://127.0.0.1:11430/api/discovery`
2. Check theme console logs in `flow-phase0.log`
3. Verify port 8080 is free: `netstat -ano | grep 8080`

### Model Selection Doesn't Navigate
1. Check `flow-phase1.log` for WebSocket disconnect messages
2. Verify backend responds to model selection
3. Check React Router configuration in theme

### Message Not Sent
1. Verify chat input selector in `flow-tester.js` matches theme
2. Check `flow-phase2.log` for WebSocket errors
3. Ensure backend is processing chat requests

---

## Customizing for Your Theme

Edit `flow-tester.js` to adapt for different UI structures:

```javascript
// Change model selection timing
await page.waitForTimeout(1500); // Increase if navigation slower

// Change chat input selector
await page.waitForSelector('input[placeholder*="message"]', { timeout: 5000 });
// → Update placeholder text to match your theme's input field

// Change send button logic
if (text?.toUpperCase().includes('SEND')) {
  // → Adapt for different button labels (Submit, Post, etc.)
}
```

---

## Performance Baseline

**32-bit Theme** (validated today):
- Phase 0 (Load + Discovery): ~2s
- Phase 1 (Model Selection): ~2s
- Phase 2 (Chat Message): ~3s
- **Total**: ~7s per theme

**For 10 themes**: ~70 seconds

---

## Next: Scale to 10+ Themes

The foundation is ready. To validate 10+ themes:

1. **Create test themes** (copy 32bit, modify CSS/colors)
2. **Run batch validator** with the script above
3. **Collect metrics** (load time, WebSocket stability)
4. **Generate reports** (success rate, failure types)
5. **CI/CD integration** (automate on every theme commit)

---

## Key Files

| File | Purpose |
|------|---------|
| `theme-tester/flow-tester.js` | Automated multi-phase validator |
| `theme-tester/tester.js` | Single-action Playwright wrapper |
| `analyze-screenshot.js` | OCR analysis of screenshots |
| `shimmy dev` | Stack initialization (backend + theme) |
| `SHAKEDOWN_VALIDATION_REPORT.md` | Detailed validation results |

---

## Success Criteria (For Any Theme)

✅ **All 3 phases complete without errors**
- Phase 0: Models visible, no console errors
- Phase 1: Chat interface appears, connection confirmed
- Phase 2: Message sent successfully

✅ **Console logs are clean**
- No `❌ Error:` messages
- `✅` confirmations for discovery, WebSocket, model selection, chat

✅ **Screenshots show expected UI**
- Phase 0: Model grid visible
- Phase 1: Chat interface with CONNECTED status
- Phase 2: User message visible in chat

---

**You're ready to validate themes! 🚀**

Start with:
```bash
bash -c "./target/release/shimmy dev 32bit --verify --no-build || cargo run --release --bin shimmy -- dev 32bit --verify --no-build"
node theme-tester/flow-tester.js http://localhost:8080
```
