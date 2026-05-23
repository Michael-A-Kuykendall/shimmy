# 故障排查指南

本文档涵盖运行 Shimmy 时最常见的问题及其诊断方法和修复步骤。

---

## 快速诊断

遇到问题时，先运行以下命令获取基本信息：

```bash
# 检查服务器状态
curl -s http://127.0.0.1:11435/api/health

# 查看 GPU 适配器
shimmy gpu-info

# 列出已发现的模型
shimmy list

# 启用崩溃堆栈跟踪后重新运行
RUST_BACKTRACE=1 SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve
```

---

## 端口冲突

**症状**：服务器无法启动，报 `address already in use` 错误。

**Linux / macOS：**

```bash
# 查找占用端口的进程
lsof -i :11435

# 停止该进程
kill <PID>
```

**Windows：**

```powershell
netstat -ano | findstr :8080
taskkill /PID <PID> /F
```

**永久解决方案**：使用 `SHIMMY_PORT` 环境变量指定其他端口：

```bash
SHIMMY_PORT=12000 shimmy serve
```

> **注意**：Shimmy 默认端口为 11435（CLI 模式）或 8080（服务器二进制模式）。

---

## 模型未找到

**症状**：`shimmy list` 显示空列表，或启动时报 `model not found` 错误。

Shimmy 自动扫描以下目录：

1. `SHIMMY_BASE_GGUF` 或 `LIBSHIMMY_MODEL_PATH` 指定的路径
2. `~/.cache/huggingface/hub/`
3. `~/.ollama/models/`
4. `~/lm-studio/models/`
5. `~/.cache/lm-studio/models/`
6. `~/Library/Application Support/LMStudio/`（macOS）

**修复方法：**

```bash
# 方法 1：明确指定模型路径
SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve

# 方法 2：查看实际搜索了哪些路径
shimmy discover

# 方法 3：检查文件权限
ls -la /path/to/model.gguf
```

---

## GPU 适配器错误

### Windows — Direct3D 12

**症状**：`No suitable GPU adapter found` 或 `Failed to create device`

```powershell
# 检查驱动版本（需要 DirectX 12）
dxdiag

# 确认 DirectX 12 可用
shimmy gpu-info
```

**常见原因**：
- 驱动版本过旧（更新至最新版 NVIDIA/AMD 驱动）
- 使用了 Windows N 版本（缺少 DirectX 组件，需手动安装媒体功能包）
- 虚拟机中的 DirectX 12 不支持

### Linux — Vulkan

```bash
# 检查 Vulkan 安装
vulkaninfo | head -20

# NVIDIA
nvidia-smi
sudo apt install nvidia-vulkan-icd  # Ubuntu

# AMD
sudo apt install mesa-vulkan-drivers

# Intel
sudo apt install intel-media-va-driver mesa-vulkan-drivers
```

**常见原因**：
- 未安装 Vulkan ICD（驱动层）
- NVIDIA 驱动有 Vulkan 支持但未正确配置

### macOS — Metal

```bash
# 确认 macOS 版本 ≥ 11（Big Sur）
sw_vers

# 检查是否有多个 GPU（MacBook Pro 双显卡）
system_profiler SPDisplaysDataType
```

**Gatekeeper 拦截问题**：首次运行时如果 macOS 阻止 shimmy 执行：

```bash
xattr -d com.apple.quarantine /usr/local/bin/shimmy
```

---

## 显存不足（OOM）

**症状**：服务器崩溃，报 `out of memory`、`buffer binding error` 或 `WebGPU buffer size exceeded`。

### 根本原因：WebGPU 单缓冲区 2 GB 上限

WebGPU 规范限制单个缓冲区最大为 2 GB。这会影响某些特定模型的**输出嵌入矩阵**（形状为 `[vocab_size, n_embd]`）。

**已知受影响的模型：**

