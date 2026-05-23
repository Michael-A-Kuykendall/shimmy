# GPU 推理管线

本文档深入解析 Airframe 在 GPU 上运行 Transformer 推理的完整过程——着色器调度架构、无绑定资源模型、KV 缓存管理，以及采样链路。

适合阅读对象：准备修改着色器的贡献者、排查 GPU 层面故障的开发者，以及希望理解底层机制的用户。

---

## 架构总览

```
HTTP 请求抵达服务器
       │
       ▼
shimmy openai_compat 层
  - 解析 JSON 请求
  - 应用对话模板 → 提示词字符串
  - 构建 SamplingParams（温度、top_p、惩罚系数、停止 token）
       │
       ▼
airframe runtime::gpu::GpuRuntime::generate()
  - Tokenize（分词）提示词
  - 预填充阶段：处理提示词 token → 填充 KV 缓存
  - 解码阶段：逐 token 自回归生成
  - 反 Tokenize → 响应字符串
       │
       ▼
shimmy → HTTP 响应
```

---

## 无绑定资源模型

传统 WebGPU 需要将每个缓冲区单独绑定到绑定槽。大型 Transformer 模型有数千个张量——逐一绑定会触及 WebGPU 的绑定数量上限，并在每次调度时产生巨大开销。

Airframe 采用**无绑定（Bindless）设计**：每层的权重张量打包进一个大型存储缓冲区，WGSL 着色器通过推送常量（WebGPU 中以 uniform 缓冲区实现）中的字节偏移量来索引：

```
层缓冲区布局（每个 Transformer 层一个）：
┌────────────────────────────────────────────────────────────┐
│  attn_norm_weight  │  attn_q.weight  │  attn_k.weight  │  │
│  attn_v.weight  │  attn_o.weight  │  ffn_norm_weight  │  │
│  ffn_gate.weight  │  ffn_up.weight  │  ffn_down.weight  │  │
└────────────────────────────────────────────────────────────┘
        ↑
  每个张量区域以 GGUF 量化格式存储。
  着色器通过元数据缓冲区获取各区域的字节偏移量。
```

**调试提示**：如遇缓冲区绑定上限或绑定组创建失败的错误，问题通常不在无绑定权重缓冲区本身，而是 KV 缓存或激活缓冲区（这些仍独立绑定）。

---

## 预填充阶段 — 分块处理

**预填充阶段**一次性处理完整的输入提示词，填充 KV 缓存。这是单次请求中 GPU 计算最密集的部分。

Airframe 将长提示词切分为 **512 token 的分块**，以避免 GPU 命令编码器超时：

```
提示词 = 2048 token
          │
 ┌────────▼────────┐
 │  分块 1：512    │ → KV 缓存 0..511   ← 前向传播，写入 KV
 │  分块 2：512    │ → KV 缓存 512..1023
 │  分块 3：512    │ → KV 缓存 1024..1535
 │  分块 4：512    │ → KV 缓存 1536..2047
 └─────────────────┘
          │
  只使用最后一个分块的 logits 来生成第一个采样 token。
  所有分块的 KV 状态保留在 GPU 缓冲区中。
```

**为什么是 512？** WebGPU 对 GPU 命令执行时间有上限限制，超时会被操作系统终止。512 token 的分块在所有已测试硬件上（RTX 3060 到集成显卡）都能安全完成。

**调试跟踪**：设置 `AIRFRAME_TRACE_PREFILL_CHUNKS=1` 可以记录分块边界和处理时间。

---

## 解码阶段 — 自回归生成

预填充完成后，解码器逐 token 生成输出。每个解码步骤：

1. **嵌入**：通过嵌入查找表将上一个 token 转换为向量
2. **前向传播**：穿过所有 Transformer 层（使用 KV 缓存）
   - 注意力机制使用 KV 缓存——当前 Q 关注所有过去的 K/V 对
   - 每步只需计算位置 `t` 的新 Q/K/V 向量
