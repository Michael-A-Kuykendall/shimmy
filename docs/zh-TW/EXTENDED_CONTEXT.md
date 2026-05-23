# 擴展上下文視窗

本文件介紹 Airframe 如何透過 **YaRN RoPE 縮放**突破模型原生上下文長度限制——包含數學原理、各模型的顯存計算方法，以及在不同硬體上的實際配置建議。

---

## 什麼是上下文視窗？

上下文視窗是模型在生成回應時能夠「看到」的最大 token 數量。超出這個限制，模型要麼截斷輸入，要麼產生無意義的輸出——它根本無法引用視窗之外的內容。

常見模型的原生上下文長度：

| 模型 | 原生上下文 |
|------|-----------|
| TinyLlama 1.1B | 2,048 |
| Phi-2 2.7B | 2,048 |
| Llama-3.2-1B / 3B | 131,072 |
| StarCoder2 3B | 16,384 |
| Gemma-2 2B | 8,192 |

---

## RoPE 位置編碼原理

Transformer 模型透過**旋轉位置編碼（RoPE）**感知 token 在序列中的位置。對於序列位置 `t`、注意力頭維度中的第 `i` 個維度：

$$\theta(t, i) = \frac{t}{\text{base}^{2i / d_{\text{head}}}}$$

其中 `base` 通常為 10000，`d_head` 為注意力頭的維度。

**問題所在**：模型訓練時只見過 `t ≤ native_ctx` 的位置。當 `t` 超出訓練範圍，旋轉角度進入模型從未見過的區域，輸出品質迅速下降。

---

## YaRN：上下文長度擴展方案

**YaRN（Yet another RoPE extensioN）**透過對位置序列進行縮放，將超出範圍的位置映射回模型熟悉的區域：

$$\theta_{\text{YaRN}}(t, i) = \frac{t / s}{\text{base}^{2i / d_{\text{head}}}}$$

縮放因子 `s` 的計算公式：

$$s = \frac{\text{max\_ctx}}{\text{native\_ctx}}$$

**觸發條件**：當 `SHIMMY_MAX_CTX > native_ctx` 時，Airframe 自動啟用 YaRN：

```bash
SHIMMY_MAX_CTX=8192 ./shimmy serve
```

---

## 顯存（VRAM）計算

KV 快取是上下文擴展的顯存瓶頸。計算公式：

$$\text{KV 快取（位元組）} = n_{\text{layers}} \times n_{\text{kv\_heads}} \times d_{\text{head}} \times \text{max\_ctx} \times 2 \times 4$$

各模型實際用量：

### TinyLlama 1.1B
（22 層，4 個 KV 頭，head\_dim = 64）

| 上下文長度 | KV 快取大小 |
|-----------|------------|
| 2,048（原生） | 88 MB |
| 4,096 | 176 MB |
| 8,192 | 352 MB |

### Llama-3.2-1B
（16 層，8 個 KV 頭，head\_dim = 64）

| 上下文長度 | KV 快取大小 |
|-----------|------------|
| 8,192 | 512 MB |
| 16,384 | 1,024 MB |
| 32,768 | 2,048 MB |

### Llama-3.2-3B
（28 層，8 個 KV 頭，head\_dim = 128）

| 上下文長度 | KV 快取大小 |
|-----------|------------|
| 8,192 | 1,792 MB |
| 16,384 | 3,584 MB |

---

## 按顯存容量的推薦配置

| 顯存 | 推薦模型 + 上下文 |
|------|-----------------|
| 4 GB | TinyLlama Q4_0，最高 4K 上下文 |
| 6 GB | Llama-3.2-1B Q4_K_M，最高 8K 上下文 |
| 8 GB | Llama-3.2-3B Q4_K_M，最高 4K 上下文 |
| 12 GB | Llama-3.2-3B Q4_K_M，最高 8K 上下文 |
| 16 GB | 7B 模型 Q4_K_M，最高 8K 上下文 |
| 內顯（共享記憶體） | TinyLlama Q4_0，最高 2K 上下文 |

---

## YaRN 品質說明

| 倍數 | 品質狀況 |
|------|---------|
| 1× 以內（≤ 原生上下文） | 完整訓練精度，無損失 |
| 1–2× | 品質輕微下降，日常任務基本無感 |
| 2–4× | 遠端內容的注意力品質逐漸下降 |
| 4× 以上 | 輸出品質顯著退化，僅適合實驗性用途 |

---

## 環境變數配置

| 變數 | 說明 | 預設值 |
|------|------|--------|
| `SHIMMY_MAX_CTX` | 覆蓋最大上下文長度 | 讀取模型 GGUF 中的原生值 |
| `LIBSHIMMY_MODEL_PATH` | 模型檔案路徑（伺服器模式） | 無 |
| `SHIMMY_BASE_GGUF` | 模型檔案路徑（CLI 模式） | 自動探索 |

---

## 延伸閱讀

- [量化格式詳解](QUANTIZATION.md) — 壓縮格式與顯存佔用
- [故障排查指南](TROUBLESHOOTING.md) — 上下文溢出與 OOM 錯誤處理
- [GPU 推論管線](GPU_PIPELINE.md) — KV 快取在 GPU 記憶體中的布局

---

> 💝 **如果 Shimmy 對您有幫助，歡迎[贊助支持](https://github.com/sponsors/Michael-A-Kuykendall)——所有款項 100% 用於保持專案永久免費。**
