# Why Shimmy Exists: The Developer's LoRA Problem

## 🎯 The Core Problem Shimmy Solves

**You just trained a LoRA adapter. Now what?**

```bash
# You have this:
my-awesome-coding-lora/
├── adapter_model.safetensors
├── adapter_config.json
└── training_results.json

# You want this:
curl -X POST http://localhost:11435/api/generate \
  -d '{"model":"phi3-lora","prompt":"def fibonacci(n):"}'
```

**The frustration:** Converting SafeTensors → GGUF → Setting up inference is a pain.

**The solution:** Shimmy does it all in one step.

## 🚀 Zero-Friction LoRA Inference

### Before Shimmy (The Hard Way)
```bash
# 1. Convert SafeTensors to GGUF (find llama.cpp scripts)
python convert-lora-to-ggml.py adapter_model.safetensors adapter.gguf

# 2. Set up llama.cpp server
./llama-server --model base.gguf --lora adapter.gguf --port 8080

# 3. Figure out API format
# 4. Debug context settings
# 5. Handle streaming properly
```

### With Shimmy (The Easy Way)
```bash
# 1. Point Shimmy at your training output
export SHIMMY_BASE_GGUF=./models/phi3-mini.gguf
export SHIMMY_LORA_GGUF=./my-awesome-coding-lora/adapter_model.safetensors

# 2. Start serving
shimmy serve --bind 127.0.0.1:11435

# 3. Use it immediately
curl -X POST http://localhost:11435/api/generate \
  -d '{"model":"phi3-lora","prompt":"def fibonacci(n):","stream":true}'
```

## 🎯 Built for the LoRA Training Workflow

**Shimmy was born from real frustration:**
- Trained a LoRA with Unsloth/PEFT/Axolotl
- Had useful results in SafeTensors format
- Conversion workflows were painful and error-prone
- Just wanted fast local inference without hassle

**Result:** Shimmy bridges training → production with zero friction.

## 🔥 Perfect for Developer Use Cases

### Code Generation LoRAs
```bash
# Train a Python specialist
unsloth train --model phi3-mini --dataset python-code

# Serve immediately with Shimmy  
shimmy serve --auto-discover ./training-output/
```

### Framework Specialists
```bash
# Train React expert LoRA
python train_react_lora.py

# Deploy instantly
export SHIMMY_LORA_GGUF=./react-expert-lora/adapter_model.safetensors
shimmy serve
```

### API Documentation Helpers
```bash
# Fine-tune for API docs
axolotl train api-docs-config.yml

# Serve for team use
shimmy serve --bind 0.0.0.0:11435  # Team accessible
```

## 🛠️ Technical Advantages

### Smart Format Handling
- **Auto-detects:** SafeTensors vs GGUF LoRA adapters
- **Auto-converts:** SafeTensors → temporary GGUF when needed
- **Auto-discovers:** Paired base models and adapters

### Developer-Friendly APIs
- **HTTP JSON:** Standard REST API
- **Server-Sent Events:** Real-time streaming
- **WebSocket:** Interactive applications
- **CLI:** Quick testing and automation

### Zero-Config Operation
```bash
# Just point and shoot
shimmy serve --models-dir ./my-training-experiments/
# Automatically finds and serves all model+LoRA pairs
```

## 🎯 The Shimmy Promise

**"From LoRA training to production API in under 30 seconds."**

No conversion scripts. No config files. No infrastructure setup.

Just fast, local LoRA inference that works the way developers think.

---

*Shimmy: The missing piece between LoRA training and LoRA deployment.*
