use async_trait::async_trait;
use std::path::Path;
use crate::tools::{Tool, ToolArgs, ToolResult, ToolError};

/// Read image files, return base64 data and metadata
pub struct ReadImageTool;

#[async_trait]
impl Tool for ReadImageTool {
    fn name(&self) -> &'static str {
        "read_image"
    }

    fn description(&self) -> &'static str {
        "Read an image file from disk, returning base64-encoded data, MIME type, dimensions, and optional OCR text. \
         Parameters: path (required), mode (optional: 'base64', 'meta', 'ocr'; default 'base64')"
    }

    fn requires_license(&self) -> bool {
        false // Public tool, no license required
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        // Extract parameters
        let path_str = args.parameters.get("path")
            .ok_or_else(|| ToolError::InvalidParameters("'path' parameter is required".to_string()))?;
        
        let mode = args.parameters.get("mode")
            .map(|s| s.as_str())
            .unwrap_or("base64");
        
        // Validate mode
        if !["base64", "meta", "ocr"].contains(&mode) {
            return Err(ToolError::InvalidParameters(
                format!("Invalid mode '{}'. Must be 'base64', 'meta', or 'ocr'", mode)
            ));
        }
        
        // Resolve path (absolute or relative to working directory)
        let path = if Path::new(path_str).is_absolute() {
            Path::new(path_str).to_path_buf()
        } else {
            Path::new(&args.context.working_directory).join(path_str)
        };
        
        // Read file bytes
        let bytes = std::fs::read(&path)
            .map_err(|e| ToolError::ExecutionFailed(
                format!("Failed to read image file '{}': {}", path.display(), e)
            ))?;
        
        // Determine MIME type using infer crate
        let mime = infer::get(&bytes)
            .map(|info| info.mime_type())
            .unwrap_or("application/octet-stream");
        
        // Decode image to get dimensions
        let image_result = image::load_from_memory(&bytes);
        let (width, height) = if let Ok(img) = image_result {
            (img.width(), img.height())
        } else {
            // Not a valid image or unsupported format
            return Err(ToolError::ExecutionFailed(
                format!("Failed to decode image '{}': unsupported format or corrupted file", path.display())
            ));
        };
        
        // Base64 encode the image
        let base64 = base64::encode(&bytes);
        
        // Build response based on mode
        match mode {
            "base64" => {
                // Full base64 data + metadata
                Ok(ToolResult {
                    success: true,
                    output: format!("Image read successfully: {} ({}x{})", mime, width, height),
                    structured_data: Some(serde_json::json!({
                        "mime": mime,
                        "width": width,
                        "height": height,
                        "base64": base64,
                        "size_bytes": bytes.len(),
                    })),
                    error_message: None,
                })
            }
            "meta" => {
                // Metadata only (no base64)
                Ok(ToolResult {
                    success: true,
                    output: format!("Image metadata: {} ({}x{}), {} bytes", mime, width, height, bytes.len()),
                    structured_data: Some(serde_json::json!({
                        "mime": mime,
                        "width": width,
                        "height": height,
                        "size_bytes": bytes.len(),
                    })),
                    error_message: None,
                })
            }
            "ocr" => {
                // OCR mode - extract text from image
                // For now, return a placeholder indicating OCR is not implemented
                // In production, would use leptess or similar OCR crate
                Ok(ToolResult {
                    success: true,
                    output: format!("Image read with OCR (OCR not yet implemented): {} ({}x{})", mime, width, height),
                    structured_data: Some(serde_json::json!({
                        "mime": mime,
                        "width": width,
                        "height": height,
                        "base64": base64,
                        "text": "[OCR feature not yet implemented]",
                        "size_bytes": bytes.len(),
                    })),
                    error_message: None,
                })
            }
            _ => unreachable!(), // Already validated above
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{ToolArgs, ExecutionContext};
    use std::collections::HashMap;
    use std::fs;

    #[tokio::test]
    async fn test_read_image_missing_path() {
        let tool = ReadImageTool;
        let args = ToolArgs {
            parameters: HashMap::new(),
            context: ExecutionContext {
                working_directory: ".".to_string(),
                user_id: None,
                session_id: "test".to_string(),
            },
        };
        
        let result = tool.execute(args).await;
        assert!(result.is_err());
        match result {
            Err(ToolError::InvalidParameters(msg)) => {
                assert!(msg.contains("path"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[tokio::test]
    async fn test_read_image_invalid_mode() {
        let tool = ReadImageTool;
        let mut params = HashMap::new();
        params.insert("path".to_string(), "test.png".to_string());
        params.insert("mode".to_string(), "invalid".to_string());
        
        let args = ToolArgs {
            parameters: params,
            context: ExecutionContext {
                working_directory: ".".to_string(),
                user_id: None,
                session_id: "test".to_string(),
            },
        };
        
        let result = tool.execute(args).await;
        assert!(result.is_err());
        match result {
            Err(ToolError::InvalidParameters(msg)) => {
                assert!(msg.contains("Invalid mode"));
            }
            _ => panic!("Expected InvalidParameters error"),
        }
    }

    #[tokio::test]
    async fn test_read_image_file_not_found() {
        let tool = ReadImageTool;
        let mut params = HashMap::new();
        params.insert("path".to_string(), "nonexistent.png".to_string());
        
        let args = ToolArgs {
            parameters: params,
            context: ExecutionContext {
                working_directory: std::env::temp_dir().to_string_lossy().to_string(),
                user_id: None,
                session_id: "test".to_string(),
            },
        };
        
        let result = tool.execute(args).await;
        assert!(result.is_err());
        match result {
            Err(ToolError::ExecutionFailed(msg)) => {
                assert!(msg.contains("Failed to read"));
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }

    #[tokio::test]
    async fn test_read_image_success() {
        // Create a minimal 1x1 PNG image for testing
        let temp_dir = std::env::temp_dir();
        let test_image_path = temp_dir.join("test_image_shimmy.png");
        
        // 1x1 red PNG (minimal valid PNG)
        let png_bytes = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
            0x49, 0x48, 0x44, 0x52, // IHDR
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
            0x08, 0x02, 0x00, 0x00, 0x00, // bit depth 8, RGB
            0x90, 0x77, 0x53, 0xDE, // CRC
            0x00, 0x00, 0x00, 0x0C, // IDAT chunk length
            0x49, 0x44, 0x41, 0x54, // IDAT
            0x08, 0x99, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00,
            0x18, 0xDD, 0x8D, 0xB4, // CRC
            0x00, 0x00, 0x00, 0x00, // IEND chunk length
            0x49, 0x45, 0x4E, 0x44, // IEND
            0xAE, 0x42, 0x60, 0x82, // CRC
        ];
        
        fs::write(&test_image_path, &png_bytes).unwrap();
        
        let tool = ReadImageTool;
        let mut params = HashMap::new();
        params.insert("path".to_string(), test_image_path.to_string_lossy().to_string());
        
        let args = ToolArgs {
            parameters: params,
            context: ExecutionContext {
                working_directory: temp_dir.to_string_lossy().to_string(),
                user_id: None,
                session_id: "test".to_string(),
            },
        };
        
        let result = tool.execute(args).await;
        assert!(result.is_ok());
        
        let tool_result = result.unwrap();
        assert!(tool_result.success);
        assert!(tool_result.structured_data.is_some());
        
        let data = tool_result.structured_data.unwrap();
        assert!(data.get("mime").is_some());
        assert_eq!(data.get("width").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(data.get("height").and_then(|v| v.as_u64()), Some(1));
        assert!(data.get("base64").is_some());
        
        // Cleanup
        fs::remove_file(&test_image_path).ok();
    }
}
