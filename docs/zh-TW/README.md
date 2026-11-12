# Shimmy — 本機推論與 OpenAI 相容 API

**語言：** [English](../../README.md) · [简体中文](../zh-CN/README.md) · **繁體中文**

> **目前已發布：Shimmy 2.6.7 / Airframe 0.4.6（2026-11-12）。** Apple M1（虛擬）Metal 修正已包含於正式發行版；實體 M4 與 Qwen3.5 推論尚未認證。歷史 2.6.6 不含此修正。

Shimmy 是本機 GGUF 推論伺服器與命令列工具，使用 **Airframe 0.4.6** Rust/WGSL GPU 推論引擎。僅提供明確限定的 OpenAI 相容介面，而非完整 OpenAI API 替代品。

**目前版本：Shimmy 2.6.7 / Airframe 0.4.6，已發布。** 以下安裝指令會安裝包含 Metal 修正的正式版本。

## 安裝與執行

需要 Rust 工具鏈與適當 GPU 驅動：

```bash
cargo install shimmy --version 2.6.7 --locked
shimmy --version
shimmy --ctx 512 --kv-quant f32 serve --model-path /path/to/model.gguf --bind 127.0.0.1:11435
```

請使用[精確支援清單](../SUPPORTED_MODELS.md)中經驗證的 GGUF 模型。原生下載、Homebrew、Scoop 和版本化 GHCR 請參閱[安裝指引](../BINARY_INSTALL.md)。Docker `latest` 標籤未隨 2.6.7 更新；沒有 npm 發行。僅使用 `fast` 建置設定無法執行推論。

## 模型、精度及平台界限

支援範圍同時取決於模型檔 SHA、量化方式、上下文長度、執行模式及硬體平台。現行發布版的主要數值證據來自 NVIDIA/WSL Vulkan，與相同 token ID、反量化 FP32 參考完整 logits 比較：最大絕對誤差 **0.1**、RMS **0.03**，且貪婪解碼決策一致。

已發布的 Airframe 0.4.6 在 **Apple M1（Virtual）、Apple Paravirtual Metal、macOS 15.8.2** 完成 **198/198** 次完整 logits 數值比較；僅代表精確驗證的模型與設定，不代表所有 Apple GPU 均已認證。**實體 M4：NOT_RUN**。歷史 8K 間歇性停滯原因尚未查明，watchdog 清理問題仍待修復。

已驗證路徑限於 F32 KV 與參考核心。INT4/TurboShimmy、實驗核心及尚未啟用的 Gemma2、MoE、MLA、循環／狀態空間及多模態執行器不在此範圍。能解析模型中繼資料不等於模型推論已獲認證。亦不保證通用速度、穩定性或多 GPU 行為。

## HTTP 與安全性

支援指定範圍的模型清單、JSON 聊天／文字補全，以及**僅聊天**的 SSE。聊天預設回傳 JSON；不支援文字串流補全。Token 使用量、停止原因與錯誤語義參閱 [API](../API.md) 與[相容性](../OPENAI_COMPAT.md)文件。

本機 loopback 連線可不設定金鑰；**非 loopback 綁定必須提供 `SHIMMY_API_KEY`**。瀏覽器來源另需設定 `SHIMMY_ALLOWED_ORIGINS`。遠端部署應搭配 TLS 與存取控制。參閱[快速開始](../quickstart.md)、[設定](../CONFIGURATION.md)和[安全政策](../../SECURITY.md)。

## 授權及依賴關係

Shimmy 採 Apache-2.0；Airframe 與 airframe-observe 採 MIT；ShimmyTok 採 MIT **OR** Apache-2.0。libfse 0.1.4 另有 [Apache-2.0 補充授權](https://github.com/Michael-A-Kuykendall/airframe/blob/main/crates/libfse/LICENSE-GRANT-0.1.4.md)，允許商業使用、修改與再散布；仍須遵守第三方授權及聲明。詳見[授權說明](../../LICENSING.md)。

## 文件與貢獻

[英文 README](../../README.md) · [變更記錄](../../CHANGELOG.md) · [路線圖](../../ROADMAP.md) · [認證規則](../CERTIFICATION.md) · [支援模型](../SUPPORTED_MODELS.md) · [疑難排解](../TROUBLESHOOTING.md) · [貢獻指南](../../CONTRIBUTING.md)。

英文技術合約為權威版本；本翻譯不擴大其驗證範圍。

完整細節請見英文：[Install and run](../../README.md#install-and-run) · [Correctness and support](../../README.md#correctness-and-support) · [Server contract](../../README.md#server-contract) · [Documentation](../../README.md#documentation) · [Support](../../README.md#support) · [License](../../README.md#license--philosophy)