| 模型 | 原因 | 状态 |
|------|------|------|
| Gemma-2-2B Q4_K_M | 词表 256K，输出头矩阵 2.19 GB | 暂不支持 |
| Qwen 系列 7B | 词表 152K + 大维度 | 未验证 |

**应对方法：**
1. 改用词表较小的模型（Llama 系列词表 32K，不受影响）
2. 检查错误信息中是否提到 `output.weight`——若是，属于此已知限制

### 上下文长度导致的 OOM

```bash
# 显存不足时降低上下文长度
SHIMMY_MAX_CTX=2048 shimmy serve  # 从最小值开始
SHIMMY_MAX_CTX=4096 shimmy serve  # 逐步增加
```

---

## 推理问题

### 输出在奇怪的位置截断

**最可能的原因**：对话模板识别错误，导致错误的停止 token 触发了提前结束。

```bash
# 检查日志中的模板识别结果
RUST_LOG=debug shimmy serve 2>&1 | grep template
```

详细的模板说明见[对话模板参考](CHAT_TEMPLATES.md)。

### 生成不停止（超过 max_tokens）

这不是 bug。如果模型没有生成停止 token，它会一直生成直到达到 `max_tokens`。

```bash
# 明确设置较低的 max_tokens
curl ... -d '{"max_tokens": 256, ...}'
```

### 输出乱码或重复

**可能原因**：
1. 使用了错误的对话模板（参见[对话模板参考](CHAT_TEMPLATES.md)）
2. `temperature` 过高（尝试设为 0.0 测试确定性输出）
3. 模型量化格式不受支持（检查启动日志中的警告）
4. 上下文超出模型能力（尝试降低 `SHIMMY_MAX_CTX`）

### 生成速度非常慢

使用 `shimmy gpu-info` 检查是否正在使用 GPU。如果显示 `Software Rasterizer` 或 `llvmpipe`，说明在 CPU 上回退运行，速度会比 GPU 慢 10–50 倍。

---

## 已知不支持的模型

| 模型 | 问题 | 说明 |
|------|------|------|
| Phi-3 / Phi-3.5 系列 | 融合 QKV（`attn_qkv.weight`） | Airframe 期望独立的 Q/K/V 张量，Phi-3 将其合并为单一张量 |
| Gemma-2-2B Q4_K_M | 输出头超过 2 GB | WebGPU 缓冲区上限限制 |
| Qwen3 系列 | 缺少 QK Norm 着色器 | 架构尚不支持 |
| Q2_K / Q3_K | 量化格式不支持 | 使用 Q4_K_M 或更高格式 |

---

## 使用 RUST_BACKTRACE 获取详细错误信息

遇到崩溃时，启用 backtrace 可以看到完整的调用栈：

```bash
# Linux / macOS
RUST_BACKTRACE=1 SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve

# Windows PowerShell
$env:RUST_BACKTRACE = "1"
$env:LIBSHIMMY_MODEL_PATH = "D:\models\model.gguf"
.\shimmy_server_gpu.exe
```

请在提交 Bug 报告时附上完整的 backtrace 输出。

---

## 提交 Bug 报告

前往 [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues) 提交，请包含：

1. 操作系统和版本
2. GPU 型号和驱动版本（`shimmy gpu-info` 的输出）
3. 模型文件名和量化格式
4. 完整错误信息
5. 启用 `RUST_BACKTRACE=1` 后的完整输出

---

## 延伸阅读

- [对话模板参考](CHAT_TEMPLATES.md) — 模板识别错误的排查
- [量化格式详解](QUANTIZATION.md) — 不支持的量化格式
- [扩展上下文窗口](EXTENDED_CONTEXT.md) — 上下文相关的 OOM
- [GPU 推理管线](GPU_PIPELINE.md) — 底层架构

---

> 💝 **如果 Shimmy 对您有帮助，欢迎[赞助支持](https://github.com/sponsors/Michael-A-Kuykendall)——所有款项 100% 用于保持项目永久免费。**
