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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_returns_error() {
        let err = create_test_safetensors("", &[]).unwrap_err();
        assert!(err.to_string().contains("Path cannot be empty"));
    }

    #[test]
    fn missing_parent_directory_returns_error() {
        let bad = std::env::temp_dir()
            .join("does-not-exist-xyz")
            .join("model.safetensors");
        let err = create_test_safetensors(bad.to_str().unwrap(), b"data").unwrap_err();
        assert!(err.to_string().contains("Parent directory does not exist"));
    }

    #[test]
    fn writes_file_successfully() {
        let dir = std::env::temp_dir().join(format!("st-utils-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("model.safetensors");
        let s = path.to_str().unwrap().to_string();

        create_test_safetensors(&s, b"hello-world").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"hello-world");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
