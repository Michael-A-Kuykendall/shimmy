use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

pub struct ReadImageTool;

// ── MIME detection via magic bytes ───────────────────────────────────────────

fn detect_mime(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 4 && bytes[..4] == [0x89, 0x50, 0x4e, 0x47] {
        return "image/png";
    }
    if bytes.len() >= 2 && bytes[..2] == [0xff, 0xd8] {
        return "image/jpeg";
    }
    if bytes.len() >= 3 && bytes[..3] == [0x47, 0x49, 0x46] {
        return "image/gif";
    }
    // WEBP: "RIFF" at 0..4, "WEBP" at 8..12
    if bytes.len() >= 12
        && bytes[..4] == [0x52, 0x49, 0x46, 0x46]
        && &bytes[8..12] == b"WEBP"
    {
        return "image/webp";
    }
    "application/octet-stream"
}

// ── PNG dimension extraction (no `image` crate) ──────────────────────────────
// PNG signature: 8 bytes, then IHDR chunk (4 len + 4 type + 13 data)
// Width at bytes 16..20, height at bytes 20..24 (big-endian u32)

fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    if bytes.len() < 24 {
        return (0, 0);
    }
    // Verify PNG signature
    if bytes[..4] != [0x89, 0x50, 0x4e, 0x47] {
        return (0, 0);
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    (width, height)
}

// ── Tool implementation ───────────────────────────────────────────────────────

#[async_trait]
impl Tool for ReadImageTool {
    fn name(&self) -> &str {
        "read_image"
    }

    fn description(&self) -> &str {
        "Read an image file and return its contents as base64 with MIME type and dimensions. \
         Modes: 'base64' (default) returns full data, 'meta' returns metadata only, \
         'ocr' returns base64 plus an OCR placeholder."
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the image file"
                },
                "mode": {
                    "type": "string",
                    "enum": ["base64", "meta", "ocr"],
                    "description": "Output mode: 'base64' (default), 'meta' (no base64), 'ocr' (placeholder)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let mode = args.get_str("mode").unwrap_or("base64");

        let file_path = Path::new(path);
        if !file_path.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "File not found: {}",
                path
            )));
        }

        let bytes = std::fs::read(file_path)?;
        let file_size = bytes.len();
        let mime = detect_mime(&bytes);
        let (width, height) = if mime == "image/png" {
            png_dimensions(&bytes)
        } else {
            (0, 0)
        };

        match mode {
            "meta" => {
                let output = format!(
                    "path: {}\nmime: {}\nsize: {} bytes\ndimensions: {}x{}",
                    path, mime, file_size, width, height
                );
                let data = serde_json::json!({
                    "path": path,
                    "mime": mime,
                    "size_bytes": file_size,
                    "width": width,
                    "height": height,
                });
                Ok(ToolResult::success_with_data(output, data))
            }
            "ocr" => {
                let b64 = base64_encode(&bytes);
                let output = format!(
                    "path: {}\nmime: {}\nsize: {} bytes\ndimensions: {}x{}\nocr: OCR not yet implemented",
                    path, mime, file_size, width, height
                );
                let data = serde_json::json!({
                    "path": path,
                    "mime": mime,
                    "size_bytes": file_size,
                    "width": width,
                    "height": height,
                    "base64": b64,
                    "ocr_text": null,
                    "ocr_status": "not_implemented",
                });
                Ok(ToolResult::success_with_data(output, data))
            }
            _ => {
                // "base64" (default) and anything unrecognised
                let b64 = base64_encode(&bytes);
                let output = format!(
                    "path: {}\nmime: {}\nsize: {} bytes\ndimensions: {}x{}\nbase64: {} chars",
                    path, mime, file_size, width, height, b64.len()
                );
                let data = serde_json::json!({
                    "path": path,
                    "mime": mime,
                    "size_bytes": file_size,
                    "width": width,
                    "height": height,
                    "base64": b64,
                });
                Ok(ToolResult::success_with_data(output, data))
            }
        }
    }
}

// ── base64 encoding (stdlib only, avoids the `base64` crate) ─────────────────
// Standard base64 alphabet, padded.

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let combined = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((combined >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((combined >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((combined >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(combined & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
