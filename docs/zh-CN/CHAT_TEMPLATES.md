# 对话模板参考

本文档说明 Shimmy 如何将对话消息格式化为模型输入，介绍三大模板族的格式细节，并解释自动识别逻辑——帮助你在输出异常时快速定位问题。

---

## 为什么模板很重要

大型语言模型在训练时使用了特定的**提示格式**。当输入格式与训练时不匹配，模型会产生奇怪的行为——输出提前截断、重复原始输入内容，或者持续生成无意义的文本。

这是实际工作中**最常见的静默失败场景**。模型运行正常，API 返回 200，但输出明显不对。

---

## 三大模板族

### 1. ChatML

**使用此模板的模型**：TinyLlama、Phi-2、StarCoder2（已通过验证），以及大多数未被识别为 Llama-3 的模型（默认回退）。

**格式：**

```
<|im_start|>system
{system_message}<|im_end|>
<|im_start|>user
{user_message}<|im_end|>
<|im_start|>assistant
```

**停止 token**：`<|im_end|>`、`<|im_start|>`

**完整示例：**

```
<|im_start|>system
你是一个有帮助的助手。<|im_end|>
<|im_start|>user
中国的首都是哪里？<|im_end|>
<|im_start|>assistant
中国的首都是北京。<|im_end|>
```

---

### 2. Llama-3

**使用此模板的模型**：Llama-3.2-1B-Instruct、Llama-3.2-3B-Instruct，以及所有 Meta Llama-3 系列。

**格式：**

```
<|begin_of_text|><|start_header_id|>system<|end_header_id|>

{system_message}<|eot_id|><|start_header_id|>user<|end_header_id|>

{user_message}<|eot_id|><|start_header_id|>assistant<|end_header_id|>

```

**停止 token**：`<|eot_id|>`、`<|end_of_text|>`

**关键细节**：`<|end_header_id|>` 后面有两个换行符，这是正确格式的必要组成部分。停止 token 必须是 `<|eot_id|>`——不是 `\n`，不是 `###`，不是 `<|im_end|>`。

---

### 3. OpenChat

**使用此模板的模型**：专门采用 OpenChat 训练格式的模型。

**格式：**

```
GPT4 Correct User: {user_message}<|end_of_turn|>GPT4 Correct Assistant:
```

**停止 token**：此模板无额外停止 token（使用模型默认的 EOS token）。

---

## 自动识别逻辑

Shimmy 根据模型文件名自动推断模板类型，规则如下（按优先级排序）：

| 模型名称包含 | 识别为 |
|------------|--------|
| `llama-3`、`llama3`、`meta-llama-3` | Llama-3 |
| 其他所有情况 | ChatML（默认） |

**实现细节**：识别逻辑在 `infer_template()` 函数中，对模型名称进行小写比较，因此大小写不影响识别结果。

**当自动识别失败时**：可通过 API 请求中的 `template` 字段手动指定：

```json
{
  "model": "local",
  "template": "chatml",
  "messages": [...]
}
```

可选值：`"chatml"`、`"llama3"`、`"openchat"`。

---

## 各模型模板对照

| 模型 | 模板 | 停止 Token |
|------|------|-----------|
| TinyLlama-1.1B-Chat | ChatML | `<\|im_end\|>` |
| Llama-3.2-1B-Instruct | Llama-3 | `<\|eot_id\|>` |
| Llama-3.2-3B-Instruct | Llama-3 | `<\|eot_id\|>` |
| Phi-2 | ChatML | `<\|im_end\|>` |
| StarCoder2-3B | ChatML\* | 建议使用 `/v1/completions` |
| GPT-2 | 无（补全模型） | 建议使用 `/v1/completions` |

\* StarCoder2 是代码补全模型，不是指令模型。虽然可以接受 ChatML 格式输入，但建议直接使用 `/v1/completions` 接口效果更好。

---

## 补全接口 vs 对话接口

Shimmy 同时支持两个接口：

**`/v1/chat/completions`**（对话格式）：
```json
{
  "model": "local",
  "messages": [
    {"role": "system", "content": "你是代码助手"},
    {"role": "user", "content": "写一个快速排序"}
  ]
}
```

**`/v1/completions`**（原始补全格式，适合代码模型）：
```json
{
  "model": "local",
  "prompt": "def quicksort(arr):",
  "max_tokens": 200,
  "temperature": 0.2
}
```

对于 StarCoder2 和 GPT-2，优先使用 `/v1/completions`——它们在补全任务上的效果远好于模拟对话格式。

---

## 自定义停止 Token

通过 API 可以添加额外的停止条件：

```json
{
  "model": "local",
  "messages": [...],
  "stop": ["###", "---", "\n\n"]
}
```

**注意事项**：
- 停止 token 必须是词表中的**单个 token**，不能是多 token 序列
- 字符串 `"###"` 如果对应多个 token 则不会生效
- 建议使用模型原生的停止 token（如上表中的格式）

---

## 调试模板问题

**常见症状与原因：**

| 症状 | 可能原因 |
|------|---------|
| 输出包含原始模板标签（如 `<\|im_start\|>`） | 模型用了错误的模板，停止 token 未生效 |
| 回复重复用户输入 | 对话模板格式错误 |
| 生成停在奇怪的地方 | 错误的停止 token 被触发 |
| 每次输出内容相同 | 对话历史未正确传递 |

```bash
# 查看原始请求的模板渲染结果
RUST_LOG=debug shimmy serve 2>&1 | grep -i template
```

---

## 延伸阅读

- [故障排查指南](TROUBLESHOOTING.md) — 生成质量问题的系统排查
- [量化格式详解](QUANTIZATION.md) — 模型格式与兼容性
- [GPU 推理管线](GPU_PIPELINE.md) — Token 采样链路

---

> 💝 **如果 Shimmy 对您有帮助，欢迎[赞助支持](https://github.com/sponsors/Michael-A-Kuykendall)——所有款项 100% 用于保持项目永久免费。**
