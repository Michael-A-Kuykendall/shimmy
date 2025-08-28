use anyhow::{anyhow, Result};
use safetensors::SafeTensors;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, debug, warn};

/// Convert SafeTensors LoRA adapter to a temporary GGUF file that llama.cpp can load
pub fn convert_safetensors_to_gguf(safetensors_path: &Path) -> Result<PathBuf> {
    info!(path=%safetensors_path.display(), "Converting SafeTensors LoRA to GGUF format");
    
    // Read the SafeTensors file
    let data = fs::read(safetensors_path)?;
    let tensors = SafeTensors::deserialize(&data)?;
    
    debug!("SafeTensors contains {} tensors", tensors.len());
    
    let safetensors_dir = safetensors_path.parent()
        .ok_or_else(|| anyhow!("Invalid SafeTensors path"))?;
    
    // Look for adapter_model.gguf in the same directory
    let gguf_path = safetensors_dir.join("adapter_model.gguf");
    if gguf_path.exists() {
        info!(path=%gguf_path.display(), "Found existing GGUF adapter");
        return Ok(gguf_path);
    }
    
    // Look for any .gguf file in the same directory
    if let Ok(entries) = fs::read_dir(safetensors_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                    info!(path=%path.display(), "Found GGUF adapter in same directory");
                    return Ok(path);
                }
            }
        }
    }
    
    // Try to find llama.cpp conversion script
    let possible_conversion_scripts = [
        "convert-lora-to-ggml.py",
        "convert_lora_to_ggml.py", 
        "convert-hf-to-gguf.py",
        "convert_hf_to_gguf.py"
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
    
    #[test]
    fn test_is_safetensors_file() {
        assert!(is_safetensors_file(Path::new("adapter_model.safetensors")));
        assert!(!is_safetensors_file(Path::new("model.gguf")));
        assert!(!is_safetensors_file(Path::new("config.json")));
    }
}