3. **输出映射**：将 LM Head 应用到激活值，得到词表上的 logits
4. **采样**：从 logits 分布中选择下一个 token
5. **检查停止条件**：EOS token、额外停止 token、max_tokens 上限
6. 以新 token 返回第 1 步重复

每个解码步骤需要**完整的一次模型前向传播**。对于 22 层的 TinyLlama，每生成一个 token 就要执行 22 次注意力计算 + 22 次 FFN 计算。

**解码是内存带宽瓶颈，不是算力瓶颈**。速度限制来自从显存读取权重张量，而非浮点计算能力。显存带宽更大的 GPU 生成速度更快。

---

## Transformer 层计算（WGSL 着色器解析）

每个 Transformer 层按以下顺序执行：

```
输入激活值（形状：[seq_len, n_embd]）
  │
  ├── RMS Norm → normalized_x（稳定数值）
  │
  ├── Q 投影：normalized_x × attn_q.weight → Q [seq, n_heads, head_dim]
  ├── K 投影：normalized_x × attn_k.weight → K [seq, n_kv_heads, head_dim]
  ├── V 投影：normalized_x × attn_v.weight → V [seq, n_kv_heads, head_dim]
  │
  ├── RoPE：对 Q 和 K 应用旋转位置编码
  │     θ(t, i) = (t / yarn_scale) / base^(2i / head_dim)
  │     YaRN：当 ctx > native_ctx 时 yarn_scale = max_ctx / native_ctx
  │
  ├── 将 K、V 写入位置 t 的 KV 缓存
  │
  ├── 注意力得分：Q × Kᵀ / √head_dim → scores [seq, n_heads, seq]
  ├── Softmax（在 seq 维度） → weights
  ├── 加权求和：weights × V → attn_out [seq, n_heads, head_dim]
  │
  ├── 输出投影：attn_out × attn_o.weight → residual_add
  │
  ├── 残差连接：x = x + residual_add
  │
  ├── RMS Norm → normalized_for_ffn
  │
  ├── FFN gate：normalized_for_ffn × ffn_gate.weight → gate
  ├── FFN up：  normalized_for_ffn × ffn_up.weight   → up
  ├── SwiGLU 激活：gate × sigmoid(gate) × up
  ├── FFN down：swiglu_out × ffn_down.weight → ffn_residual
  │
  └── 残差连接：x = x + ffn_residual → 输出激活值
```

**反量化在矩阵乘法内部实时进行**：每个矩阵乘法着色器在读取量化权重块时立即解码，然后与激活值相乘，结果直接累加到输出，不产生额外的显存分配。

---

## KV 缓存

KV 缓存存储过去的键值对，避免在每个解码步骤重新计算。

**缓冲区布局：**
```
key_cache:   [n_layers][n_kv_heads][max_ctx][head_dim]  f32
value_cache: [n_layers][n_kv_heads][max_ctx][head_dim]  f32
```

以 F32 格式存储为扁平 GPU 缓冲区。总大小：
```
key_cache_bytes = n_layers × n_kv_heads × max_ctx × head_dim × 4
total = key_cache_bytes × 2
```

TinyLlama @ 2048 上下文：`22 × 4 × 2048 × 64 × 2 × 4 ≈ 88 MB`

**写入**：预填充和每个解码步骤中，当前位置的新 K/V 向量被写入 `cache[layer][head][position]`。

**读取**：注意力计算时，读回到 `current_position` 为止的完整 K/V 切片。

**重置**：每次请求之间缓存会被重置（清零所有位置）。Shimmy 是**无状态**的——不维护会话级别的 KV 缓存。

---

## 采样链路

前向传播产生词表上的 logits（长度为 `vocab_size` 的浮点向量）后，采样按以下顺序处理：

