#![allow(dead_code)]

use anyhow::{anyhow, Result};
use safetensors::SafeTensors;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

/// Convert SafeTensors LoRA adapter to a temporary GGUF file that llama.cpp can load
pub fn convert_safetensors_to_gguf(safetensors_path: &Path) -> Result<PathBuf> {
    info!(path=%safetensors_path.display(), "Converting SafeTensors LoRA to GGUF format");

    // Read the SafeTensors file
    let data = fs::read(safetensors_path)?;
    let tensors = SafeTensors::deserialize(&data)?;

    debug!("SafeTensors contains {} tensors", tensors.len());

    let safetensors_dir = safetensors_path
        .parent()
        .ok_or_else(|| anyhow!("Invalid SafeTensors path"))?;

    // Look for adapter_model.gguf in the same directory
    let gguf_path = safetensors_dir.join("adapter_model.gguf");
    if gguf_path.exists() {
        info!(path=%gguf_path.display(), "Found existing GGUF adapter");
        return Ok(gguf_path);
    }

    // Look for any .gguf file in the same directory
    if let Ok(entries) = fs::read_dir(safetensors_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                info!(path=%path.display(), "Found GGUF adapter in same directory");
                return Ok(path);
            }
        }
    }

    // Try to find llama.cpp conversion script
    let possible_conversion_scripts = [
        "convert-lora-to-ggml.py",
        "convert_lora_to_ggml.py",
        "convert-hf-to-gguf.py",
        "convert_hf_to_gguf.py",
    ];

    for script_name in &possible_conversion_scripts {
        if let Some(script_path) = find_llama_cpp_script(script_name) {
            info!(script=%script_path.display(), "Found conversion script, attempting conversion");

            match run_conversion_script(&script_path, safetensors_path, &gguf_path) {
                Ok(_) => {
                    if gguf_path.exists() {
                        info!(path=%gguf_path.display(), "Successfully converted SafeTensors to GGUF");
                        return Ok(gguf_path);
                    }
                }
                Err(e) => {
                    warn!(error=%e, "Conversion script failed");
                }
            }
        }
    }

    // Try to create a temporary GGUF file with a simple format conversion
    // This is a simplified approach - in a full implementation, we'd need to properly
    // convert the tensor formats and metadata
    warn!("No llama.cpp conversion scripts found, providing guidance");

    Err(anyhow!(
        "SafeTensors to GGUF conversion needed for: {}\n\
        \n\
        Shimmy detected a SafeTensors LoRA adapter but requires GGUF format.\n\
        \n\
        To enable this adapter:\n\
        1. Install llama.cpp with Python bindings\n\
        2. Run conversion: python /path/to/llama.cpp/convert-lora-to-ggml.py {} {}\n\
        3. Or place a pre-converted .gguf file in: {}\n\
        \n\
        This is the shim functionality - shimmy bridges SafeTensors adapters to GGUF-based inference.",
        safetensors_path.display(),
        safetensors_path.display(),
        gguf_path.display(),
        safetensors_dir.display()
    ))
}

