<div align="center">

# Shimmy 使用者手冊

### 輕量本地 AI 推論伺服器，相容 OpenAI API

[简体中文](USER_MANUAL.zh-CN.md) · **繁體中文** · [English](../README.md)

版本：v2.0.0 及以上 · 最後更新：2026 年 5 月

</div>

---

## 目錄

1. [Shimmy 簡介](#1-shimmy-簡介)
2. [Airframe 引擎](#2-airframe-引擎)
3. [系統需求](#3-系統需求)
4. [安裝](#4-安裝)
5. [快速入門（30 秒）](#5-快速入門30-秒)
6. [取得模型檔案](#6-取得模型檔案)
7. [環境變數設定](#7-環境變數設定)
8. [命令列參考](#8-命令列參考)
9. [OpenAI 相容 API](#9-openai-相容-api)
10. [WebSocket 串流 API](#10-websocket-串流-api)
11. [模型自動探索](#11-模型自動探索)
12. [延伸上下文視窗（YaRN RoPE）](#12-延伸上下文視窗yarn-rope)
13. [連接開發工具](#13-連接開發工具)
14. [使用 SDK 呼叫](#14-使用-sdk-呼叫)
15. [效能參考](#15-效能參考)
16. [常見問題排解](#16-常見問題排解)
17. [從 v1.x 遷移](#17-從-v1x-遷移)
18. [從原始碼建置](#18-從原始碼建置)
19. [贊助支持](#19-贊助支持)

---

## 1. Shimmy 簡介

Shimmy 是一個**單一執行檔**的本地 AI 推論伺服器，提供與 OpenAI API **100% 相容**的 HTTP 介面，專為在本地端執行大型語言模型（LLM）而設計。

### 核心優勢

| 特性 | 說明 |
|------|------|
| 🔒 **完全本地** | 所有推論在您的裝置上進行，資料不會上傳至任何雲端 |
| 🆓 **永久免費** | 無訂閱費，無按 token 計費，無任何隱藏收費 |
| ⚡ **零設定** | 無需設定檔，自動偵測模型與 GPU |
| 🔌 **即插即用** | 現有 OpenAI 用戶端程式碼**無需修改**，僅替換 API 位址即可 |
| 🦀 **純 Rust 實作** | 單一可執行檔，無 Python 執行環境，無 C++ 相依套件 |
| 🖥️ **跨平台 GPU 加速** | 透過 WebGPU（wgpu）支援 NVIDIA、AMD、Intel 及 Apple Silicon |

### v2.0.0 新功能

- **Airframe 引擎**：全新純 Rust WGSL 推論引擎，完全取代 llama.cpp
- **WebGPU 加速**：透過 wgpu 自動選擇最佳 GPU 適配器
- **YaRN RoPE 縮放**：支援超長上下文視窗（最高 16384+ tokens）
- **零工具鏈相依**：無需 CUDA 工具包、Vulkan SDK 或任何 C++ 編譯器

---

## 2. Airframe 引擎

自 v2.0.0 起，Shimmy 的預設推論引擎為 **Airframe**——一個從零開始以純 Rust 撰寫的 WebGPU（WGSL）變換器執行環境。

### 技術架構

```
您的請求
    │
    ▼
Shimmy HTTP 伺服器（Axum，純 Rust）
    │
    ▼
Airframe 推論引擎
    │
    ├─ WGSL 計算著色器（由 wgpu 在執行時期編譯）
    ├─ F32 全精度運算
    ├─ YaRN RoPE 延伸（長上下文）
    └─ GPU 自動選擇（wgpu 列舉最佳適配器）
         │
         ├─ NVIDIA → Vulkan / Direct3D 12
         ├─ AMD   → Vulkan / Direct3D 12
         ├─ Intel → Vulkan / Direct3D 12
         └─ Apple → Metal
```

### 為何選擇 Airframe？

- **無 C++ 工具鏈**：從頭到尾全部使用 Rust，無需安裝 CUDA、Vulkan SDK 等
- **F32 全精度**：確定性輸出，高品質推論結果
- **WGSL 著色器**：透過 WebGPU 在任何 GPU 上執行
- **從 GGUF 元資料自動推導模型規格**：無需硬式編碼每個模型的參數
- **GPU 自動偵測**：執行時期自動列舉並選擇最佳 GPU 適配器

---

## 3. 系統需求

### 最低需求

| 元件 | 最低需求 |
|------|---------|
| 作業系統 | Windows 10/11、Linux（x86_64/ARM64）、macOS 12+ |
| CPU | x86_64 或 ARM64（Apple Silicon） |
| 記憶體 | 8GB RAM（建議 16GB 以上） |
| 儲存空間 | 5GB 以上（用於存放模型檔案） |
| 顯示卡 | 可選，但強烈建議（支援 Vulkan、Direct3D 12 或 Metal 的 GPU） |

### 建議配置

| 用途 | 顯示卡 | 顯示記憶體 | 記憶體 |
|------|--------|----------|--------|
| 小型模型（1B-3B） | 任意 GPU | 4GB | 8GB |
| 中型模型（7B-13B） | RTX 3060 / RX 6700 | 8-12GB | 16GB |
| 大型模型（30B+） | RTX 4090 / A100 | 24GB+ | 32GB+ |
| Apple Silicon | M1/M2/M3/M4 | 統一記憶體 16GB+ | — |

> **提示**：無 GPU 時，Shimmy 自動回退至 CPU 推論，速度較慢但完全可用。

---

## 4. 安裝

### 方式一：下載預先編譯的執行檔（建議）

預先編譯版本內建 Airframe WebGPU 引擎，**開箱即用，無需任何相依套件**。

#### Windows（x86_64）

```powershell
# 使用 curl（Windows 10 及以上自帶）
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# 或使用 PowerShell
Invoke-WebRequest -Uri "https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe" -OutFile "shimmy.exe"
```

#### Linux（x86_64）

```bash
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy
chmod +x shimmy
```

#### macOS Apple Silicon（M1/M2/M3/M4）

```bash
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy
chmod +x shimmy
```

#### macOS Intel

```bash
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-intel -o shimmy
chmod +x shimmy
```

#### Linux（ARM64，如 Raspberry Pi 5、AWS Graviton）

```bash
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-aarch64 -o shimmy
chmod +x shimmy
```

> **ARM64 注意**：Linux ARM64 版本使用 HuggingFace 引擎。Airframe 的 ARM64 交叉編譯支援正在開發中。

### 方式二：透過 cargo 安裝

```bash
# 從 crates.io 安裝（使用 HuggingFace 引擎，無需 GPU 即可執行）
cargo install shimmy
```

### 方式三：從原始碼建置（含 Airframe 引擎）

```bash
git clone https://github.com/Michael-A-Kuykendall/shimmy --recurse-submodules
cd shimmy
cargo build --release --features airframe,huggingface

# 建置完成後，執行檔位於：
./target/release/shimmy
```

---

## 5. 快速入門（30 秒）

```bash
# 第 1 步：設定模型路徑並啟動伺服器
SHIMMY_BASE_GGUF=/path/to/your-model.gguf ./shimmy serve

# Windows 使用者：
set SHIMMY_BASE_GGUF=C:\models\your-model.gguf && shimmy.exe serve

# 第 2 步：查看已註冊的模型
./shimmy list

# 第 3 步：測試 API（使用 curl）
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "your-model-name",
    "messages": [{"role": "user", "content": "你好，請用五個字回答。"}],
    "max_tokens": 32
  }' | jq -r '.choices[0].message.content'
```

伺服器啟動後，Shimmy 監聽於 `http://127.0.0.1:11435`。

---

## 6. 取得模型檔案

Shimmy 使用 **GGUF 格式**的模型檔案，這是目前最通用的量化模型格式。

### 建議模型

以下模型經過測試，可與 Shimmy 搭配使用：

| 模型 | 大小 | 顯示記憶體 | 下載位址 |
|------|------|----------|---------|
| Phi-3-mini-4k（4bit 量化） | ~2GB | 3GB | [HuggingFace](https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf) |
| Llama-3.2-1B-Instruct | ~0.8GB | 2GB | [HuggingFace](https://huggingface.co/meta-llama/Llama-3.2-1B-Instruct-GGUF) |
| Llama-3.2-3B-Instruct | ~2GB | 3GB | [HuggingFace](https://huggingface.co/meta-llama/Llama-3.2-3B-Instruct-GGUF) |
| Mistral-7B-Instruct（Q4） | ~4GB | 5GB | [HuggingFace](https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF) |
| Qwen2.5-7B-Instruct（Q4） | ~4GB | 5GB | [HuggingFace](https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF) |
| DeepSeek-R1-1.5B | ~1GB | 2GB | [HuggingFace](https://huggingface.co/unsloth/DeepSeek-R1-Distill-Qwen-1.5B-GGUF) |

### 使用 huggingface-cli 下載

```bash
# 安裝 huggingface_hub
pip install huggingface_hub

# 下載範例：Phi-3-mini
huggingface-cli download microsoft/Phi-3-mini-4k-instruct-gguf \
  Phi-3-mini-4k-instruct-q4.gguf \
  --local-dir ./models/
```

### 量化等級說明

GGUF 檔名中的量化後綴含義：

| 後綴 | 精度 | 記憶體佔用 | 品質 | 適用情境 |
|------|------|----------|------|---------|
| `Q4_0` | 4-bit | 最小 | 好 | 日常使用，記憶體受限 |
| `Q4_K_M` | 4-bit（改良） | 小 | 較好 | 建議的平衡選項 |
| `Q5_K_M` | 5-bit | 中等 | 很好 | 品質與速度均衡 |
| `Q6_K` | 6-bit | 較大 | 極好 | 高品質推論 |
| `Q8_0` | 8-bit | 大 | 接近原始 | 最高品質 |
| `F16` | 16-bit | 最大 | 原始精度 | 僅限高顯示記憶體環境 |

---

## 7. 環境變數設定

### 必填變數

| 變數名稱 | 說明 | 範例 |
|---------|------|------|
| `SHIMMY_BASE_GGUF` | 模型檔案路徑（或使用 `--model-path` 參數） | `/models/phi3.gguf` |

### 選填變數

| 變數名稱 | 預設值 | 說明 |
|---------|--------|------|
| `SHIMMY_MAX_CTX` | 模型原生（通常 2048） | 最大上下文 token 數，超過 2048 自動啟用 YaRN |
| `SHIMMY_ENGINE_BACKEND` | `airframe` | 推論引擎，設為 `airframe`（預設）或 `safetensors` |
| `SHIMMY_PORT` | `11435` | 伺服器監聽埠 |
| `SHIMMY_BIND_ADDRESS` | `127.0.0.1:11435` | 伺服器監聽位址 |
| `SHIMMY_LOG_LEVEL` | `warn` | 日誌等級：`error`/`warn`/`info`/`debug`/`trace` |
| `SHIMMY_MODEL_PATHS` | — | 額外模型目錄，多個路徑以 `;` 分隔 |
| `SHIMMY_LORA_GGUF` | — | LoRA 適配器檔案路徑 |
| `RUST_BACKTRACE` | — | 設為 `1` 開啟崩潰堆疊追蹤（除錯用） |
| `NO_COLOR` | — | 設為任意值停用彩色輸出 |

### 設定範例

```bash
# 最簡啟動（使用預設埠 11435）
SHIMMY_BASE_GGUF=/models/phi3-mini.gguf ./shimmy serve

# 指定埠與延伸上下文
SHIMMY_BASE_GGUF=/models/phi3-mini.gguf \
SHIMMY_MAX_CTX=8192 \
SHIMMY_PORT=8080 \
./shimmy serve

# 多目錄模型搜尋
SHIMMY_MODEL_PATHS="/data/models;/home/user/llm-models" \
./shimmy serve

# 啟用詳細日誌
SHIMMY_BASE_GGUF=/models/model.gguf \
SHIMMY_LOG_LEVEL=debug \
./shimmy serve

# 對外提供服務（繫結所有網路介面，請注意資安風險）
SHIMMY_BASE_GGUF=/models/model.gguf \
./shimmy serve --bind 0.0.0.0:11435
```

---

## 8. 命令列參考

### 全域選項

```
shimmy [全域選項] <子命令>

全域選項：
  --model-dirs <目錄清單>    額外的模型目錄（多個目錄用 ; 分隔）
  --gpu-backend <後端>       GPU 後端（auto/cpu）
  --cpu-moe                  將所有 MoE 專家層卸載至 CPU（節省顯示記憶體）
  --n-cpu-moe <N>            將前 N 層 MoE 專家卸載至 CPU
  --legacy                   使用 CPU 適配器（而非 Airframe GPU）
  -h, --help                 顯示說明資訊
  -V, --version              顯示版本號碼
```

### serve — 啟動 HTTP 伺服器

```bash
shimmy serve [選項]

選項：
  --bind <位址>          繫結位址（預設：auto，自動分配埠）
  --model-path <路徑>    模型檔案路徑（覆寫 SHIMMY_BASE_GGUF 環境變數）

範例：
  shimmy serve                                        # 使用預設設定
  shimmy serve --bind 127.0.0.1:11435                 # 指定埠
  shimmy serve --bind 0.0.0.0:8080                    # 對外服務
  shimmy serve --model-path /models/qwen2.5-7b.gguf   # 指定模型
  SHIMMY_MAX_CTX=4096 shimmy serve                    # 4K 上下文
```

### list — 列出已探索的模型

```bash
shimmy list

輸出範例：
  phi3-mini-4k        /home/user/.cache/huggingface/hub/.../phi3-mini.gguf
  mistral-7b-instruct /models/mistral-7b-instruct.Q4_K_M.gguf
  qwen2.5-7b          /data/models/qwen2.5-7b-instruct-q4_k_m.gguf
```

### generate — 命令列直接產生文字

```bash
shimmy generate [選項]

選項：
  --name <模型名稱>    使用的模型名稱
  --prompt <文字>      輸入提示詞
  --max-tokens <N>    最大產生 token 數（預設：256）
  --temperature <F>   採樣溫度（預設：0.7，範圍 0.0-2.0）

範例：
  shimmy generate --name phi3-mini --prompt "請解釋量子糾纏"
  shimmy generate --name mistral-7b --prompt "寫一首七言絕句" --max-tokens 100
  shimmy generate --name qwen2.5 --prompt "Hello" --temperature 0.3
```

### discover — 探索並列出所有可用模型路徑

```bash
shimmy discover

# 顯示 Shimmy 搜尋的所有目錄及找到的模型檔案
```

### gpu-info — 顯示 GPU 適配器資訊

```bash
shimmy gpu-info

輸出範例：
  Adapter 0: NVIDIA GeForce RTX 4090 (Vulkan)
  Adapter 1: Intel UHD Graphics 770 (Vulkan)
  Selected: Adapter 0 (discrete GPU preferred)
```

---

## 9. OpenAI 相容 API

Shimmy 在 `http://127.0.0.1:11435` 提供完整的 OpenAI 相容 API，**無需修改現有程式碼**，只需將 `api_base` 替換為 Shimmy 位址即可。

### 介面總覽

| 介面 | 方法 | 狀態 | 說明 |
|------|------|------|------|
| `/v1/chat/completions` | POST | ✅ 支援 | 對話補全，支援串流輸出 |
| `/v1/models` | GET | ✅ 支援 | 列出所有本地可用模型 |
| `/v1/models/:id` | GET | ✅ 支援 | 取得特定模型的元資料 |
| `/v1/completions` | POST | ✅ 支援 | 傳統文字補全介面 |
| `/api/generate` | POST | ✅ 支援 | Ollama 相容介面 |
| `/api/tags` | GET | ✅ 支援 | Ollama 相容介面 |
| `/api/health` | GET | ✅ 支援 | 健康檢查 |
| `/metrics` | GET | ✅ 支援 | Prometheus 指標 |
| `/v1/embeddings` | POST | ❌ 計劃中 | 向量嵌入（roadmap） |

### POST /v1/chat/completions

#### 請求內容

```json
{
  "model": "phi3-mini",
  "messages": [
    {
      "role": "system",
      "content": "你是一個專業的繁體中文助手，回答簡潔準確。"
    },
    {
      "role": "user",
      "content": "請解釋什麼是大型語言模型。"
    }
  ],
  "max_tokens": 512,
  "temperature": 0.7,
  "top_p": 0.9,
  "stream": false
}
```

#### 支援的請求欄位

| 欄位 | 型別 | 必填 | 說明 |
|------|------|------|------|
| `model` | string | 是 | 本地模型 ID 或別名 |
| `messages` | array | 是 | 對話歷史，每筆包含 `role` 和 `content` |
| `stream` | boolean | 否 | `true` 啟用 SSE 串流輸出，預設 `false` |
| `max_tokens` | integer | 否 | 最大產生 token 數 |
| `temperature` | float | 否 | 採樣溫度（0.0-2.0，預設 0.7） |
| `top_p` | float | 否 | nucleus 採樣（0.0-1.0，預設 0.9） |
| `top_k` | integer | 否 | Top-K 採樣（預設 40） |
| `stop` | string/array | 否 | 停止詞或停止詞陣列 |

#### 非串流回應

```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1716192000,
  "model": "phi3-mini",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "大型語言模型（LLM）是一種基於 Transformer 架構的神經網路..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 45,
    "completion_tokens": 128,
    "total_tokens": 173
  }
}
```

#### 串流回應（Server-Sent Events）

設定 `"stream": true` 後，伺服器以 SSE 格式逐 token 推送：

```
data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"role":"assistant","content":"大"},"finish_reason":null}]}

data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"型"},"finish_reason":null}]}

data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

#### 使用 curl 測試

```bash
# 非串流
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "phi3-mini",
    "messages": [
      {"role": "system", "content": "你是一個簡潔的繁體中文助手。"},
      {"role": "user", "content": "介紹一下台灣的科技產業。"}
    ],
    "max_tokens": 200
  }' | jq -r '.choices[0].message.content'

# 串流（加 -N 停用 curl 緩衝）
curl -N http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "phi3-mini",
    "stream": true,
    "messages": [{"role": "user", "content": "數數從 1 到 10"}]
  }'
```

### GET /v1/models

列出所有已註冊和自動探索的本地模型。

```bash
curl http://127.0.0.1:11435/v1/models | jq
```

```json
{
  "object": "list",
  "data": [
    {
      "id": "phi3-mini",
      "object": "model",
      "created": 1716192000,
      "owned_by": "local"
    },
    {
      "id": "qwen2.5-7b-instruct",
      "object": "model",
      "created": 1716192000,
      "owned_by": "local"
    }
  ]
}
```

### GET /api/health

健康檢查介面，可用於監控系統和負載平衡器探針。

```bash
curl http://127.0.0.1:11435/api/health
```

```json
{
  "status": "healthy",
  "models_loaded": 2,
  "version": "2.0.0"
}
```

---

## 10. WebSocket 串流 API

除 HTTP SSE 外，Shimmy 還提供 WebSocket 介面實現即時 token 推送。

### 連線位址

```
ws://127.0.0.1:11435/ws/generate
```

### 發送請求

連線後發送 JSON 訊息：

```json
{
  "model": "phi3-mini",
  "prompt": "請寫一首關於秋天的現代詩",
  "max_tokens": 200,
  "temperature": 0.8
}
```

### 接收回應

伺服器逐 token 推送：

```json
{"token": "秋"}
{"token": "風"}
{"token": "輕"}
{"token": "撫"}
...
{"done": true, "total_tokens": 87}
```

### JavaScript 範例

```javascript
const ws = new WebSocket('ws://127.0.0.1:11435/ws/generate');

ws.onopen = () => {
  ws.send(JSON.stringify({
    model: 'phi3-mini',
    prompt: '用繁體中文介紹人工智慧的發展歷史',
    max_tokens: 300,
    temperature: 0.7
  }));
};

let output = '';
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.done) {
    console.log('產生完成，共', data.total_tokens, '個 token');
    ws.close();
  } else {
    output += data.token;
    process.stdout.write(data.token); // 即時列印
  }
};
```

---

## 11. 模型自動探索

Shimmy 啟動時會自動掃描以下目錄尋找 GGUF 和 SafeTensors 模型檔案：

### 自動搜尋路徑

| 平台 | 自動搜尋路徑 |
|------|------------|
| 所有平台 | `./models/`（目前目錄下的 models 資料夾） |
| 所有平台 | `~/models/` |
| 所有平台 | `~/Downloads/`（含 .gguf 檔案） |
| Linux/macOS | `~/.cache/huggingface/hub/` |
| Windows | `%USERPROFILE%\.cache\huggingface\hub\` |
| Windows | `%LOCALAPPDATA%\Ollama\models\` |
| Linux | `~/.ollama/models/` |
| macOS | `~/Library/Application Support/Ollama/models/` |

### 新增自訂搜尋目錄

```bash
# 方式一：環境變數（多個路徑用 ; 分隔）
SHIMMY_MODEL_PATHS="/data/models;/mnt/nas/llm;/tmp/models" ./shimmy serve

# 方式二：命令列參數
./shimmy serve --model-dirs "/data/models;/mnt/nas/llm"

# 查看所有已探索的模型
./shimmy list

# 查看所有正在搜尋的目錄
./shimmy discover
```

### 模型命名規則

Shimmy 會依據檔名自動產生模型 ID：

- `/models/Qwen2.5-7B-Instruct-Q4_K_M.gguf` → ID 為 `qwen2.5-7b-instruct`
- `/models/phi-3-mini-4k-instruct-q4.gguf` → ID 為 `phi-3-mini-4k-instruct`
- `/models/deepseek-r1-1.5b.gguf` → ID 為 `deepseek-r1-1.5b`

---

## 12. 延伸上下文視窗（YaRN RoPE）

Shimmy v2.0 透過 Airframe 引擎內建 **YaRN RoPE 位置編碼縮放**，無需任何額外設定即可支援超出模型原生訓練上下文的輸入長度。

### 使用方式

```bash
# 4K 上下文（YaRN 自動啟用）
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=4096 ./shimmy serve

# 8K 上下文
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=8192 ./shimmy serve

# 16K 上下文
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=16384 ./shimmy serve

# 32K 上下文（需要充足的顯示記憶體）
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=32768 ./shimmy serve
```

### 上下文大小與顯示記憶體消耗

以 7B 模型（Q4_K_M）為例：

| 上下文長度 | 額外顯示記憶體佔用 | 適用情境 |
|-----------|----------------|---------|
| 2048（預設） | 基礎 | 短對話 |
| 4096 | +~0.5GB | 中等文件分析 |
| 8192 | +~1GB | 長文件處理 |
| 16384 | +~2GB | 超長文件、程式碼倉庫分析 |
| 32768 | +~4GB | 極長上下文（需高顯示記憶體） |

---

## 13. 連接開發工具

### VSCode / GitHub Copilot

```json
// .vscode/settings.json
{
  "github.copilot.advanced": {
    "serverUrl": "http://localhost:11435"
  }
}
```

### Continue.dev

```json
// ~/.continue/config.json
{
  "models": [
    {
      "title": "本地 Shimmy - Phi3 Mini",
      "provider": "openai",
      "model": "phi3-mini",
      "apiBase": "http://127.0.0.1:11435/v1",
      "apiKey": "sk-local"
    }
  ]
}
```

### Cursor 編輯器

1. 開啟 **Settings → Models → OpenAI API Key**
2. 填入任意字串作為 API Key（如 `sk-local`）
3. 展開 **Override OpenAI Base URL**
4. 填入 `http://127.0.0.1:11435/v1`

### Open WebUI（類 ChatGPT 本地介面）

```bash
# 安裝並啟動 Open WebUI（需要 Docker）
docker run -d -p 3000:8080 \
  -e OPENAI_API_BASE_URL="http://host.docker.internal:11435/v1" \
  -e OPENAI_API_KEY="sk-local" \
  ghcr.io/open-webui/open-webui:main

# 存取 http://localhost:3000
```

### SillyTavern（角色扮演前端）

1. 在 **API → Chat Completion → API Type** 中選擇 `OpenAI`
2. 填入 API Key：`sk-local`
3. 修改 Proxy URL 為：`http://127.0.0.1:11435/v1`
4. 點擊 **Connect**

---

## 14. 使用 SDK 呼叫

### Python（openai >= 1.0.0）

```python
from openai import OpenAI

# 連線至本地 Shimmy
client = OpenAI(
    base_url="http://127.0.0.1:11435/v1",
    api_key="sk-local"  # 任意字串，Shimmy 忽略此欄位
)

# 非串流呼叫
response = client.chat.completions.create(
    model="phi3-mini",
    messages=[
        {"role": "system", "content": "你是一個專業的繁體中文助手。"},
        {"role": "user", "content": "請介紹量子計算的基本原理。"}
    ],
    max_tokens=512,
    temperature=0.7
)
print(response.choices[0].message.content)

# 串流呼叫
stream = client.chat.completions.create(
    model="phi3-mini",
    messages=[{"role": "user", "content": "寫一個關於農曆新年的短故事"}],
    max_tokens=300,
    stream=True
)
for chunk in stream:
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="", flush=True)
print()  # 換行
```

### Python 非同步呼叫（asyncio）

```python
import asyncio
from openai import AsyncOpenAI

client = AsyncOpenAI(
    base_url="http://127.0.0.1:11435/v1",
    api_key="sk-local"
)

async def chat(prompt: str) -> str:
    response = await client.chat.completions.create(
        model="phi3-mini",
        messages=[{"role": "user", "content": prompt}],
        max_tokens=256
    )
    return response.choices[0].message.content

async def main():
    result = await chat("用三句話解釋機器學習")
    print(result)

asyncio.run(main())
```

### Node.js / TypeScript（openai v4）

```typescript
import OpenAI from "openai";

const client = new OpenAI({
  baseURL: "http://127.0.0.1:11435/v1",
  apiKey: "sk-local"  // Shimmy 忽略此欄位
});

// 非串流
async function chat(prompt: string): Promise<string> {
  const response = await client.chat.completions.create({
    model: "phi3-mini",
    messages: [
      { role: "system", content: "你是一個簡潔的繁體中文助手。" },
      { role: "user", content: prompt }
    ],
    max_tokens: 256
  });
  return response.choices[0].message?.content ?? "";
}

// 串流
async function chatStream(prompt: string): Promise<void> {
  const stream = client.chat.completions.stream({
    model: "phi3-mini",
    messages: [{ role: "user", content: prompt }],
    max_tokens: 512
  });
  
  for await (const chunk of stream) {
    const text = chunk.choices[0]?.delta?.content ?? "";
    process.stdout.write(text);
  }
  console.log();
}

(async () => {
  console.log(await chat("解釋一下區塊鏈技術"));
  await chatStream("寫一首關於月亮的詩");
})();
```

### Go

```go
package main

import (
    "context"
    "fmt"
    "github.com/sashabaranov/go-openai"
)

func main() {
    config := openai.DefaultConfig("sk-local")
    config.BaseURL = "http://127.0.0.1:11435/v1"
    client := openai.NewClientWithConfig(config)

    resp, err := client.CreateChatCompletion(
        context.Background(),
        openai.ChatCompletionRequest{
            Model: "phi3-mini",
            Messages: []openai.ChatCompletionMessage{
                {Role: openai.ChatMessageRoleUser, Content: "用繁體中文介紹 Go 語言的特點"},
            },
            MaxTokens: 256,
        },
    )
    if err != nil {
        panic(err)
    }
    fmt.Println(resp.Choices[0].Message.Content)
}
```

### curl（通用）

```bash
# 完整範例（多輪對話）
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "phi3-mini",
    "messages": [
      {"role": "system", "content": "你是一個專業的程式設計助手，擅長 Rust 和 Python。"},
      {"role": "user", "content": "如何在 Rust 中實作一個簡單的 HTTP 伺服器？"},
      {"role": "assistant", "content": "你可以使用 axum 或 actix-web 框架..."},
      {"role": "user", "content": "給我一個用 axum 實作 Hello World 的完整範例"}
    ],
    "max_tokens": 500,
    "temperature": 0.3
  }' | jq -r '.choices[0].message.content'
```

---

## 15. 效能參考

### 各模型顯示記憶體佔用（Q4_K_M 量化）

| 模型參數量 | 模型範例 | 顯示記憶體佔用 |
|-----------|---------|--------------|
| 1B 參數 | Llama-3.2-1B | ~1GB |
| 3B 參數 | Phi-3-mini | ~2GB |
| 7B 參數 | Mistral-7B / Qwen2.5-7B | ~4-5GB |
| 13B 參數 | Llama-2-13B | ~8GB |
| 34B 參數 | CodeLlama-34B | ~20GB |
| 70B 參數 | Llama-3.1-70B | ~40GB |

### GPU 適配器優先順序

Airframe 透過 wgpu 列舉適配器，優先選擇獨立顯示卡（Discrete GPU）：

```
優先順序：
1. 獨立 GPU（NVIDIA Vulkan / D3D12）
2. 獨立 GPU（AMD Vulkan / D3D12）
3. 整合 GPU（Intel / AMD APU）
4. CPU 軟體渲染（Mesa llvmpipe / WARP）
```

使用 `shimmy gpu-info` 查看您的系統選擇了哪個適配器。

### 效能調校建議

```bash
# 減少日誌輸出（降低 I/O 負擔）
SHIMMY_LOG_LEVEL=error ./shimmy serve

# 限制搜尋目錄（加快啟動速度）
SHIMMY_BASE_GGUF=/models/model.gguf ./shimmy serve

# CPU 推論最佳化（無 GPU 時）
OMP_NUM_THREADS=8 ./shimmy serve --legacy
```

---

## 16. 常見問題排解

### 問題：找不到模型

```
Error: No models found. Set SHIMMY_BASE_GGUF or place .gguf files in ./models/
```

**解決方案：**

```bash
# 方案一：設定環境變數
export SHIMMY_BASE_GGUF=/path/to/your-model.gguf

# 方案二：將模型放到 models/ 目錄
mkdir -p models && cp your-model.gguf models/

# 方案三：查看 Shimmy 正在搜尋哪些目錄
./shimmy discover
```

### 問題：埠號被佔用

```
Error: Address already in use (os error 98): bind 127.0.0.1:11435
```

**解決方案：**

```bash
# 換一個埠號
./shimmy serve --bind 127.0.0.1:11436

# 或找到並停止佔用埠號的程序
# Linux/macOS：
lsof -i :11435
kill -9 <PID>

# Windows：
netstat -ano | findstr :11435
taskkill /PID <PID> /F
```

### 問題：GPU 未被使用（回退至 CPU）

```bash
# 查看 GPU 適配器資訊
./shimmy gpu-info

# 啟用 debug 日誌，查看 wgpu 適配器選擇過程
RUST_LOG=wgpu=debug ./shimmy serve
```

**常見原因：**
- GPU 驅動程式版本過舊（NVIDIA 請更新至最新版，AMD 請確認已安裝 Vulkan 支援）
- Linux 下缺少 Vulkan 函式庫（安裝：`sudo apt install libvulkan1 vulkan-tools`）
- Windows 下 DirectX 12 未啟用

### 問題：產生速度緩慢

**可能原因與解決方案：**

| 原因 | 解決方案 |
|------|---------|
| 使用 CPU 推論 | 確認 GPU 被正確偵測（執行 `shimmy gpu-info`） |
| 模型過大 | 改用更小的量化版本（Q4_0 代替 Q8_0） |
| 上下文過長 | 降低 `SHIMMY_MAX_CTX` 或縮短對話歷史 |
| 系統其他程式佔用 GPU | 關閉其他 GPU 密集型應用程式 |

### 問題：中文輸出不完整或亂碼

```bash
# 確認使用支援中文的模型
# 建議的中文模型：
# - Qwen2.5-7B-Instruct（通義千問，繁體中文表現優異）
# - DeepSeek-R1
# - Yi-6B / Yi-34B（零一萬物）
# 設定適當的 max_tokens
max_tokens=1024  # 中文每個漢字約 1-3 token
```

### 問題：API 回傳 404

```bash
# 確認服務是否正在執行
curl http://127.0.0.1:11435/api/health

# 確認模型名稱正確
curl http://127.0.0.1:11435/v1/models | jq '.data[].id'

# 使用正確的模型 ID 請求
curl ... -d '{"model": "正確的模型名稱", ...}'
```

---

## 17. 從 v1.x 遷移

| 變更點 | v1.x | v2.0 |
|--------|------|------|
| 預設推論引擎 | llama.cpp | Airframe（WGSL/WebGPU） |
| GPU 加速方式 | CUDA / Vulkan / OpenCL | WebGPU via wgpu（自動選擇） |
| 模型路徑設定 | `--model-path` | `SHIMMY_BASE_GGUF` 或 `--model-path` |
| `--gpu-backend cuda/vulkan` | 有效 | 被忽略（wgpu 自動選擇） |
| MoE 模型支援 | 預設支援 | Airframe roadmap 中 |
| `cargo install shimmy` | 不可用 | ✅ 可用 |

### 遷移步驟

```bash
# 第 1 步：下載 v2.0 執行檔（見上方「安裝」章節）

# 第 2 步：將模型路徑切換為環境變數
export SHIMMY_BASE_GGUF=/path/to/your-model.gguf

# 第 3 步：移除 GPU backend 相關旗標（不再需要）
# 舊命令：shimmy serve --gpu-backend cuda
# 新命令：shimmy serve    （wgpu 自動處理）

# 第 4 步：啟動並驗證
./shimmy serve
curl http://127.0.0.1:11435/api/health
```

如遇任何遷移問題，請參閱 [docs/MIGRATION_v2.md](MIGRATION_v2.md) 或在 [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues) 提問。

---

## 18. 從原始碼建置

如需從原始碼建置 Shimmy（例如為特定硬體編譯最佳化版本）：

### 前置需求

- Rust 穩定版（最新版，透過 [rustup.rs](https://rustup.rs) 安裝）
- Git

### 建置步驟

```bash
# 複製儲存庫（含 Airframe 子模組）
git clone https://github.com/Michael-A-Kuykendall/shimmy --recurse-submodules
cd shimmy

# 建置（僅 HuggingFace 引擎，快速建置，適合 CI）
cargo build --release

# 建置（含 Airframe GPU 引擎，建議用於正式使用）
cargo build --release --features airframe,huggingface

# 執行測試
cargo test --features huggingface

# 安裝至系統
cargo install --path . --features airframe,huggingface
```

### 交叉編譯

```bash
# 為 Linux ARM64 建置（在 x86_64 Linux 上）
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# 為 Windows 建置（在 Linux 上）
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

---

## 19. 贊助支持

**Shimmy 將永久免費**——沒有星號，沒有「現在免費」的附加條件。

如果 Shimmy 對您有幫助，請考慮贊助專案，這將協助我們持續維護並推進功能開發：

| 贊助層級 | 金額 | 權益 |
|---------|------|------|
| ☕ 咖啡支持者 | $5/月 | 永久感謝 + 贊助徽章 |
| 🐛 Bug 優先處理 | $25/月 | 優先支援 + 名字列入 SPONSORS.md |
| 🏢 企業支持者 | $100/月 | Logo 展示 + 每月答疑 |
| 🚀 基礎設施合作夥伴 | $500/月 | 直接支援 + roadmap 決策權 |

[**💝 成為贊助者**](https://github.com/sponsors/Michael-A-Kuykendall)

---

## 附錄：快速參考卡

```bash
# ===== 常用命令速查 =====

# 啟動伺服器
SHIMMY_BASE_GGUF=/path/to/model.gguf ./shimmy serve

# 延伸上下文
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=8192 ./shimmy serve

# 查看模型
./shimmy list

# GPU 資訊
./shimmy gpu-info

# 快速測試
curl -s http://127.0.0.1:11435/api/health
curl http://127.0.0.1:11435/v1/models

# 產生文字（API）
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"<模型名稱>","messages":[{"role":"user","content":"你好"}],"max_tokens":64}'

# 串流產生
curl -N http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"<模型名稱>","stream":true,"messages":[{"role":"user","content":"講個故事"}]}'

# 預設埠：11435
# 預設位址：http://127.0.0.1:11435
# OpenAI API 前綴：/v1
# Ollama 相容前綴：/api
```

---

*本文件與 Shimmy 主儲存庫同步維護。如發現錯誤或有改進建議，歡迎提交 [Issue](https://github.com/Michael-A-Kuykendall/shimmy/issues) 或 Pull Request。*

*[English README](../README.md) | [简体中文手册](USER_MANUAL.zh-CN.md)*
