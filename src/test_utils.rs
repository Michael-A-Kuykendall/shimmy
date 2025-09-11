// Test utilities for shimmy
use anyhow::Result;
use std::path::Path;

/// Create a test SafeTensors file with given data
pub fn create_test_safetensors(path: &str, data: &[u8]) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow::anyhow!("Path cannot be empty"));
    }

    let path_obj = Path::new(path);

    // Check if path is valid and parent directory exists
    if let Some(parent) = path_obj.parent() {
        if !parent.exists() {
            return Err(anyhow::anyhow!(
                "Parent directory does not exist: {:?}",
                parent
            ));
        }
    }

    // For now, just create a minimal safetensors file structure
    // In a real implementation, this would use the safetensors format
    std::fs::write(path, data).map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;

    Ok(())
}