fn find_llama_cpp_script(script_name: &str) -> Option<PathBuf> {
    // Common locations where llama.cpp might be installed
    let possible_paths = [
        format!("/usr/local/bin/{}", script_name),
        format!("/usr/bin/{}", script_name),
        format!("./llama.cpp/{}", script_name),
        format!("../llama.cpp/{}", script_name),
        format!("../../llama.cpp/{}", script_name),
        format!("C:/llama.cpp/{}", script_name),
        format!("C:/Users/*/llama.cpp/{}", script_name),
    ];

    for path in &possible_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // Try to find it in PATH
    if let Ok(output) = Command::new("which").arg(script_name).output() {
        if output.status.success() {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path = PathBuf::from(path_str.trim());
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

fn run_conversion_script(script_path: &Path, input_path: &Path, output_path: &Path) -> Result<()> {
    info!(script=%script_path.display(), input=%input_path.display(), output=%output_path.display(), 
          "Running LoRA conversion script");

    let output = Command::new("python")
        .arg(script_path)
        .arg(input_path)
        .arg(output_path)
        .output()
        .map_err(|e| anyhow!("Failed to run conversion script: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Conversion script failed: {}", stderr));
    }

    Ok(())
}

/// Check if a path is a SafeTensors file
pub fn is_safetensors_file(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str()) == Some("safetensors")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_is_safetensors_file() {
        assert!(is_safetensors_file(Path::new("adapter_model.safetensors")));
        assert!(!is_safetensors_file(Path::new("model.gguf")));
        assert!(!is_safetensors_file(Path::new("config.json")));
        assert!(!is_safetensors_file(Path::new("file.txt")));
        assert!(!is_safetensors_file(Path::new("file")));
        assert!(!is_safetensors_file(Path::new("file.")));
        assert!(!is_safetensors_file(Path::new(".safetensors")));
    }

    #[test]
    fn test_convert_safetensors_to_gguf_invalid_path() {
        // Test with non-existent file
        let result = convert_safetensors_to_gguf(Path::new("non_existent.safetensors"));
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_safetensors_to_gguf_invalid_safetensors() {
        let temp_dir = TempDir::new().unwrap();
        let safetensors_path = temp_dir.path().join("invalid.safetensors");

        // Create an invalid safetensors file
        let mut file = fs::File::create(&safetensors_path).unwrap();
        writeln!(file, "invalid safetensors data").unwrap();

        let result = convert_safetensors_to_gguf(&safetensors_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_safetensors_to_gguf_with_existing_adapter_model() {
        let temp_dir = TempDir::new().unwrap();
        let safetensors_path = temp_dir.path().join("adapter.safetensors");
        let gguf_path = temp_dir.path().join("adapter_model.gguf");

        // Create a minimal valid safetensors file
        let data = create_minimal_safetensors();
        fs::write(&safetensors_path, &data).unwrap();

        // Create existing adapter_model.gguf
        fs::write(&gguf_path, b"fake gguf data").unwrap();

        let result = convert_safetensors_to_gguf(&safetensors_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), gguf_path);
    }

    #[test]
    fn test_convert_safetensors_to_gguf_with_existing_gguf() {
        let temp_dir = TempDir::new().unwrap();
        let safetensors_path = temp_dir.path().join("adapter.safetensors");
        let gguf_path = temp_dir.path().join("some_model.gguf");

        // Create a minimal valid safetensors file
        let data = create_minimal_safetensors();
        fs::write(&safetensors_path, &data).unwrap();

        // Create existing .gguf file
        fs::write(&gguf_path, b"fake gguf data").unwrap();

        let result = convert_safetensors_to_gguf(&safetensors_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), gguf_path);
    }

    #[test]
    fn test_convert_safetensors_to_gguf_no_existing_gguf() {
        let temp_dir = TempDir::new().unwrap();
        let safetensors_path = temp_dir.path().join("adapter.safetensors");

        // Create a minimal valid safetensors file
        let data = create_minimal_safetensors();
        fs::write(&safetensors_path, &data).unwrap();

        // No existing GGUF files
        let result = convert_safetensors_to_gguf(&safetensors_path);
        assert!(result.is_err());

        // Should contain helpful error message
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("SafeTensors to GGUF conversion needed"));
        assert!(error_msg.contains("shimmy bridges SafeTensors"));
    }

    #[test]
    fn test_convert_safetensors_path_without_parent() {
        // Test with a path that has no parent (should be impossible in practice, but test edge case)
        let result = convert_safetensors_to_gguf(Path::new(""));
        assert!(result.is_err());
    }

    #[test]
    fn test_find_llama_cpp_script_not_found() {
        let result = find_llama_cpp_script("definitely_does_not_exist_script.py");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_llama_cpp_script_with_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("convert-lora-to-ggml.py");
        fs::write(&script_path, b"#!/usr/bin/env python\nprint('test')").unwrap();

        // This test checks the logic but won't find the temp file since it's not in the hardcoded paths
        // The real test is that the function doesn't crash with various inputs
        let result = find_llama_cpp_script("convert-lora-to-ggml.py");
        // Can't assert on the result since it depends on system state, but function should not panic
        let _ = result;
    }

    #[test]
    fn test_run_conversion_script_python_not_found() {
        // This will fail because we're using a non-existent python executable
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("script.py");
        let input_path = temp_dir.path().join("input.safetensors");
        let output_path = temp_dir.path().join("output.gguf");

        fs::write(&script_path, b"print('test')").unwrap();
        fs::write(&input_path, b"fake input").unwrap();

        // This should fail since python command might not exist or script will fail
        let result = run_conversion_script(&script_path, &input_path, &output_path);
        // In most environments this will fail, which is expected behavior
        let _ = result;
    }

    #[test]
    fn test_safetensors_deserialization_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let safetensors_path = temp_dir.path().join("corrupt.safetensors");

        // Create a file with invalid safetensors format
        fs::write(&safetensors_path, b"not valid safetensors format").unwrap();

        let result = convert_safetensors_to_gguf(&safetensors_path);
        assert!(result.is_err());

        // Error should be related to deserialization
        let error_msg = result.unwrap_err().to_string();
        // The actual error message will come from safetensors library
        assert!(!error_msg.is_empty());
    }

    #[test]
    fn test_directory_read_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let safetensors_path = temp_dir.path().join("adapter.safetensors");

        // Create a minimal valid safetensors file
        let data = create_minimal_safetensors();
        fs::write(&safetensors_path, &data).unwrap();

        // Remove the temp directory to simulate read error (this is tricky to test)
        // Instead, we'll test with a valid directory but no GGUF files
        let result = convert_safetensors_to_gguf(&safetensors_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_gguf_files_in_directory() {
        let temp_dir = TempDir::new().unwrap();
        let safetensors_path = temp_dir.path().join("adapter.safetensors");
        let gguf_path1 = temp_dir.path().join("model1.gguf");
        let gguf_path2 = temp_dir.path().join("model2.gguf");

        // Create a minimal valid safetensors file
        let data = create_minimal_safetensors();
        fs::write(&safetensors_path, &data).unwrap();

        // Create multiple GGUF files
        fs::write(&gguf_path1, b"fake gguf data 1").unwrap();
        fs::write(&gguf_path2, b"fake gguf data 2").unwrap();

        let result = convert_safetensors_to_gguf(&safetensors_path);
        assert!(result.is_ok());

        // Should return one of the GGUF files (order may vary)
        let returned_path = result.unwrap();
        assert!(returned_path == gguf_path1 || returned_path == gguf_path2);
        assert!(returned_path.extension().unwrap() == "gguf");
    }

    // Helper function to create a minimal valid SafeTensors file
    fn create_minimal_safetensors() -> Vec<u8> {
        // Create a minimal valid SafeTensors format
        // SafeTensors format: 8-byte header (length) + JSON metadata + tensor data
        let metadata = r#"{"test_tensor":{"dtype":"F32","shape":[1,1],"data_offsets":[0,4]}}"#;
        let metadata_bytes = metadata.as_bytes();
        let metadata_len = metadata_bytes.len() as u64;

        let mut data = Vec::new();
        data.extend_from_slice(&metadata_len.to_le_bytes());
        data.extend_from_slice(metadata_bytes);

        // Add minimal tensor data (4 bytes for a single F32)
        data.extend_from_slice(&[0u8, 0u8, 0u8, 0u8]);

        data
    }
}
