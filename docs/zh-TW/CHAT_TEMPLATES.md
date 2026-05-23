# 對話範本參考

本文件說明 Shimmy 如何將對話訊息格式化為模型輸入，介紹三大範本族的格式細節，並解釋自動識別邏輯——協助您在輸出異常時快速定位問題。

---

## 為什麼範本很重要

大型語言模型在訓練時使用了特定的**提示格式**。當輸入格式與訓練時不匹配，模型會產生奇怪的行為——輸出提前截斷、重複原始輸入內容，或者持續生成無意義的文字。

這是實際使用中**最常見的靜默失敗場景**。模型執行正常，API 回傳 200，但輸出明顯不對。

---

## 三大範本族

### 1. ChatML

**使用此範本的模型**：TinyLlama、Phi-2、StarCoder2（已通過驗證），以及大多數未被識別為 Llama-3 的模型（預設回退）。

**格式：**

```
<|im_start|>system
{system_message}<|im_end|>
<|im_start|>user
{user_message}<|im_end|>
<|im_start|>assistant
```

**停止 token**：`<|im_end|>`、`<|im_start|>`

---

### 2. Llama-3

**使用此範本的模型**：Llama-3.2-1B-Instruct、Llama-3.2-3B-Instruct，以及所有 Meta Llama-3 系列。

**格式：**

```
<|begin_of_text|><|start_header_id|>system<|end_header_id|>

{system_message}<|eot_id|><|start_header_id|>user<|end_header_id|>

{user_message}<|eot_id|><|start_header_id|>assistant<|end_header_id|>

```

**停止 token**：`<|eot_id|>`、`<|end_of_text|>`

**關鍵細節**：`<|end_header_id|>` 後面有兩個換行符號——這是正確格式的必要組成部分。

---

### 3. OpenChat

**格式：**

```
GPT4 Correct User: {user_message}<|end_of_turn|>GPT4 Correct Assistant:
```

**停止 token**：此範本無額外停止 token（使用模型預設的 EOS token）。

---

## 自動識別邏輯

| 模型名稱包含 | 識別為 |
|------------|--------|
| `llama-3`、`llama3`、`meta-llama-3` | Llama-3 |
| 其他所有情況 | ChatML（預設） |

**手動指定範本**：

```json
{
  "model": "local",
  "template": "chatml",
  "messages": [...]
}
```

可選值：`"chatml"`、`"llama3"`、`"openchat"`。

---

## 各模型範本對照

| 模型 | 範本 | 停止 Token |
|------|------|-----------|
| TinyLlama-1.1B-Chat | ChatML | `<\|im_end\|>` |
| Llama-3.2-1B-Instruct | Llama-3 | `<\|eot_id\|>` |
| Llama-3.2-3B-Instruct | Llama-3 | `<\|eot_id\|>` |
| Phi-2 | ChatML | `<\|im_end\|>` |
| StarCoder2-3B | ChatML\* | 建議使用 `/v1/completions` |
| GPT-2 | 無（補全模型） | 建議使用 `/v1/completions` |

\* StarCoder2 是程式碼補全模型。建議直接使用 `/v1/completions` 介面。

---

## 補全介面 vs 對話介面

**`/v1/chat/completions`**（對話格式）：
```json
{
  "model": "local",
  "messages": [
    {"role": "system", "content": "你是程式碼助手"},
    {"role": "user", "content": "寫一個快速排序"}
  ]
}
```

**`/v1/completions`**（原始補全格式，適合程式碼模型）：
```json
{
  "model": "local",
  "prompt": "def quicksort(arr):",
  "max_tokens": 200,
  "temperature": 0.2
}
```

---

## 常見症狀與原因

| 症狀 | 可能原因 |
|------|---------|
| 輸出包含原始範本標籤 | 模型用了錯誤的範本，停止 token 未生效 |
| 回覆重複使用者輸入 | 對話範本格式錯誤 |
| 生成停在奇怪的地方 | 錯誤的停止 token 被觸發 |

---

## 延伸閱讀

- [故障排查指南](TROUBLESHOOTING.md) — 生成品質問題的系統排查
- [量化格式詳解](QUANTIZATION.md) — 模型格式與相容性
- [GPU 推論管線](GPU_PIPELINE.md) — Token 採樣鏈路
