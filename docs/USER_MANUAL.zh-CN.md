<div align="center">

# Shimmy 用户手册

### 轻量级本地 AI 推理服务器，兼容 OpenAI API

**简体中文** · [繁體中文](USER_MANUAL.zh-TW.md) · [English](../README.md)

版本：v2.0.0 及以上 · 最后更新：2026 年 5 月

</div>

---

## 目录

1. [Shimmy 简介](#1-shimmy-简介)
2. [Airframe 引擎](#2-airframe-引擎)
3. [系统要求](#3-系统要求)
4. [安装](#4-安装)
5. [快速入门（30 秒）](#5-快速入门30-秒)
6. [获取模型文件](#6-获取模型文件)
7. [环境变量配置](#7-环境变量配置)
8. [命令行参考](#8-命令行参考)
9. [OpenAI 兼容 API](#9-openai-兼容-api)
10. [WebSocket 流式 API](#10-websocket-流式-api)
11. [模型自动发现](#11-模型自动发现)
12. [扩展上下文窗口（YaRN RoPE）](#12-扩展上下文窗口yarn-rope)
13. [连接开发工具](#13-连接开发工具)
14. [使用 SDK 调用](#14-使用-sdk-调用)
15. [性能参考](#15-性能参考)
16. [常见问题排查](#16-常见问题排查)
17. [从 v1.x 迁移](#17-从-v1x-迁移)
18. [构建源码](#18-构建源码)
19. [赞助支持](#19-赞助支持)

---

## 1. Shimmy 简介

Shimmy 是一个**单一二进制文件**的本地 AI 推理服务器，提供与 OpenAI API **100% 兼容**的 HTTP 接口，专为在本地运行大型语言模型（LLM）而设计。

### 核心优势

| 特性 | 说明 |
|------|------|
| 🔒 **完全本地** | 所有推理在您的设备上进行，数据不会上传到任何云端 |
| 🆓 **永久免费** | 无订阅费，无按 token 计费，无任何隐藏收费 |
| ⚡ **零配置** | 无需配置文件，自动检测模型和 GPU |
| 🔌 **即插即用** | 现有 OpenAI 客户端代码**无需修改**，仅替换 API 地址即可 |
| 🦀 **纯 Rust 实现** | 单一可执行文件，无 Python 运行时，无 C++ 依赖 |
| 🖥️ **跨平台 GPU 加速** | 通过 WebGPU（wgpu）支持 NVIDIA、AMD、Intel 及 Apple Silicon |

### v2.0.0 新特性

- **Airframe 引擎**：全新纯 Rust WGSL 推理引擎，彻底取代 llama.cpp
- **WebGPU 加速**：通过 wgpu 自动选择最佳 GPU 适配器
- **YaRN RoPE 缩放**：支持超长上下文窗口（最高 16384+ tokens）
- **零工具链依赖**：无需 CUDA 工具包、Vulkan SDK 或任何 C++ 编译器

---

## 2. Airframe 引擎

从 v2.0.0 起，Shimmy 的默认推理引擎为 **Airframe**——一个从零开始用纯 Rust 编写的 WebGPU（WGSL）变换器运行时。

### 技术架构

```
您的请求
    │
    ▼
Shimmy HTTP 服务器（Axum，纯 Rust）
    │
    ▼
Airframe 推理引擎
    │
    ├─ WGSL 计算着色器（在运行时由 wgpu 编译）
    ├─ F32 全精度计算
    ├─ YaRN RoPE 扩展（长上下文）
    └─ GPU 自动选择（wgpu 枚举最佳适配器）
         │
         ├─ NVIDIA → Vulkan / Direct3D 12
         ├─ AMD   → Vulkan / Direct3D 12
         ├─ Intel → Vulkan / Direct3D 12
         └─ Apple → Metal
```

### 为什么选择 Airframe？

- **无 C++ 工具链**：从上到下全部使用 Rust，无需安装 CUDA、Vulkan SDK 等
- **F32 全精度**：确定性输出，高质量推理结果
- **WGSL 着色器**：通过 WebGPU 在任何 GPU 上运行
- **从 GGUF 元数据自动推导模型规格**：无需硬编码每个模型的参数
- **GPU 自动检测**：运行时自动枚举并选择最佳 GPU 适配器

---

## 3. 系统要求

### 最低要求

| 组件 | 最低要求 |
|------|---------|
| 操作系统 | Windows 10/11、Linux（x86_64/ARM64）、macOS 12+ |
| CPU | x86_64 或 ARM64（Apple Silicon） |
| 内存 | 8GB RAM（推荐 16GB+） |
| 存储空间 | 5GB 以上（用于存放模型文件） |
| GPU | 可选，但强烈推荐（支持 Vulkan、Direct3D 12 或 Metal 的 GPU） |

### 推荐配置

| 用途 | GPU | VRAM | RAM |
|------|-----|------|-----|
| 小型模型（1B-3B） | 任意 GPU | 4GB | 8GB |
| 中型模型（7B-13B） | RTX 3060 / RX 6700 | 8-12GB | 16GB |
| 大型模型（30B+） | RTX 4090 / A100 | 24GB+ | 32GB+ |
| Apple Silicon | M1/M2/M3/M4 | 统一内存 16GB+ | — |

> **提示**：无 GPU 时，Shimmy 自动回退到 CPU 推理，速度较慢但完全可用。

---

## 4. 安装

### 方式一：下载预编译二进制文件（推荐）

预编译版本内置 Airframe WebGPU 引擎，**开箱即用，无需任何依赖**。

#### Windows（x86_64）

```powershell
# 使用 curl（Windows 10 及以上自带）
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# 或者使用 PowerShell
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

> **ARM64 注意**：Linux ARM64 版本使用 HuggingFace 引擎。Airframe 的 ARM64 交叉编译支持正在开发中。

### 方式二：通过 cargo 安装

```bash
# 从 crates.io 安装（使用 HuggingFace 引擎，无需 GPU 即可运行）
cargo install shimmy
```

### 方式三：从源码构建（含 Airframe 引擎）

```bash
git clone https://github.com/Michael-A-Kuykendall/shimmy --recurse-submodules
cd shimmy
cargo build --release --features airframe,huggingface

# 构建完成后，二进制文件位于：
./target/release/shimmy
```

---

## 5. 快速入门（30 秒）

```bash
# 第 1 步：设置模型路径并启动服务
SHIMMY_BASE_GGUF=/path/to/your-model.gguf ./shimmy serve

# Windows 用户：
set SHIMMY_BASE_GGUF=C:\models\your-model.gguf && shimmy.exe serve

# 第 2 步：查看已注册的模型
./shimmy list

# 第 3 步：测试 API（使用 curl）
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "your-model-name",
    "messages": [{"role": "user", "content": "你好，请用五个字回答。"}],
    "max_tokens": 32
  }' | jq -r '.choices[0].message.content'
```

服务启动后，Shimmy 监听在 `http://127.0.0.1:11435`。

---

## 6. 获取模型文件

Shimmy 使用 **GGUF 格式**的模型文件，这是目前最通用的量化模型格式。

### 推荐模型

以下模型已通过 GPU 数学验证（`quant_verify`），可与 Shimmy Airframe 引擎配合使用：

| 模型 | 架构 | 量化 | 大小 | 最小 VRAM | 下载地址 |
|------|------|------|------|-----------|----------|
| TinyLlama-1.1B-Chat | Llama | Q4_0 | 638MB | ~800MB | [HuggingFace](https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF) |
| Llama-3.2-1B-Instruct | Llama | Q4_K_M | ~770MB | ~1GB | [HuggingFace](https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF) |
| Llama-3.2-3B-Instruct | Llama | Q4_K_M | ~1.9GB | ~2.5GB | [HuggingFace](https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF) |
| phi-2 | Phi-2 | Q4_K_M | ~1.7GB | ~2.2GB | [HuggingFace](https://huggingface.co/TheBloke/phi-2-GGUF) |
| gemma-2-2b-it | Gemma-2 | Q4_K_M | ~1.6GB | ~2GB | [HuggingFace](https://huggingface.co/bartowski/gemma-2-2b-it-GGUF) |
| starcoder2-3b | StarCoder2 | Q4_K_M | ~1.8GB | ~2.3GB | [HuggingFace](https://huggingface.co/second-state/StarCoder2-3B-GGUF) |

**以下模型需要更大显存（≥16GB），将在路线图中支持：**

| 模型 | 量化 | 大小 | 状态 |
|------|------|------|------|
| deepseek-coder-6.7b-instruct | Q4_K_M | ~3.9GB | 待远程 GPU 验证 |
| deepseek-llm-7b-chat | Q4_K_M | ~4.0GB | 待远程 GPU 验证 |
| qwen2-7b-instruct | Q4_K_M | ~4.5GB | 待远程 GPU 验证 |

### 使用 huggingface-cli 下载

```bash
# 安装 huggingface_hub
pip install huggingface_hub

# 下载示例：Phi-3-mini
huggingface-cli download microsoft/Phi-3-mini-4k-instruct-gguf \
  Phi-3-mini-4k-instruct-q4.gguf \
  --local-dir ./models/

# 使用国内镜像（hf-mirror.com）
HF_ENDPOINT=https://hf-mirror.com huggingface-cli download \
  microsoft/Phi-3-mini-4k-instruct-gguf \
  Phi-3-mini-4k-instruct-q4.gguf \
  --local-dir ./models/
```

> **提示（中国大陆用户）**：如 HuggingFace 访问受限，可使用镜像站 `https://hf-mirror.com`，
> 设置环境变量 `HF_ENDPOINT=https://hf-mirror.com` 即可。

### 量化等级说明

GGUF 文件名中的量化后缀含义：

| 后缀 | 精度 | 内存占用 | 质量 | 适用场景 |
|------|------|---------|------|---------|
| `Q4_0` | 4-bit | 最小 | 好 | 日常使用，内存受限 |
| `Q4_K_M` | 4-bit（改进） | 小 | 较好 | 推荐的平衡选项 |
| `Q5_K_M` | 5-bit | 中等 | 很好 | 质量与速度均衡 |
| `Q6_K` | 6-bit | 较大 | 极好 | 高质量推理 |
| `Q8_0` | 8-bit | 大 | 接近原始 | 最高质量 |
| `F16` | 16-bit | 最大 | 原始精度 | 仅限高 VRAM 环境 |

---

## 7. 环境变量配置

### 必填变量

| 变量名 | 说明 | 示例 |
|--------|------|------|
| `SHIMMY_BASE_GGUF` | 模型文件路径（或使用 `--model-path` 参数） | `/models/phi3.gguf` |

### 可选变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `SHIMMY_MAX_CTX` | 模型原生（从 GGUF 自动读取） | 最大上下文 token 数，超过模型原生值时自动启用 YaRN |
| `SHIMMY_ENGINE_BACKEND` | `airframe` | 推理引擎，设为 `airframe`（默认）或 `safetensors` |
| `SHIMMY_PORT` | `11435` | 服务监听端口 |
| `SHIMMY_BIND_ADDRESS` | `127.0.0.1:11435` | 服务监听地址 |
| `SHIMMY_LOG_LEVEL` | `warn` | 日志级别：`error`/`warn`/`info`/`debug`/`trace` |
| `SHIMMY_MODEL_PATHS` | — | 额外模型目录，多个路径用 `;` 分隔 |
| `SHIMMY_LORA_GGUF` | — | LoRA 适配器文件路径 |
| `RUST_BACKTRACE` | — | 设为 `1` 开启崩溃堆栈跟踪（调试用） |
| `NO_COLOR` | — | 设为任意值禁用彩色输出 |

### 配置示例

```bash
# 最简启动（使用默认端口 11435）
SHIMMY_BASE_GGUF=/models/phi3-mini.gguf ./shimmy serve

# 指定端口和扩展上下文
SHIMMY_BASE_GGUF=/models/phi3-mini.gguf \
SHIMMY_MAX_CTX=8192 \
SHIMMY_PORT=8080 \
./shimmy serve

# 多目录模型搜索
SHIMMY_MODEL_PATHS="/data/models;/home/user/llm-models" \
./shimmy serve

# 启用详细日志
SHIMMY_BASE_GGUF=/models/model.gguf \
SHIMMY_LOG_LEVEL=debug \
./shimmy serve

# 对外提供服务（绑定所有网络接口，请注意安全风险）
SHIMMY_BASE_GGUF=/models/model.gguf \
./shimmy serve --bind 0.0.0.0:11435
```

---

## 8. 命令行参考

### 全局选项

```
shimmy [全局选项] <子命令>

全局选项：
  --model-dirs <目录列表>    额外的模型目录（多个目录用 ; 分隔）
  --gpu-backend <后端>       GPU 后端（auto/cpu）
  --cpu-moe                  将所有 MoE 专家层卸载到 CPU（节省显存）
  --n-cpu-moe <N>            将前 N 层 MoE 专家卸载到 CPU
  --legacy                   使用 CPU 适配器（而非 Airframe GPU）
  -h, --help                 显示帮助信息
  -V, --version              显示版本号
```

### serve — 启动 HTTP 服务器

```bash
shimmy serve [选项]

选项：
  --bind <地址>          绑定地址（默认：auto，自动分配端口）
  --model-path <路径>    模型文件路径（覆盖 SHIMMY_BASE_GGUF 环境变量）

示例：
  shimmy serve                                        # 使用默认设置
  shimmy serve --bind 127.0.0.1:11435                 # 指定端口
  shimmy serve --bind 0.0.0.0:8080                    # 对外服务
  shimmy serve --model-path /models/qwen2.5-7b.gguf   # 指定模型
  SHIMMY_MAX_CTX=4096 shimmy serve                    # 4K 上下文
```

### list — 列出已发现的模型

```bash
shimmy list

输出示例：
  phi3-mini-4k        /home/user/.cache/huggingface/hub/.../phi3-mini.gguf
  mistral-7b-instruct /models/mistral-7b-instruct.Q4_K_M.gguf
  qwen2.5-7b          /data/models/qwen2.5-7b-instruct-q4_k_m.gguf
```

### generate — 命令行直接生成文本

```bash
shimmy generate [选项]

选项：
  --name <模型名>      使用的模型名称
  --prompt <文本>      输入提示词
  --max-tokens <N>    最大生成 token 数（默认：256）
  --temperature <F>   采样温度（默认：0.7，范围 0.0-2.0）

示例：
  shimmy generate --name phi3-mini --prompt "请解释量子纠缠"
  shimmy generate --name mistral-7b --prompt "写一首七言绝句" --max-tokens 100
  shimmy generate --name qwen2.5 --prompt "Hello" --temperature 0.3
```

### discover — 发现并列出所有可用模型路径

```bash
shimmy discover

# 显示 Shimmy 搜索的所有目录及找到的模型文件
```

### gpu-info — 显示 GPU 适配器信息

```bash
shimmy gpu-info

输出示例：
  Adapter 0: NVIDIA GeForce RTX 4090 (Vulkan)
  Adapter 1: Intel UHD Graphics 770 (Vulkan)
  Selected: Adapter 0 (discrete GPU preferred)
```

---

## 9. OpenAI 兼容 API

Shimmy 在 `http://127.0.0.1:11435` 提供完整的 OpenAI 兼容 API，**无需修改现有代码**，只需将 `api_base` 替换为 Shimmy 地址即可。

### 接口总览

| 接口 | 方法 | 状态 | 说明 |
|------|------|------|------|
| `/v1/chat/completions` | POST | ✅ 支持 | 对话补全，支持流式输出 |
| `/v1/models` | GET | ✅ 支持 | 列出所有本地可用模型 |
| `/v1/models/:id` | GET | ✅ 支持 | 获取特定模型的元数据 |
| `/v1/completions` | POST | ✅ 支持 | 传统文本补全接口 |
| `/api/generate` | POST | ✅ 支持 | Ollama 兼容接口 |
| `/api/tags` | GET | ✅ 支持 | Ollama 兼容接口 |
| `/api/health` | GET | ✅ 支持 | 健康检查 |
| `/metrics` | GET | ✅ 支持 | Prometheus 指标 |
| `/v1/embeddings` | POST | ❌ 计划中 | 向量嵌入（roadmap） |

### POST /v1/chat/completions

#### 请求体

```json
{
  "model": "phi3-mini",
  "messages": [
    {
      "role": "system",
      "content": "你是一个专业的中文助手，回答简洁准确。"
    },
    {
      "role": "user",
      "content": "请解释什么是大语言模型。"
    }
  ],
  "max_tokens": 512,
  "temperature": 0.7,
  "top_p": 0.9,
  "stream": false
}
```

#### 支持的请求字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `model` | string | 是 | 本地模型 ID 或别名 |
| `messages` | array | 是 | 对话历史，每条包含 `role` 和 `content` |
| `stream` | boolean | 否 | `true` 启用 SSE 流式输出，默认 `false` |
| `max_tokens` | integer | 否 | 最大生成 token 数 |
| `temperature` | float | 否 | 采样温度（0.0-2.0，默认 0.7） |
| `top_p` | float | 否 | nucleus 采样（0.0-1.0，默认 0.9） |
| `top_k` | integer | 否 | Top-K 采样（默认 40） |
| `stop` | string/array | 否 | 停止词或停止词数组 |

#### 非流式响应

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
        "content": "大语言模型（LLM）是一种基于 Transformer 架构的神经网络..."
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

#### 流式响应（Server-Sent Events）

设置 `"stream": true` 后，服务器以 SSE 格式逐 token 推送：

```
data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"role":"assistant","content":"大"},"finish_reason":null}]}

data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"语"},"finish_reason":null}]}

data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

#### 使用 curl 测试

```bash
# 非流式
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "phi3-mini",
    "messages": [
      {"role": "system", "content": "你是一个简洁的中文助手。"},
      {"role": "user", "content": "介绍一下北京。"}
    ],
    "max_tokens": 200
  }' | jq -r '.choices[0].message.content'

# 流式（加 -N 禁用 curl 缓冲）
curl -N http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "phi3-mini",
    "stream": true,
    "messages": [{"role": "user", "content": "数数从 1 到 10"}]
  }'
```

### GET /v1/models

列出所有已注册和自动发现的本地模型。

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

健康检查接口，可用于监控系统和负载均衡器探针。

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

## 10. WebSocket 流式 API

除 HTTP SSE 外，Shimmy 还提供 WebSocket 接口实现实时 token 推送。

### 连接地址

```
ws://127.0.0.1:11435/ws/generate
```

### 发送请求

连接后发送 JSON 消息：

```json
{
  "model": "phi3-mini",
  "prompt": "请写一首关于秋天的现代诗",
  "max_tokens": 200,
  "temperature": 0.8
}
```

### 接收响应

服务器逐 token 推送：

```json
{"token": "秋"}
{"token": "风"}
{"token": "轻"}
{"token": "抚"}
...
{"done": true, "total_tokens": 87}
```

### JavaScript 示例

```javascript
const ws = new WebSocket('ws://127.0.0.1:11435/ws/generate');

ws.onopen = () => {
  ws.send(JSON.stringify({
    model: 'phi3-mini',
    prompt: '用中文介绍人工智能的发展历史',
    max_tokens: 300,
    temperature: 0.7
  }));
};

let output = '';
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.done) {
    console.log('生成完成，共', data.total_tokens, '个 token');
    ws.close();
  } else {
    output += data.token;
    process.stdout.write(data.token); // 实时打印
  }
};
```

---

## 11. 模型自动发现

Shimmy 启动时会自动扫描以下目录寻找 GGUF 和 SafeTensors 模型文件：

### 自动搜索路径

| 平台 | 自动搜索路径 |
|------|------------|
| 所有平台 | `./models/`（当前目录下的 models 文件夹） |
| 所有平台 | `~/models/` |
| 所有平台 | `~/Downloads/`（含 .gguf 文件） |
| Linux/macOS | `~/.cache/huggingface/hub/` |
| Windows | `%USERPROFILE%\.cache\huggingface\hub\` |
| Windows | `%LOCALAPPDATA%\Ollama\models\` |
| Linux | `~/.ollama/models/` |
| macOS | `~/Library/Application Support/Ollama/models/` |

### 添加自定义搜索目录

```bash
# 方式一：环境变量（多个路径用 ; 分隔）
SHIMMY_MODEL_PATHS="/data/models;/mnt/nas/llm;/tmp/models" ./shimmy serve

# 方式二：命令行参数
./shimmy serve --model-dirs "/data/models;/mnt/nas/llm"

# 查看所有发现的模型
./shimmy list

# 查看所有正在搜索的目录
./shimmy discover
```

### 模型命名规则

Shimmy 会根据文件名自动生成模型 ID：

- `/models/Qwen2.5-7B-Instruct-Q4_K_M.gguf` → ID 为 `qwen2.5-7b-instruct`
- `/models/phi-3-mini-4k-instruct-q4.gguf` → ID 为 `phi-3-mini-4k-instruct`
- `/models/deepseek-r1-1.5b.gguf` → ID 为 `deepseek-r1-1.5b`

---

## 12. 扩展上下文窗口（YaRN RoPE）

Shimmy v2.0 通过 Airframe 引擎内置 **YaRN RoPE 位置编码缩放**，无需任何额外配置即可支持超出模型原生训练上下文的输入长度。

### 使用方法

```bash
# 4K 上下文（YaRN 自动激活）
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=4096 ./shimmy serve

# 8K 上下文
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=8192 ./shimmy serve

# 16K 上下文
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=16384 ./shimmy serve

# 32K 上下文（需要足够的 VRAM）
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=32768 ./shimmy serve
```

### 上下文大小与 VRAM 消耗

以 7B 模型（Q4_K_M）为例：

| 上下文长度 | 额外 VRAM 占用 | 适用场景 |
|-----------|--------------|---------|
| 2048（默认） | 基础 | 短对话 |
| 4096 | +~0.5GB | 中等文档分析 |
| 8192 | +~1GB | 长文档处理 |
| 16384 | +~2GB | 超长文档、代码仓库分析 |
| 32768 | +~4GB | 极长上下文（需高 VRAM） |

---

## 13. 连接开发工具

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

### Cursor 编辑器

1. 打开 **Settings → Models → OpenAI API Key**
2. 填写任意字符串作为 API Key（如 `sk-local`）
3. 展开 **Override OpenAI Base URL**
4. 填入 `http://127.0.0.1:11435/v1`

### Open WebUI（类 ChatGPT 本地界面）

```bash
# 安装并启动 Open WebUI（需要 Docker）
docker run -d -p 3000:8080 \
  -e OPENAI_API_BASE_URL="http://host.docker.internal:11435/v1" \
  -e OPENAI_API_KEY="sk-local" \
  ghcr.io/open-webui/open-webui:main

# 访问 http://localhost:3000
```

### SillyTavern（角色扮演前端）

1. 在 **API → Chat Completion → API Type** 中选择 `OpenAI`
2. 填写 API Key：`sk-local`
3. 修改 Proxy URL 为：`http://127.0.0.1:11435/v1`
4. 点击 **Connect**

---

## 14. 使用 SDK 调用

### Python（openai >= 1.0.0）

```python
from openai import OpenAI

# 连接到本地 Shimmy
client = OpenAI(
    base_url="http://127.0.0.1:11435/v1",
    api_key="sk-local"  # 任意字符串，Shimmy 忽略此字段
)

# 非流式调用
response = client.chat.completions.create(
    model="phi3-mini",
    messages=[
        {"role": "system", "content": "你是一个专业的中文助手。"},
        {"role": "user", "content": "请介绍一下量子计算的基本原理。"}
    ],
    max_tokens=512,
    temperature=0.7
)
print(response.choices[0].message.content)

# 流式调用
stream = client.chat.completions.create(
    model="phi3-mini",
    messages=[{"role": "user", "content": "写一个关于春节的短故事"}],
    max_tokens=300,
    stream=True
)
for chunk in stream:
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="", flush=True)
print()  # 换行
```

### Python 异步调用（asyncio）

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
    result = await chat("用三句话解释机器学习")
    print(result)

asyncio.run(main())
```

### Node.js / TypeScript（openai v4）

```typescript
import OpenAI from "openai";

const client = new OpenAI({
  baseURL: "http://127.0.0.1:11435/v1",
  apiKey: "sk-local"  // Shimmy 忽略此字段
});

// 非流式
async function chat(prompt: string): Promise<string> {
  const response = await client.chat.completions.create({
    model: "phi3-mini",
    messages: [
      { role: "system", content: "你是一个简洁的中文助手。" },
      { role: "user", content: prompt }
    ],
    max_tokens: 256
  });
  return response.choices[0].message?.content ?? "";
}

// 流式
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
  console.log(await chat("解释一下区块链技术"));
  await chatStream("写一首关于月亮的诗");
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
                {Role: openai.ChatMessageRoleUser, Content: "用中文介绍 Go 语言的特点"},
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
# 完整示例（多轮对话）
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "phi3-mini",
    "messages": [
      {"role": "system", "content": "你是一个专业的编程助手，擅长 Rust 和 Python。"},
      {"role": "user", "content": "如何在 Rust 中实现一个简单的 HTTP 服务器？"},
      {"role": "assistant", "content": "你可以使用 axum 或 actix-web 框架..."},
      {"role": "user", "content": "给我一个用 axum 实现 Hello World 的完整示例"}
    ],
    "max_tokens": 500,
    "temperature": 0.3
  }' | jq -r '.choices[0].message.content'
```

---

## 15. 性能参考

### 各模型显存占用（Q4_K_M 量化）

| 模型参数量 | 模型示例 | 显存占用 |
|-----------|---------|---------|
| 1B 参数 | Llama-3.2-1B | ~1GB |
| 3B 参数 | Phi-3-mini | ~2GB |
| 7B 参数 | Mistral-7B / Qwen2.5-7B | ~4-5GB |
| 13B 参数 | Llama-2-13B | ~8GB |
| 34B 参数 | CodeLlama-34B | ~20GB |
| 70B 参数 | Llama-3.1-70B | ~40GB |

### GPU 适配器优先级

Airframe 通过 wgpu 枚举适配器，优先选择独立显卡（Discrete GPU）：

```
优先级：
1. 独立 GPU（NVIDIA Vulkan / D3D12）
2. 独立 GPU（AMD Vulkan / D3D12）
3. 集成 GPU（Intel / AMD APU）
4. CPU 软件渲染（Mesa llvmpipe / WARP）
```

使用 `shimmy gpu-info` 查看您的系统选择了哪个适配器。

### 性能调优建议

```bash
# 减少日志输出（减少 I/O 开销）
SHIMMY_LOG_LEVEL=error ./shimmy serve

# 限制搜索目录（加快启动速度）
SHIMMY_BASE_GGUF=/models/model.gguf ./shimmy serve

# CPU 推理优化（无 GPU 时）
OMP_NUM_THREADS=8 ./shimmy serve --legacy
```

---

## 16. 常见问题排查

### 问题：找不到模型

```
Error: No models found. Set SHIMMY_BASE_GGUF or place .gguf files in ./models/
```

**解决方案：**

```bash
# 方案一：设置环境变量
export SHIMMY_BASE_GGUF=/path/to/your-model.gguf

# 方案二：将模型放到 models/ 目录
mkdir -p models && cp your-model.gguf models/

# 方案三：查看 Shimmy 正在搜索哪些目录
./shimmy discover
```

### 问题：端口被占用

```
Error: Address already in use (os error 98): bind 127.0.0.1:11435
```

**解决方案：**

```bash
# 换一个端口
./shimmy serve --bind 127.0.0.1:11436

# 或者找到并停止占用端口的进程
# Linux/macOS：
lsof -i :11435
kill -9 <PID>

# Windows：
netstat -ano | findstr :11435
taskkill /PID <PID> /F
```

### 问题：GPU 未被使用（回退到 CPU）

```bash
# 查看 GPU 适配器信息
./shimmy gpu-info

# 启用 debug 日志，查看 wgpu 适配器选择过程
RUST_LOG=wgpu=debug ./shimmy serve
```

**常见原因：**
- GPU 驱动版本过旧（NVIDIA 请更新到最新版，AMD 请确保安装 Vulkan 支持）
- Linux 下缺少 Vulkan 库（安装：`sudo apt install libvulkan1 vulkan-tools`）
- Windows 下 DirectX 12 未启用

### 问题：生成速度缓慢

**可能原因和解决方案：**

| 原因 | 解决方案 |
|------|---------|
| 使用 CPU 推理 | 确认 GPU 被正确检测（运行 `shimmy gpu-info`） |
| 模型过大 | 改用更小的量化版本（Q4_0 代替 Q8_0） |
| 上下文过长 | 减少 `SHIMMY_MAX_CTX` 或缩短对话历史 |
| 系统其他程序占用 GPU | 关闭其他 GPU 密集型应用 |

### 问题：中文乱码或截断

```bash
# 确保使用支持中文的模型
# 推荐中文模型：
# - Qwen2.5-7B-Instruct（通义千问）
# - DeepSeek-R1
# - Yi-6B / Yi-34B
# 设置适当的 max_tokens
max_tokens=1024  # 中文每个汉字约 1-3 token
```

### 问题：API 返回 404

```bash
# 检查服务是否正在运行
curl http://127.0.0.1:11435/api/health

# 确认模型名称正确
curl http://127.0.0.1:11435/v1/models | jq '.data[].id'

# 使用正确的模型 ID 请求
curl ... -d '{"model": "正确的模型名称", ...}'
```

---

## 17. 从 v1.x 迁移

| 变更点 | v1.x | v2.0 |
|--------|------|------|
| 默认推理引擎 | llama.cpp | Airframe（WGSL/WebGPU） |
| GPU 加速方式 | CUDA / Vulkan / OpenCL | WebGPU via wgpu（自动选择） |
| 模型路径配置 | `--model-path` | `SHIMMY_BASE_GGUF` 或 `--model-path` |
| `--gpu-backend cuda/vulkan` | 有效 | 被忽略（wgpu 自动选择） |
| MoE 模型支持 | 默认支持 | Airframe roadmap 中 |
| `cargo install shimmy` | 不可用 | ✅ 可用 |

### 迁移步骤

```bash
# 第 1 步：下载 v2.0 二进制文件（见上方"安装"章节）

# 第 2 步：将模型路径切换为环境变量
export SHIMMY_BASE_GGUF=/path/to/your-model.gguf

# 第 3 步：移除 GPU backend 相关标志（不再需要）
# 旧命令：shimmy serve --gpu-backend cuda
# 新命令：shimmy serve    （wgpu 自动处理）

# 第 4 步：启动并验证
./shimmy serve
curl http://127.0.0.1:11435/api/health
```

如遇任何迁移问题，请参阅 [docs/MIGRATION_v2.md](MIGRATION_v2.md) 或在 [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues) 提问。

---

## 18. 构建源码

如需从源码构建 Shimmy（例如为特定硬件编译优化版本）：

### 前置要求

- Rust 稳定版（最新版，通过 [rustup.rs](https://rustup.rs) 安装）
- Git

### 构建步骤

```bash
# 克隆仓库（含 Airframe 子模块）
git clone https://github.com/Michael-A-Kuykendall/shimmy --recurse-submodules
cd shimmy

# 构建（仅 HuggingFace 引擎，快速构建，适合 CI）
cargo build --release

# 构建（含 Airframe GPU 引擎，推荐用于正式使用）
cargo build --release --features airframe,huggingface

# 运行测试
cargo test --features huggingface

# 安装到系统
cargo install --path . --features airframe,huggingface
```

### 交叉编译

```bash
# 为 Linux ARM64 构建（在 x86_64 Linux 上）
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# 为 Windows 构建（在 Linux 上）
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

---

## 19. 赞助支持

**Shimmy 将永久免费**——没有星号，没有"现在免费"的附加条件。

如果 Shimmy 对您有帮助，请考虑赞助项目，这将帮助我们持续维护并推进功能开发：

| 赞助层级 | 金额 | 权益 |
|---------|------|------|
| ☕ 咖啡支持者 | $5/月 | 永久感谢 + 赞助徽章 |
| 🐛 Bug 优先处理 | $25/月 | 优先支持 + 名字列入 SPONSORS.md |
| 🏢 企业支持者 | $100/月 | Logo 展示 + 每月答疑 |
| 🚀 基础设施合作伙伴 | $500/月 | 直接支持 + roadmap 决策权 |

[**💝 成为赞助者**](https://github.com/sponsors/Michael-A-Kuykendall)

---

## 附录：快速参考卡

```bash
# ===== 常用命令速查 =====

# 启动服务
SHIMMY_BASE_GGUF=/path/to/model.gguf ./shimmy serve

# 扩展上下文
SHIMMY_BASE_GGUF=/models/model.gguf SHIMMY_MAX_CTX=8192 ./shimmy serve

# 查看模型
./shimmy list

# GPU 信息
./shimmy gpu-info

# 快速测试
curl -s http://127.0.0.1:11435/api/health
curl http://127.0.0.1:11435/v1/models

# 生成文本（API）
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"<模型名>","messages":[{"role":"user","content":"你好"}],"max_tokens":64}'

# 流式生成
curl -N http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"<模型名>","stream":true,"messages":[{"role":"user","content":"讲个故事"}]}'

# 默认端口：11435
# 默认地址：http://127.0.0.1:11435
# OpenAI API 前缀：/v1
# Ollama 兼容前缀：/api
```

---

*本文档与 Shimmy 主仓库同步维护。如发现错误或有改进建议，欢迎提交 [Issue](https://github.com/Michael-A-Kuykendall/shimmy/issues) 或 Pull Request。*

*[English README](../README.md) | [繁體中文手冊](USER_MANUAL.zh-TW.md)*
