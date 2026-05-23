<div align="center">

# Shimmy 中文文件中心

### 輕量本地 AI 推論伺服器 · 相容 OpenAI API · 永久免費

[简体中文](../zh-CN/README.md) · **繁體中文** · [English](../../README.md)

[![GitHub Stars](https://img.shields.io/github/stars/Michael-A-Kuykendall/shimmy?style=social)](https://github.com/Michael-A-Kuykendall/shimmy/stargazers)
[![💝 贊助專案](https://img.shields.io/badge/💝_贊助專案-ea4aaa?style=flat&logo=github&logoColor=white)](https://github.com/sponsors/Michael-A-Kuykendall)

</div>

---

## 歡迎使用 Shimmy

Shimmy 是一個以純 **Rust** 撰寫的本地 AI 推論伺服器，相容 **OpenAI API**，無須連接雲端，無訂閱費用，**永久免費**。

其核心引擎 **Airframe** 透過 WebGPU（WGSL 計算著色器）直接在您的 GPU 上執行推論——無需 CUDA、無需 Python、無需 C++ 工具鏈。只需一個執行檔，30 秒內即可執行您的第一個本地模型。

---

## � 支持 Shimmy 的發展

🚀 **如果 Shimmy 對您有幫助，歡迎[贊助支持](https://github.com/sponsors/Michael-A-Kuykendall)——所有贊助款項 100% 用於保持專案永久免費。**

- **$5/月**：咖啡檔 ☕ 永久感謝 + 贊助者徽章
- **$25/月**：Bug 優先處理檔 🐛 優先支援 + 名字收錄於 [SPONSORS.md](../../SPONSORS.md)
- **$100/月**：企業支援檔 🏢 Logo 展示 + 每月答疑
- **$500/月**：基礎設施合作檔 🚀 直接支援 + 路線圖參與

[**🎯 成為贊助者**](https://github.com/sponsors/Michael-A-Kuykendall) | 查看[贊助者名單](../../SPONSORS.md) 🙏

---

## �📚 文件索引

### 入門指南

| 文件 | 說明 |
|------|------|
| [使用者手冊（完整版）](../USER_MANUAL.zh-TW.md) | 從安裝到 API 的完整繁體中文手冊（19 個章節，1200+ 行） |
| [快速入門](../quickstart.md) | 5 分鐘跑起來（英文，簡單直接） |
| [從 v1.x 遷移](../MIGRATION_v2.md) | v1 升 v2 的變更與遷移指南（英文） |

### 核心技術深度解析

以下文件深入說明 Shimmy 和 Airframe 引擎的內部機制，適合開發者和進階使用者：

| 文件 | 說明 |
|------|------|
| [量化格式詳解](QUANTIZATION.md) | Q4_0、Q8_0、K-quant 的位元級原理，GPU 著色器如何在矩陣乘法中即時反量化 |
| [擴展上下文視窗](EXTENDED_CONTEXT.md) | YaRN RoPE 縮放原理，超長上下文的顯存計算，各型號配置參考 |
| [故障排查指南](TROUBLESHOOTING.md) | GPU 錯誤、模型載入失敗、連接埠衝突、OOM 等問題的診斷與修復 |
| [對話範本參考](CHAT_TEMPLATES.md) | ChatML / Llama-3 / OpenChat 三大範本族，自動識別邏輯，停止 token 說明 |
| [GPU 推論管線](GPU_PIPELINE.md) | 無綁定架構、預填充分塊、KV 快取布局、採樣鏈路全解析 |

### API 與整合

| 文件 | 說明 |
|------|------|
| [API 參考](../API.md) | 完整的 HTTP 介面文件（英文） |
| [OpenAI 相容矩陣](../OPENAI_COMPAT.md) | 哪些 OpenAI 參數已支援、哪些未支援（英文） |
| [整合範例](../INTEGRATION.md) | LangChain、OpenAI SDK、VSCode 等整合方法（英文） |

### 模型相關

| 文件 | 說明 |
|------|------|
| [模型擴展協議](../MODEL_EXPANSION.md) | 如何為新模型架構新增支援（英文） |
| [跨平台編譯](../CROSS_COMPILATION.md) | 為 ARM、Linux、Windows 交叉編譯（英文） |

---

## ⚡ 30 秒快速開始

```bash
# 1. 下載模型（以 TinyLlama 為例）
# 前往 https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF
# 下載 TinyLlama-1.1B-Chat-v1.0.Q4_0.gguf

# 2. 啟動伺服器
SHIMMY_BASE_GGUF=/path/to/TinyLlama-1.1B-Chat-v1.0.Q4_0.gguf ./shimmy serve

# 3. 發送請求
curl http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "local",
    "messages": [{"role": "user", "content": "你好！請介紹一下你自己。"}],
    "max_tokens": 128
  }'
```

就這樣。無需設定檔，無需資料庫，無需 Docker。

---

## 🖥️ 已驗證的 GPU 支援

| 平台 | GPU 類型 | 後端 |
|------|---------|------|
| Windows | NVIDIA / AMD / Intel | Direct3D 12 |
| Linux | NVIDIA / AMD / Intel | Vulkan |
| macOS | Apple Silicon / AMD | Metal |
| 任意平台 | 獨顯/內顯 | 軟體回退（慢） |

詳見[故障排查指南](TROUBLESHOOTING.md)中的 GPU 專項章節。

---

## 🤖 支援的模型格式

Shimmy 載入 **GGUF 格式**模型，支援以下量化類型：

| 格式 | 每權重位元數 | 品質 | 推薦 |
|------|-----------|------|------|
| Q4_K_M | 4.5 位元 | ★★★★☆ | ✅ 首選 |
| Q4_0 | 4 位元 | ★★★☆☆ | ✅ 相容性最佳 |
| Q8_0 | 8 位元 | ★★★★★ | ✅ 最高品質 |
| Q5_K_M | 5.5 位元 | ★★★★☆ | ✅ 品質/大小平衡 |

詳見[量化格式詳解](QUANTIZATION.md)。

---

## 💬 常見問題

**Shimmy 和 Ollama 有什麼差別？**
Shimmy 是純 Rust 實作，無 Python 執行環境，無 C++ 相依，啟動時間不到 100ms。Airframe 引擎透過 WebGPU 而非 CUDA 直接驅動 GPU。

**如何選擇模型量化格式？**
日常使用首選 `Q4_K_M`——在檔案大小和推論品質之間取得了最好的平衡。若追求最高品質且顯存充足，選 `Q8_0`。詳見[量化格式詳解](QUANTIZATION.md)。

**上下文長度不夠怎麼辦？**
設定 `SHIMMY_MAX_CTX=8192`（或更高）即可，Airframe 會自動套用 YaRN RoPE 縮放。注意超出模型原生上下文 2 倍以上時品質會有所下降。詳見[擴展上下文視窗](EXTENDED_CONTEXT.md)。

**啟動報 GPU 錯誤怎麼辦？**
執行 `shimmy gpu-info` 查看 GPU 適配器狀態，然後參考[故障排查指南](TROUBLESHOOTING.md)。

**模型生成內容在奇怪的地方截斷？**
可能是對話範本識別錯誤。參見[對話範本參考](CHAT_TEMPLATES.md)了解如何手動指定範本。

---

## 💝 支援專案

**Shimmy 將永久免費。** 如果它對您有幫助，歡迎贊助，支持持續開發：

| 贊助層級 | 金額 | 權益 |
|---------|------|------|
| ☕ 咖啡支持者 | $5/月 | 永久感謝 + 贊助徽章 |
| 🐛 Bug 優先處理 | $25/月 | 優先回應 + 名字列入 SPONSORS.md |
| 🏢 企業支持者 | $100/月 | Logo 展示 + 每月答疑 |
| 🚀 基礎設施合作夥伴 | $500/月 | 直接支持 + roadmap 決策權 |

[**💝 成為贊助者**](https://github.com/sponsors/Michael-A-Kuykendall) · [查看贊助者名單](../../SPONSORS.md)

---

## 🐛 問題回報與社群

- **提交 Bug**：[GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues)
- **參與討論**：[GitHub Discussions](https://github.com/Michael-A-Kuykendall/shimmy/discussions)
- **查看更新日誌**：[CHANGELOG.md](../../CHANGELOG.md)

歡迎提交 Pull Request。貢獻指南見 [CONTRIBUTING.md](../../CONTRIBUTING.md)。

---

*本文件與主倉庫同步維護。發現問題或有改進建議，歡迎[提交 Issue](https://github.com/Michael-A-Kuykendall/shimmy/issues)。*

*[简体中文文档中心](../zh-CN/README.md) · [English Documentation](../../README.md)*
