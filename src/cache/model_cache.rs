// Model-specific caching utilities
// Helper functions for extracting and caching model metadata

use super::{ModelFormat, ModelMetadata, TensorInfo};
use anyhow::{anyhow, Result};
use safetensors::SafeTensors;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Extract metadata from a SafeTensors model file
pub fn extract_safetensors_metadata(model_path: &Path) -> Result<ModelMetadata> {
    // Get file metadata
    let file_metadata = fs::metadata(model_path)?;
    let file_size = file_metadata.len();
    let modified_time = file_metadata
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    // Read only the header portion first (much faster than full file read)
    let model_data = fs::read(model_path)?;
    let tensors = SafeTensors::deserialize(&model_data)?;

    // Extract tensor information without loading the actual data
    let mut tensor_infos = Vec::new();
    for name in tensors.names() {
        if let Ok(tensor) = tensors.tensor(name) {
            tensor_infos.push(TensorInfo {
                name: name.to_string(),
                shape: tensor.shape().to_vec(),
                dtype: format!("{:?}", tensor.dtype()),
                offset: None, // SafeTensors doesn't expose offset directly
                size_bytes: Some(tensor.data().len() as u64),
            });
        }
    }

    // Load config.json if it exists
    let config = load_companion_json(model_path, "config.json")?;

    // Load tokenizer.json if it exists
    let tokenizer = load_companion_json(model_path, "tokenizer.json")?;

    Ok(ModelMetadata {
        model_path: model_path.to_path_buf(),
        file_size,
        modified_time,
        format: ModelFormat::SafeTensors,
        tensors: tensor_infos,
        config,
        tokenizer,
    })
}

/// Extract metadata from a GGUF model file (placeholder for future implementation)
pub fn extract_gguf_metadata(model_path: &Path) -> Result<ModelMetadata> {
    let file_metadata = fs::metadata(model_path)?;
    let file_size = file_metadata.len();
    let modified_time = file_metadata
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    // TODO: Implement GGUF header parsing
    // For now, return minimal metadata
    Ok(ModelMetadata {
        model_path: model_path.to_path_buf(),
        file_size,
        modified_time,
        format: ModelFormat::GGUF,
        tensors: Vec::new(), // TODO: Parse GGUF tensors
        config: None,
        tokenizer: None,
    })
}

/// Load companion JSON file (config.json, tokenizer.json, etc.)
fn load_companion_json(model_path: &Path, filename: &str) -> Result<Option<serde_json::Value>> {
    let json_path = model_path.with_file_name(filename);

    if json_path.exists() {
        let json_data = fs::read_to_string(&json_path)?;
        let parsed: serde_json::Value = serde_json::from_str(&json_data)?;
        Ok(Some(parsed))
    } else {
        Ok(None)
    }
}

/// Fast metadata extraction that only reads what's necessary
pub fn fast_extract_metadata(model_path: &Path) -> Result<ModelMetadata> {
    // Determine format by extension
    if let Some(ext) = model_path.extension().and_then(|s| s.to_str()) {
        match ext.to_lowercase().as_str() {
            "safetensors" => extract_safetensors_metadata(model_path),
            "gguf" => extract_gguf_metadata(model_path),
            _ => Err(anyhow!("Unsupported model format: {}", ext)),
        }
    } else {
        Err(anyhow!("Could not determine model format from path"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_companion_json() {
        // This would need actual test files to run
        // For now, just test the basic structure
        let test_path = PathBuf::from("nonexistent.safetensors");
        let result = load_companion_json(&test_path, "config.json");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
