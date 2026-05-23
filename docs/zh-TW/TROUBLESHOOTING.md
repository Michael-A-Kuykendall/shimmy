# 故障排查指南

本文件涵蓋執行 Shimmy 時最常見的問題及其診斷方法和修復步驟。

---

## 快速診斷

遇到問題時，先執行以下指令獲取基本資訊：

```bash
# 檢查伺服器狀態
curl -s http://127.0.0.1:11435/api/health

# 查看 GPU 適配器
shimmy gpu-info

# 列出已探索的模型
shimmy list

# 啟用崩潰堆疊追蹤後重新執行
RUST_BACKTRACE=1 SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve
```

---

## 連接埠衝突

**症狀**：伺服器無法啟動，報 `address already in use` 錯誤。

**Linux / macOS：**

```bash
lsof -i :11435
kill <PID>
```

**Windows：**

```powershell
netstat -ano | findstr :8080
taskkill /PID <PID> /F
```

**永久解決方案**：使用 `SHIMMY_PORT` 環境變數指定其他連接埠：

```bash
SHIMMY_PORT=12000 shimmy serve
```

---

## 模型未找到

**症狀**：`shimmy list` 顯示空清單，或啟動時報 `model not found` 錯誤。

Shimmy 自動掃描以下目錄：

1. `SHIMMY_BASE_GGUF` 或 `LIBSHIMMY_MODEL_PATH` 指定的路徑
2. `~/.cache/huggingface/hub/`
3. `~/.ollama/models/`
4. `~/lm-studio/models/`
5. `~/.cache/lm-studio/models/`
6. `~/Library/Application Support/LMStudio/`（macOS）

```bash
# 明確指定模型路徑
SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve

# 查看實際搜尋了哪些路徑
shimmy discover
```

---

## GPU 適配器錯誤

### Windows — Direct3D 12

```powershell
# 確認 DirectX 12 可用
dxdiag
shimmy gpu-info
```

**常見原因**：驅動版本過舊；Windows N 版本缺少 DirectX 元件（需安裝媒體功能套件）；虛擬機器中 D3D12 不支援。

### Linux — Vulkan

```bash
vulkaninfo | head -20

# NVIDIA
sudo apt install nvidia-vulkan-icd

# AMD
sudo apt install mesa-vulkan-drivers

# Intel
sudo apt install intel-media-va-driver mesa-vulkan-drivers
```

### macOS — Metal

確認 macOS 版本 ≥ 11（Big Sur）。Gatekeeper 封鎖問題：

```bash
xattr -d com.apple.quarantine /usr/local/bin/shimmy
```

---

## 顯存不足（OOM）

### WebGPU 單緩衝區 2 GB 上限

WebGPU 規格限制單個緩衝區最大為 2 GB，影響部分模型的**輸出嵌入矩陣**。

**已知受影響的模型：**

| 模型 | 原因 | 狀態 |
|------|------|------|
| Gemma-2-2B Q4_K_M | 詞表 256K，輸出頭矩陣 2.19 GB | 暫不支援 |

### 上下文長度導致的 OOM

```bash
SHIMMY_MAX_CTX=2048 shimmy serve  # 從最小值開始
SHIMMY_MAX_CTX=4096 shimmy serve  # 逐步增加
```

---

## 推論問題

### 輸出在奇怪的位置截斷

最可能的原因是對話範本識別錯誤。詳見[對話範本參考](CHAT_TEMPLATES.md)。

### 輸出亂碼或重複

1. 使用了錯誤的對話範本
2. `temperature` 過高（嘗試設為 0.0 測試確定性輸出）
3. 上下文超出模型能力（嘗試降低 `SHIMMY_MAX_CTX`）

### 生成速度非常慢

執行 `shimmy gpu-info` 確認正在使用 GPU。若顯示 `Software Rasterizer` 或 `llvmpipe`，代表在 CPU 上回退執行，速度會比 GPU 慢 10–50 倍。

---

## 已知不支援的模型

| 模型 | 問題 | 說明 |
|------|------|------|
| Phi-3 / Phi-3.5 系列 | 融合 QKV（`attn_qkv.weight`） | Airframe 期望獨立的 Q/K/V 張量 |
| Gemma-2-2B Q4_K_M | 輸出頭超過 2 GB | WebGPU 緩衝區上限限制 |
| Qwen3 系列 | 缺少 QK Norm 著色器 | 架構尚不支援 |
| Q2_K / Q3_K | 量化格式不支援 | 使用 Q4_K_M 或更高格式 |

---

## 使用 RUST_BACKTRACE 獲取詳細錯誤資訊

```bash
# Linux / macOS
RUST_BACKTRACE=1 SHIMMY_BASE_GGUF=/path/to/model.gguf shimmy serve

# Windows PowerShell
$env:RUST_BACKTRACE = "1"
$env:LIBSHIMMY_MODEL_PATH = "D:\models\model.gguf"
.\shimmy_server_gpu.exe
```

提交 Bug 報告時請附上完整的 backtrace 輸出。

---

## 提交 Bug 報告

前往 [GitHub Issues](https://github.com/Michael-A-Kuykendall/shimmy/issues) 提交，請包含：

1. 作業系統和版本
2. GPU 型號和驅動版本（`shimmy gpu-info` 的輸出）
3. 模型檔案名稱和量化格式
4. 完整錯誤訊息
5. 啟用 `RUST_BACKTRACE=1` 後的完整輸出

---

## 延伸閱讀

- [對話範本參考](CHAT_TEMPLATES.md) — 範本識別錯誤的排查
- [量化格式詳解](QUANTIZATION.md) — 不支援的量化格式
- [擴展上下文視窗](EXTENDED_CONTEXT.md) — 上下文相關的 OOM
- [GPU 推論管線](GPU_PIPELINE.md) — 底層架構

---

> 💝 **如果 Shimmy 對您有幫助，歡迎[贊助支持](https://github.com/sponsors/Michael-A-Kuykendall)——所有款項 100% 用於保持專案永久免費。**
