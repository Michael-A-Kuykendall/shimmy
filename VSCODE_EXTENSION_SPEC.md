# Shimmy VS Code Extension Specification

## 🎯 Extension Overview

**Name:** Shimmy - Instant LoRA Inference  
**ID:** `shimmy.lora-inference`  
**Category:** AI, Machine Learning, Developer Tools

## 🔥 Core Value Proposition

**"Serve your trained LoRA adapters instantly without leaving VS Code"**

Target workflow:
1. Developer trains LoRA with Unsloth/PEFT/Axolotl in VS Code
2. Gets `adapter_model.safetensors` in workspace  
3. Right-clicks file → "Serve with Shimmy"
4. Immediately test custom model via HTTP API

## ✨ Key Features

### 1. 🔍 **Auto-Discovery**
- Scan workspace for LoRA adapter files
- Detect paired base models automatically
- Show model compatibility in explorer

### 2. 🚀 **One-Click Serving**
```typescript
// Context menu on .safetensors/.gguf files
- "Serve with Shimmy"
- "Stop Shimmy Server" 
- "Test Model"
- "Copy API Endpoint"
```

### 3. 📊 **Status Integration**
```typescript
// Status bar items
- "Shimmy: phi3-lora (localhost:11435)" [Active]
- "Shimmy: Stopped" [Click to start]
```

### 4. 🔧 **Settings Integration**
```json
{
  "shimmy.defaultPort": 11435,
  "shimmy.modelsDirectory": "./models",
  "shimmy.autoStart": false,
  "shimmy.showStatusBar": true
}
```

### 5. 🎯 **Command Palette**
```typescript
// Available commands
- "Shimmy: Start Server"
- "Shimmy: Stop Server"  
- "Shimmy: Restart Server"
- "Shimmy: Show Models"
- "Shimmy: Test Generation"
```

## 🛠️ Technical Implementation

### Extension Structure
```
shimmy-vscode/
├── package.json          # Extension manifest
├── src/
│   ├── extension.ts      # Main extension entry
│   ├── shimmy-manager.ts # Subprocess management  
│   ├── model-explorer.ts # File discovery
│   └── status-bar.ts     # UI integration
├── resources/            # Icons, etc.
└── README.md            # Extension documentation
```

### Key APIs Used
```typescript
// VS Code Extension APIs
- vscode.workspace.findFiles() // Find LoRA files
- vscode.window.createTerminal() // Run shimmy commands
- vscode.StatusBarItem // Show server status
- vscode.commands.registerCommand() // Register commands
- vscode.FileSystemWatcher // Watch for new models
```

### Shimmy Integration
```typescript
// Spawn shimmy as subprocess
const shimmy = spawn('shimmy', [
  'serve',
  '--bind', '127.0.0.1:11435',
  '--models-dir', workspace.modelsDir
]);

// Monitor process health
shimmy.on('exit', (code) => {
  statusBar.update('Stopped');
});
```

## 📦 Installation & Setup

### Prerequisites Detection
```typescript
// Check if shimmy is installed
async function checkShimmyInstalled(): Promise<boolean> {
  try {
    const result = await exec('shimmy --version');
    return result.exitCode === 0;
  } catch {
    return false;
  }
}

// Offer installation if missing
if (!await checkShimmyInstalled()) {
  const action = await vscode.window.showInformationMessage(
    'Shimmy not found. Install now?',
    'Install via Cargo',
    'Manual Install'
  );
  
  if (action === 'Install via Cargo') {
    terminal.sendText('cargo install shimmy --features llama');
  }
}
```

## 🎯 User Experience Flow

### 1. **First Time Setup**
```
1. Install extension from marketplace
2. Extension detects if shimmy binary exists
3. If not, shows install guidance
4. Scans workspace for models
5. Shows available models in explorer
```

### 2. **Daily Workflow**  
```
1. Train LoRA in Jupyter/Python notebook
2. See new adapter_model.safetensors in explorer
3. Right-click → "Serve with Shimmy"
4. Status bar shows "Shimmy: my-lora (localhost:11435)"
5. Test immediately with curl/HTTP requests
```

### 3. **Model Management**
```
1. Explorer view shows all discovered models
2. Click to start/stop specific models
3. Switch between adapters easily
4. View server logs in output panel
```

## 📊 Competitive Analysis

**Similar Extensions:**
- **Continue.dev** (1.5M installs) - Local AI coding assistant
- **Ollama** extensions - Local model management
- **GitHub Copilot** - AI code completion

**Shimmy's Unique Value:**
- ✅ **LoRA-specific** - designed for training workflow
- ✅ **Zero-config** - auto-discovery and setup
- ✅ **Production-ready** - proper HTTP APIs, not just chat
- ✅ **Performance-focused** - GGUF optimization

## 🚀 Development Timeline

### Phase 1: Core Extension (1-2 weeks)
- [ ] Basic file discovery
- [ ] Shimmy subprocess management  
- [ ] Status bar integration
- [ ] Context menu actions

### Phase 2: Enhanced UX (1 week)
- [ ] Settings integration
- [ ] Command palette commands
- [ ] Better error handling
- [ ] Installation guidance

### Phase 3: Advanced Features (Future)
- [ ] Model explorer tree view
- [ ] Built-in API testing
- [ ] Performance monitoring
- [ ] VS Code Language Model API integration

## 📈 Success Metrics

**Adoption Targets:**
- Month 1: 1,000 installs 
- Month 3: 5,000 installs
- Month 6: 15,000 installs

**Usage Indicators:**
- Models served per day
- API requests generated  
- User retention rate
- GitHub stars/issues

## 🎯 Marketing Strategy

**Discovery Channels:**
- VS Code Marketplace AI category
- Machine Learning subreddits
- Unsloth/PEFT community channels
- HuggingFace forums
- ML Twitter/LinkedIn

**Key Messages:**
- "Train locally, serve instantly"
- "Zero-config LoRA deployment"
- "From notebook to API in 30 seconds"

The VS Code extension represents the **highest-impact distribution channel** because it integrates directly into the ML developer workflow where LoRA training happens.
