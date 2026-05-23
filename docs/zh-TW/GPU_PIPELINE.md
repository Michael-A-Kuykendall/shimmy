# GPU 推論管線

本文件深入解析 Airframe 在 GPU 上執行 Transformer 推論的完整過程——著色器調度架構、無綁定資源模型、KV 快取管理，以及採樣鏈路。

---

## 架構總覽

```
HTTP 請求抵達伺服器
       │
       ▼
shimmy openai_compat 層
  - 解析 JSON 請求
  - 套用對話範本 → 提示詞字串
  - 建立 SamplingParams（溫度、top_p、懲罰係數、停止 token）
       │
       ▼
airframe runtime::gpu::GpuRuntime::generate()
  - Tokenize（分詞）提示詞
  - 預填充階段：處理提示詞 token → 填充 KV 快取
  - 解碼階段：逐 token 自迴歸生成
  - 反 Tokenize → 回應字串
       │
       ▼
shimmy → HTTP 回應
```

---

## 無綁定資源模型

Airframe 採用**無綁定（Bindless）設計**：每層的權重張量打包進一個大型儲存緩衝區，WGSL 著色器透過推送常數（WebGPU 中以 uniform 緩衝區實作）中的位元組偏移量來索引：

```
層緩衝區布局（每個 Transformer 層一個）：
┌────────────────────────────────────────────────────────────┐
│  attn_norm_weight  │  attn_q.weight  │  attn_k.weight  │  │
│  attn_v.weight  │  attn_o.weight  │  ffn_norm_weight  │  │
│  ffn_gate.weight  │  ffn_up.weight  │  ffn_down.weight  │  │
└────────────────────────────────────────────────────────────┘
```

---

## 預填充階段 — 分塊處理

Airframe 將長提示詞切分為 **512 token 的分塊**，避免 GPU 指令編碼器逾時：

```
提示詞 = 2048 token
          │
 ┌────────▼────────┐
 │  分塊 1：512    │ → KV 快取 0..511
 │  分塊 2：512    │ → KV 快取 512..1023
 │  分塊 3：512    │ → KV 快取 1024..1535
 │  分塊 4：512    │ → KV 快取 1536..2047
 └─────────────────┘
```

設定 `AIRFRAME_TRACE_PREFILL_CHUNKS=1` 可記錄分塊邊界和處理時間。

---

## 解碼階段 — 自迴歸生成

預填充完成後，解碼器逐 token 生成輸出。每個解碼步驟：

1. **嵌入**：透過嵌入查找表將上一個 token 轉換為向量
2. **前向傳播**：穿過所有 Transformer 層（使用 KV 快取）
3. **輸出映射**：將 LM Head 套用到激活值，得到詞表上的 logits
4. **採樣**：從 logits 分布中選擇下一個 token
5. **檢查停止條件**：EOS token、額外停止 token、max_tokens 上限
6. 以新 token 返回第 1 步重複

**解碼是記憶體頻寬瓶頸，不是算力瓶頸。**

---

## Transformer 層計算

```
輸入激活值（形狀：[seq_len, n_embd]）
  │
  ├── RMS Norm → normalized_x
  ├── Q / K / V 投影
  ├── RoPE 旋轉位置編碼（YaRN 縮放）
  ├── 寫入 KV 快取
  ├── 注意力得分 + Softmax + 加權求和
  ├── 輸出投影 + 殘差連接
  ├── RMS Norm → normalized_for_ffn
  ├── FFN（SwiGLU 激活）
  └── 殘差連接 → 輸出激活值
```

**反量化在矩陣乘法內部即時進行**——沒有額外的顯存分配。

---

## KV 快取

**緩衝區布局：**
```
key_cache:   [n_layers][n_kv_heads][max_ctx][head_dim]  f32
value_cache: 同上
```

TinyLlama @ 2048 上下文：`22 × 4 × 2048 × 64 × 2 × 4 ≈ 88 MB`

每次請求之間快取會被重置（清零）。Shimmy 是**無狀態**的——不維護 session 級別的 KV 快取。

---

## 採樣鏈路

```
logits[vocab_size]
  │
  1. 重複懲罰（repeat_penalty > 1.0 時）
  2. 溫度縮放（temperature = 0.0 → 貪心）
  3. Softmax
  4. Top-p 核採樣
  5. 採樣 → token_id
```

**確定性輸出**：`temperature=0.0, top_p=1.0`

`frequency_penalty` / `presence_penalty` 映射：
```
raw = max(frequency_penalty, presence_penalty)
如果 raw > 0.0：repeat_penalty = 1.0 + raw × 0.5
```

---

## RTX 3060 12GB 效能參考

| 模型 | 上下文 | Token/秒 |
|------|--------|---------|
| TinyLlama 1.1B Q4_0 | 2048 | ~35-50 |
| Llama-3.2-1B Q4_K_M | 2048 | ~30-45 |
| Llama-3.2-3B Q4_K_M | 2048 | ~12-18 |
| Phi-2 2.7B Q4_K_M | 2048 | ~10-15 |

---

## 延伸閱讀

- [量化格式詳解](QUANTIZATION.md) — 著色器內部反量化
- [擴展上下文視窗](EXTENDED_CONTEXT.md) — YaRN RoPE 縮放實作
- [故障排查指南](TROUBLESHOOTING.md) — GPU 故障排查
