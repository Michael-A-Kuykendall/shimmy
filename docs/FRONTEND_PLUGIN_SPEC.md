# Shimmy Frontend Plugin Specification v1.0

**Last Updated:** October 23, 2025  
**Status:** DRAFT - Foundation for modular frontend system

---

## 🎯 Philosophy

**Design Freedom + Zero Backend Knowledge**

Any designer/developer should be able to build a Shimmy frontend using:
- **Their preferred tools** (React/Lovable, Tauri, TUI libraries, game engines, etc.)
- **Any design language** (retro, cyberpunk, modern, minimal, etc.)
- **Zero Rust knowledge** (consume JSON APIs only)

The frontend should be **100% independent** - drop it in, it connects, it works.

---

## 📡 Connection Spec

### Required Backend Endpoints

Your frontend needs to consume these **3 endpoints** (that's it):

#### 1. WebSocket: Real-time Chat
```
ws://localhost:11435/ws/console
```

**Protocol:**
- **Send:** Plain text user messages
- **Receive:** Streaming AI tokens (one token per frame)
- **Final Frame:** `{"done": true}`

**Example Flow:**
```javascript
// Connect
const ws = new WebSocket('ws://localhost:11435/ws/console');

// Send user message
ws.send("What's the weather?");

// Receive streaming tokens
ws.onmessage = (event) => {
  const data = event.data;
  if (data === '{"done":true}') {
    console.log('Generation complete');
  } else {
    // Append token to display
    appendToChat(data);
  }
};
```

#### 2. HTTP: System Metrics
```
GET http://localhost:11435/api/metrics
Content-Type: application/json
```

**Response Schema:**
```json
{
  "cpu_usage_percent": 45.2,
  "memory_used_bytes": 8589934592,
  "memory_total_bytes": 34359738368,
  "memory_usage_percent": 25.0,
  "tokens_per_second": 24.3,
  "current_session_tokens": 1247,
  "uptime_seconds": 3600,
  "requests_total": 42,
  "generation_errors": 0
}
```

**Polling:** Update every 500ms - 2s (your choice)

#### 3. HTTP: Model Info
```
GET http://localhost:11435/api/models
Content-Type: application/json
```

**Response Schema:**
```json
{
  "models": [
    {
      "name": "Phi-3-mini-4k-instruct",
      "size_bytes": 2400000000,
      "parameter_count": "3.8B",
      "source": "registered"
    }
  ]
}
```

---

## 🎨 Required UI Components

Your design MUST include these 5 elements (style them however you want):

### 1. Chat Display
- **Purpose:** Show conversation history
- **Data:** Message array with `{role, content, timestamp}`
- **Behavior:** Auto-scroll, word wrap, support markdown

### 2. System Gauges
- **Purpose:** Visualize system health
- **Data:** `cpu_usage_percent`, `memory_usage_percent`, `tokens_per_second`
- **Behavior:** Real-time updates, visual indicators (gauges/bars/graphs)

### 3. Input Field
- **Purpose:** User message entry
- **Behavior:** Text input + submit button, Enter key support

### 4. Model Display
- **Purpose:** Show current model name and status
- **Data:** `models[0].name`, `models[0].parameter_count`
- **Behavior:** Static display or dropdown selector

### 5. Connection Indicator
- **Purpose:** WebSocket connection status
- **States:** Connected (green), Disconnected (red), Reconnecting (yellow)
- **Behavior:** Auto-reconnect on failure

---

## 🎭 Design Freedom

**You control 100%:**

### Visual Style
- ✅ Colors (retro 32bit palette, cyberpunk neon, modern minimalist)
- ✅ Typography (8x8 bitmap font, monospace, sans-serif, custom)
- ✅ Animations (none, smooth transitions, retro chunky pixels)
- ✅ Layout (grid, flex, fixed, responsive)

### Technology Stack
- ✅ **Web:** React, Vue, Svelte, vanilla JS
- ✅ **Desktop:** Tauri, Electron, native frameworks
- ✅ **Terminal:** ratatui, crossterm, blessed
- ✅ **Game Engine:** Bevy, Unity, Godot (yes, really!)

### Example Themes

**Retro 32bit (320×256 indexed color):**
- 8x8 bitmap font, authentic AGA palette
- Chunky pixel beveled UI chrome
- Copper bar effects, authentic Workbench aesthetic

**Cyberpunk Terminal:**
- Neon green text on black
- Scanline effects, glitch animations
- ASCII art borders, matrix-style scrolling

**Modern Clean:**
- Tailwind UI components
- Smooth glassmorphism effects
- Responsive grid layout

---

## 🔌 Integration Examples

### Example 1: React + Lovable.dev

**You can build in Lovable and send them this:**

```typescript
// shimmy-frontend/src/api/shimmy.ts
export interface SystemMetrics {
  cpu_usage_percent: number;
  memory_usage_percent: number;
  tokens_per_second: number;
  current_session_tokens: number;
}

export async function getMetrics(): Promise<SystemMetrics> {
  const response = await fetch('http://localhost:11435/api/metrics');
  return response.json();
}

export function connectChat(
  onToken: (token: string) => void,
  onComplete: () => void
): WebSocket {
  const ws = new WebSocket('ws://localhost:11435/ws/console');
  
  ws.onmessage = (event) => {
    const data = event.data;
    if (data === '{"done":true}') {
      onComplete();
    } else {
      onToken(data);
    }
  };
  
  return ws;
}

export function sendMessage(ws: WebSocket, message: string) {
  ws.send(message);
}
```

**Design in Lovable:**
- Create beautiful React components
- Wire up the 3 functions above
- Export and run alongside shimmy server

### Example 2: Tauri Desktop App

```rust
// tauri-frontend/src-tauri/src/main.rs
#[tauri::command]
async fn get_metrics() -> Result<SystemMetrics, String> {
    let response = reqwest::get("http://localhost:11435/api/metrics")
        .await
        .map_err(|e| e.to_string())?;
    let metrics: SystemMetrics = response.json()
        .await
        .map_err(|e| e.to_string())?;
    Ok(metrics)
}

// Frontend (React/Vue/Svelte in Tauri webview)
// Same as Example 1, just bundled as desktop app
```

### Example 3: TUI (Terminal)

```rust
// ratatui-frontend/src/main.rs
use ratatui::prelude::*;
use tokio_tungstenite::connect_async;

async fn run_tui() -> Result<()> {
    // Connect WebSocket
    let (ws, _) = connect_async("ws://localhost:11435/ws/console").await?;
    
    // Poll metrics
    let metrics: SystemMetrics = reqwest::get("http://localhost:11435/api/metrics")
        .await?
        .json()
        .await?;
    
    // Render with ratatui (your design)
    render_chat_ui(&messages, &metrics)?;
    
    Ok(())
}
```

---

## 📦 Distribution Models

### Standalone Binary
- **Use Case:** Users run frontend separately from shimmy
- **Example:** `./32bit-frontend` connects to `shimmy serve`
- **Pros:** Total independence, any tech stack

### Shimmy Feature Flag
- **Use Case:** `cargo build --features cyberpunk-tui`
- **Example:** Single binary with embedded frontend
- **Pros:** One-command launch, better UX

### Web Dashboard
- **Use Case:** Host static files in shimmy's HTTP server
- **Example:** `http://localhost:11435/ui/32bit.html`
- **Pros:** Zero install, browser-based

---

## 🚀 Quickstart: Build Your Own Frontend

### Step 1: Clone Template
```bash
git clone https://github.com/shimmy/frontend-template
cd frontend-template
```

### Step 2: Customize Design
- Edit `src/theme.css` (colors, fonts, spacing)
- Modify `src/components/Chat.tsx` (layout)
- Style `src/components/Gauges.tsx` (visualization)

### Step 3: Test
```bash
# Start shimmy backend
shimmy serve --bind localhost:11435

# Start your frontend (example: React dev server)
npm run dev
```

### Step 4: Ship
```bash
# Build static bundle
npm run build

# Distribute as:
# - Standalone web app (deploy anywhere)
# - Tauri desktop app (bundle with backend)
# - Embedded in shimmy (contribute via PR)
```

---

## 🔒 Backend Guarantees

**Shimmy backend promises:**

1. **Stable API:** These 3 endpoints won't break between versions
2. **CORS Enabled:** Web frontends work out-of-box
3. **Auto-discovery:** Backend finds open ports automatically
4. **Graceful Failures:** 503 Service Unavailable when model loading
5. **WebSocket Keepalive:** Connection stays open indefinitely

**What backend does NOT provide:**
- ❌ Authentication (local-only, trusted environment)
- ❌ Multi-user sessions (single-session model)
- ❌ Frontend hosting (bring your own or use embedded option)

---

## 📋 Validation Checklist

Before shipping your frontend, verify:

- [ ] **WebSocket Connection:** Connects to `ws://localhost:11435/ws/console`
- [ ] **Metrics Polling:** Fetches `GET /api/metrics` every 1-2 seconds
- [ ] **Chat Display:** Shows streaming tokens in real-time
- [ ] **System Gauges:** Visualizes CPU/RAM/tokens-per-second
- [ ] **Error Handling:** Shows friendly message when backend offline
- [ ] **Reconnection:** Auto-reconnects WebSocket on failure
- [ ] **Model Display:** Shows current model name from `/api/models`

---

## 🎓 Reference Implementations

**Study these for examples:**

1. **64bit 32bit Frontend** (`examples/aga_64bit.rs`)
   - Tech: Rust + winit + pixels
   - Style: Retro AGA chunky pixels, 8x8 bitmap font
   - Layout: Dual-panel (chat + metrics)

2. **Cyberpunk TUI** (`src/frontend/cyberpunk.rs`)
   - Tech: Rust + ratatui + crossterm
   - Style: Neon terminal, ASCII borders
   - Layout: Split-pane terminal UI

3. **Modern Web Template** (TODO: Community contribution)
   - Tech: React + Tailwind + Vite
   - Style: Clean, responsive, glassmorphism
   - Layout: Centered chat with side metrics panel

---

## 🤝 Contributing Your Frontend

Want your frontend included in shimmy?

1. **Build it** following this spec
2. **Test it** with checklist above
3. **Document it** (README with screenshots)
4. **PR it** to `shimmy/frontends/your-theme/`

**Requirements for inclusion:**
- Follows this spec (3 endpoints, 5 components)
- Has clear README with setup instructions
- Includes at least 2 screenshots
- Works with latest shimmy backend (test with `shimmy serve`)

---

## 📞 Support

- **Questions:** Open GitHub Discussion
- **Bugs:** File GitHub Issue with "frontend:" prefix
- **Design Ideas:** Share in Discord #frontend-showcase

---

**TL;DR:** Consume 3 JSON endpoints, render 5 UI components however you want. That's the entire contract.
