# Shimmy — 本地推理与 OpenAI 兼容 API

**语言：** [English](../../README.md) · **简体中文** · [繁體中文](../zh-TW/README.md)

> 本文档说明当前**已发布**的 Shimmy 2.6.6。此后完成的 Apple M1（虚拟）Metal 认证属于**尚未发布的候选实现**，不能据此声称已发布的 2.6.6 二进制包含后续修复。

Shimmy 是本地 GGUF 推理服务器及命令行程序，由 **Airframe 0.4.5** 提供 Rust/WGSL GPU 推理引擎。只实现经过明确限定的 OpenAI 兼容接口，并非完整 OpenAI API 替代品。

候选版本：**Shimmy 2.6.7 / Airframe 0.4.6**，尚未发布。以下安装命令仍对应已发布的 2.6.6，不会安装候选 Metal 修复。

## 安装与运行

使用具备 Rust 工具链和 GPU 驱动的系统：

```bash
cargo install shimmy --version 2.6.6 --locked
shimmy --version
shimmy --ctx 512 --kv-quant f32 serve --model-path /path/to/model.gguf --bind 127.0.0.1:11435
```

请将模型路径替换为[准确支持列表](../SUPPORTED_MODELS.md)中经过验证的 GGUF 文件。其他发布渠道（原生二进制、Homebrew、Scoop 及版本化 GHCR）见[安装指南](../BINARY_INSTALL.md)。Docker 的 `latest` 标签没有随 2.6.6 更新；本次不提供 npm 发布。仅进行 `fast` 构建不会提供推理功能。

## 模型、精度和平台范围

支持范围必须同时满足具体模型文件哈希、量化格式、上下文长度、推理方式和设备环境。已发布版本的可引用数值验证主要来自 NVIDIA/WSL Vulkan 环境：对相同 token ID 和反量化 FP32 基准比较完整 logits，最大绝对误差不超过 **0.1**，RMS 不超过 **0.03**，并核对贪心解码决策。

新候选实现已在 **Apple M1（Virtual）、Apple Paravirtual Metal、macOS 15.8.2** 上完成 **198/198** 次数值比较；这是候选认证，不表示目前发布的 2.6.6 已经采用该引擎。**实体 M4：NOT_RUN**。历史 8K 间歇性停顿原因未确定，watchdog 清理问题仍然存在。

只验证 F32 KV 与参考内核；INT4/TurboShimmy、实验性内核以及尚未实现的 Gemma2、MoE、MLA、循环/状态空间和多模态执行器均不属于已验证路径。能够解析某架构的元数据不等于该模型已通过推理认证。不存在通用速度、稳定性或多 GPU 保证。

## HTTP、安全与操作语义

已验证接口包括限定范围内的模型列表、JSON 聊天/文本补全以及**仅聊天**的 SSE。聊天默认返回 JSON；文本流式响应不受支持。token 用量、停止原因和错误处理按[API 规范](../API.md)及[兼容性说明](../OPENAI_COMPAT.md)执行。

本机 loopback 可不配置密钥；**非 loopback 地址必须设置 `SHIMMY_API_KEY`**。浏览器来源还需设置 `SHIMMY_ALLOWED_ORIGINS`。远程部署应采用 TLS 和网络访问控制。详情见[快速开始](../quickstart.md)、[配置](../CONFIGURATION.md)及[安全策略](../../SECURITY.md)。

## 许可与组成

Shimmy 使用 Apache-2.0；Airframe 与 airframe-observe 使用 MIT；ShimmyTok 使用 MIT **OR** Apache-2.0。libfse 0.1.4 另有明确的 [Apache-2.0 授权补充](https://github.com/Michael-A-Kuykendall/airframe/blob/main/crates/libfse/LICENSE-GRANT-0.1.4.md)，允许商业使用、修改和再分发，但第三方许可证与通知仍须保留。参阅[许可说明](../../LICENSING.md)。

## 文档与贡献

[英文 README](../../README.md) · [更新记录](../../CHANGELOG.md) · [路线图](../../ROADMAP.md) · [认证规则](../CERTIFICATION.md) · [模型范围](../SUPPORTED_MODELS.md) · [故障排查](../TROUBLESHOOTING.md) · [贡献指南](../../CONTRIBUTING.md)。

英文技术合同为权威版本；本文档不扩大其已验证范围。

完整细节见英文：[Install and run](../../README.md#install-and-run) · [Correctness and support](../../README.md#correctness-and-support) · [Server contract](../../README.md#server-contract) · [Documentation](../../README.md#documentation) · [Support](../../README.md#support) · [License](../../README.md#license--philosophy)