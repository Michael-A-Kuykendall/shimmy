<div align="center">

# Shimmy 中文文档中心

### 轻量级本地 AI 推理服务器 · 兼容 OpenAI API · 永久免费

**简体中文** · [繁體中文](../zh-TW/README.md) · [English](../../README.md)

[![GitHub Stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=social)](https://github.com/Michael-A-Kuykendall/shimmy/stargazers)
[![💝 赞助项目](https://img.shields.io/badge/💝_赞助项目-ea4aaa?style=flat&logo=github&logoColor=white)](https://github.com/sponsors/Michael-A-Kuykendall)

</div>

---

## 欢迎使用 Shimmy

Shimmy 是一个用纯 **Rust** 编写的本地 AI 推理服务器，兼容 **OpenAI API**，无需云端连接，无订阅费用，**永久免费**。

它的核心引擎 **Airframe** 使用 WebGPU（WGSL 计算着色器）直接在您的 GPU 上运行推理——无需 CUDA、无需 Python、无需 C++ 工具链。只需一个二进制文件，30 秒内即可运行您的第一个本地模型。

---

## � 支持 Shimmy 的发展

🚀 **如果 Shimmy 对您有帮助，欢迎[赞助支持](https://github.com/sponsors/Michael-A-Kuykendall)——所有赞助款项 100% 用于保持项目永久免费。**

- **$5/月**：咖啡档 ☕ 永久感谢 + 赞助者徽章
- **$25/月**：Bug 优先处理档 🐛 优先支持 + 名字收录于 [SPONSORS.md](../../SPONSORS.md)
- **$100/月**：企业支持档 🏢 Logo 展示 + 每月答疑
- **$500/月**：基础设施合作档 🚀 直接支持 + 路线图参与

[**🎯 成为赞助者**](https://github.com/sponsors/Michael-A-Kuykendall) | 查看[赞助者名单](../../SPONSORS.md) 🙏

---

## �📚 文档索引

### 入门指南

| 文档 | 说明 |
|------|------|
| [用户手册（完整版）](../USER_MANUAL.zh-CN.md) | 从安装到 API 的完整中文手册（19 个章节，1200+ 行） |
| [快速入门](../quickstart.md) | 5 分钟跑起来（英文，简单直接） |
| [从 v1.x 迁移](../MIGRATION_v2.md) | v1 升 v2 的变更与迁移指南（英文） |

### 核心技术深度解析

以下文档深入讲解 Shimmy 和 Airframe 引擎的内部机制，适合开发者和高级用户：

| 文档 | 说明 |
|------|------|
| [量化格式详解](QUANTIZATION.md) | Q4_0、Q8_0、K-quant 的位级原理，GPU 着色器如何在矩阵乘法中实时反量化 |
| [扩展上下文窗口](EXTENDED_CONTEXT.md) | YaRN RoPE 缩放原理，超长上下文的显存计算，各型号配置参考 |
| [故障排查指南](TROUBLESHOOTING.md) | GPU 错误、模型加载失败、端口冲突、OOM 等问题的诊断与修复 |
| [对话模板参考](CHAT_TEMPLATES.md) | ChatML / Llama-3 / OpenChat 三大模板族，自动识别逻辑，停止 token 说明 |
| [GPU 推理管线](GPU_PIPELINE.md) | 无绑定架构、预填充分块、KV 缓存布局、采样链路全解析 |

### API 与集成

| 文档 | 说明 |
|------|------|
| [API 参考](../API.md) | 完整的 HTTP 接口文档（英文） |
| [OpenAI 兼容矩阵](../OPENAI_COMPAT.md) | 哪些 OpenAI 参数已支持、哪些未支持（英文） |
| [集成示例](../INTEGRATION.md) | LangChain、OpenAI SDK、VSCode 等集成方法（英文） |

### 模型相关

| 文档 | 说明 |
|------|------|
| [模型扩展协议](../MODEL_EXPANSION.md) | 如何为新模型架构添加支持（英文） |
| [跨平台编译](../CROSS_COMPILATION.md) | 为 ARM、Linux、Windows 交叉编译（英文） |

---

## ⚡ 30 秒快速开始

```bash
# 1. 下载模型（以 TinyLlama 为例）
# 前往 https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF
# 下载 TinyLlama-1.1B-Chat-v1.0.Q4_0.gguf

# 2. 启动服务器
SHIMMY_BASE_GGUF=/path/to/TinyLlama-1.1B-Chat-v1.0.Q4_0.gguf ./shimmy serve

# 3. 发送请求
curl http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "local",
    "messages": [{"role": "user", "content": "你好！请介绍一下你自己。"}],
    "max_tokens": 128
  }'
```

就这些。无需配置文件，无需数据库，无需 Docker。

---

## ⚡ TurboShimmy INT4 KV —— v2.1.0 新功能

**TurboShimmy** 是 Shimmy v2.1.0 带来的纯 GPU INT4 KV 缓存压缩系统。通过 WGSL 计算着色器，将 KV 缓存从 32 位浮点数压缩为逐头向量 4 位整数，全程在 GPU 上完成。**一行指令，约 7 倍 KV 显存节省，输出品质不变。**

```bash
# 启用 TurboShimmy
SHIMMY_KV_QUANT=int4 ./shimmy serve

# 或通过命令行参数
./shimmy serve --kv-quant int4

# Windows + 长提示：预防 GPU TDR 重置
./shimmy serve --kv-quant int4 --prefill-chunk 8
```

**TurboShimmy 改变了消费级 GPU 的可用范围：**

| GPU 显存 | 未启用 TurboShimmy | 启用后（`--kv-quant int4`） |
|---|---|---|
| 3 GB | 仅能运行 Llama-3.2-1B | **Llama-3.2-3B 可运行 ✅** |
| 4 GB | Llama-3.2-3B，ctx=2048（勉强） | **Llama-3.2-3B，ctx=8192 ✅** |
| 6 GB | 3B 模型，短上下文 | **7B 模型，合理上下文 ✅** |

**显存对比（Llama-3.2-3B，ctx=2048）：**

| 模式 | KV 缓存 | 总显存 | 最低显存 |
|---|---|---|---|
| 默认（f32） | ~512 MB | ~2.4 GB | 3 GB（勉强） |
| TurboShimmy（int4） | **~72 MB** | **~2.0 GB** | **2.5 GB ✅** |

> **品质验证：** 在 Llama-3.2-3B 上进行的“大海捕针”基准测试表明，ctx≤2048 时 INT4 对比 F32 检索准确率零退化（各测试深度 15%〈50%〈85% 均为 100%）。详细文档：[TurboShimmy Wiki](https://github.com/Michael-A-Kuykendall/shimmy/wiki/TurboShimmy-zh-CN)。

---

## 🖥️ 已验证的 GPU 支持

| 平台 | GPU 类型 | 后端 |
|------|---------|------|
| Windows | NVIDIA / AMD / Intel | Direct3D 12 |
| Linux | NVIDIA / AMD / Intel | Vulkan |
| macOS | Apple Silicon / AMD | Metal |
| 任意平台 | 独显/集显 | 软件回退（慢） |

详见[故障排查指南](TROUBLESHOOTING.md)中的 GPU 专项章节。

---

## 🤖 支持的模型格式

Shimmy 加载 **GGUF 格式**模型，支持以下量化类型：

| 格式 | 每权重位数 | 质量 | 推荐 |
|------|-----------|------|------|
| Q4_K_M | 4.5 位 | ★★★★☆ | ✅ 首选 |
| Q4_0 | 4 位 | ★★★☆☆ | ✅ 兼容性最好 |
| Q8_0 | 8 位 | ★★★★★ | ✅ 最高质量 |
| Q5_K_M | 5.5 位 | ★★★★☆ | ✅ 质量/大小平衡 |

详见[量化格式详解](QUANTIZATION.md)。

---

## 💬 常见问题

**Shimmy 和 Ollama 有什么区别？**
Shimmy 是纯 Rust 实现，无 Python 运行时，无 C++ 依赖，启动时间不到 100ms。Airframe 引擎通过 WebGPU 而非 CUDA 直接驱动 GPU。

**如何选择模型量化格式？**
日常使用首选 `Q4_K_M`——在文件大小和推理质量之间取得了最好的平衡。若追求最高质量且显存充足，选 `Q8_0`。详见[量化格式详解](QUANTIZATION.md)。

**如何在 4 GB 显存的显卡上运行 3B 模型？**
启用 TurboShimmy：`SHIMMY_KV_QUANT=int4 ./shimmy serve`。这将 KV 显存减少约 7 倍，使 Llama-3.2-3B 能在 2.5 GB 总显存下运行。详见[上方 TurboShimmy 节](#turboshimmy-int4-kv--v210-)。

**上下文长度不够怎么办？**
设置 `SHIMMY_MAX_CTX=8192`（或更高）即可，Airframe 会自动应用 YaRN RoPE 缩放。注意超出模型原生上下文 2 倍以上时质量会有所下降。详见[扩展上下文窗口](EXTENDED_CONTEXT.md)。

**启动报 GPU 错误怎么办？**
运行 `shimmy gpu-info` 查看 GPU 适配器状态，然后参考[故障排查指南](TROUBLESHOOTING.md)。

**模型生成内容在奇怪的地方截断？**
可能是对话模板识别错误。参见[对话模板参考](CHAT_TEMPLATES.md)了解如何手动指定模板。

---

## 💝 支持项目

**Shimmy 将永久免费。** 如果它对您有帮助，欢迎赞助，支持持续开发：

| 赞助层级 | 金额 | 权益 |
|---------|------|------|
| ☕ 咖啡支持者 | $5/月 | 永久感谢 + 赞助徽章 |
| 🐛 Bug 优先处理 | $25/月 | 优先响应 + 名字列入 SPONSORS.md |
| 🏢 企业支持者 | $100/月 | Logo 展示 + 每月答疑 |
| 🚀 基础设施合作伙伴 | $500/月 | 直接支持 + roadmap 决策权 |

[**💝 成为赞助者**](https://github.com/sponsors/Michael-A-Kuykendall) · [查看赞助者名单](../../SPONSORS.md)

---

## 🐛 问题反馈与社区

- **提交 Bug**：[GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- **参与讨论**：[GitHub Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)
- **查看更新日志**：[CHANGELOG.md](../../CHANGELOG.md)

欢迎提交 Pull Request。贡献指南见 [CONTRIBUTING.md](../../CONTRIBUTING.md)。

---

*本文档与主仓库同步维护。发现问题或有改进建议，欢迎[提交 Issue](https://github.com/Michael-A-Kuykendall/shimmy/issues)。*

*[繁體中文文檔中心](../zh-TW/README.md) · [English Documentation](../../README.md)*