```
logits[vocab_size]
  │
  1. 重复惩罚（如果 repeat_penalty > 1.0）：
     对过去 N 个 token 中出现过的每个 token t：
       logits[t] /= repeat_penalty（提高重复的"代价"）
  │
  2. 温度缩放：
     logits[i] = logits[i] / temperature
     （temperature = 0.0 → 贪心；temperature → ∞ → 均匀随机）
  │
  3. Softmax：
     probs[i] = exp(logits[i]) / Σ exp(logits[j])
  │
  4. Top-p（核采样）：
     按概率降序排列。
     累计到总和 ≥ top_p 阈值为止。
     对保留的 token 重新归一化。
     从此压缩分布中采样。
  │
  5. 采样 → token_id
```

**确定性输出（贪心解码）**：`temperature=0.0, top_p=1.0`

**创意生成**：`temperature=0.8, top_p=0.95`

重复惩罚与 API 的 `frequency_penalty` / `presence_penalty` 字段的映射关系：
```
raw = max(frequency_penalty, presence_penalty)
如果 raw > 0.0：repeat_penalty = 1.0 + raw × 0.5
```

采样器在 CPU 上运行（logits 在前向传播后从 GPU 显存读回）。对于词表规模 32K 以内的模型，每 token 添加的延迟 < 1ms。

---

## 着色器调度模式

每次矩阵乘法是一个独立的计算调度。以 TinyLlama（22 层）单个解码步骤为例：

```
每 token 的大致调度次数：
  22 × (Q投影 + K投影 + V投影 + 注意力输出 + FFN_gate/up + FFN_down)
  ≈ 132 次调度
  + 22 次 RMSNorm + 22 次 RoPE + 22 次注意力得分 + 22 次 Softmax
  ≈ 共 220 次 GPU 调度/token
```

每次调度使用 `(16, 16, 1)` 的工作组，最大计算调用数为 256/工作组。命令编码器将所有调度打包为**每 token 一次**命令缓冲区提交——而不是每次调度单独提交。这对性能至关重要：每 token 一次 GPU 往返，而非 220 次。

**调试调度**：设置 `SHIMMY_DEBUG_RAW=1` 可记录管线各点的原始激活值（极度冗长——仅用于着色器调试）。

---

## 输出头

最终层产生形状为 `[1, n_embd]` 的激活值（最后位置）。将其与**输出嵌入权重**（`output.weight`，形状 `[vocab_size, n_embd]`）相乘，得到 logits。

输出权重通常是显存占用最大的张量：以词表 32K、维度 4096 的模型为例，`output.weight` 全精度下为 `32768 × 4096 × 4 ≈ 512 MB`。GGUF 的 Q6_K 格式下约 210 MB。

**WebGPU 2 GB 缓冲区限制**：输出权重必须放入单个 GPU 缓冲区。词表规模超大的模型（如 Gemma-2 词表 256K）的输出嵌入超过 2 GB，无法加载。这是已知限制——参见[故障排查指南](TROUBLESHOOTING.md)。

---

## 性能参考

**Token 生成速度**主要取决于：
1. GPU 显存带宽（不是算力 FLOPS）——权重读取是主要开销
2. 模型大小——参数越多，每 token 读取的字节越多
3. 上下文长度——解码阶段注意力得分按 O(n) 规模增长

**RTX 3060 12GB 实测基准：**

| 模型 | 上下文 | Token/秒 |
|------|--------|---------|
| TinyLlama 1.1B Q4_0 | 2048 | ~35-50 |
| Llama-3.2-1B Q4_K_M | 2048 | ~30-45 |
| Llama-3.2-3B Q4_K_M | 2048 | ~12-18 |
| Phi-2 2.7B Q4_K_M | 2048 | ~10-15 |

预填充通常比解码快 3-10 倍（每 token），因为注意力和 FFN 可以在序列维度上并行处理多个 token。

---

## 延伸阅读

- [量化格式详解](QUANTIZATION.md) — 着色器内部反量化
- [扩展上下文窗口](EXTENDED_CONTEXT.md) — YaRN RoPE 缩放实现
- [故障排查指南](TROUBLESHOOTING.md) — GPU 故障调试
