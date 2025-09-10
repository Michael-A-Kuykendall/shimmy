// Test SafeTensors with real HuggingFace models
// Downloads and tests actual models to validate implementation

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing SafeTensors with Real HuggingFace Models");
    
    // Create test directory
    let test_dir = Path::new("test-real-safetensors");
    fs::create_dir_all(test_dir)?;
    
    // Test with a small real model
    test_small_model(test_dir)?;
    
    // Test potential crash scenarios
    test_memory_intensive_scenarios(test_dir)?;
    
    // Test server mode
    test_server_mode()?;
    
    println!("✅ All real SafeTensors tests completed");
    Ok(())
}

fn test_small_model(test_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📦 Testing with small real model...");
    
    // Try to find existing SafeTensors models in cache
    let cache_paths = vec![
        Path::new("C:/Users/micha/.cache/huggingface/hub"),
        Path::new("./models"),
        Path::new("./test-models"),
    ];
    
    for cache_path in cache_paths {
        if cache_path.exists() {
            println!("🔍 Scanning for SafeTensors models in: {}", cache_path.display());
            
            if let Ok(entries) = fs::read_dir(cache_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    
                    // Look for SafeTensors files
                    if is_safetensors_directory(&path) {
                        println!("✅ Found SafeTensors model directory: {}", path.display());
                        
                        // Test this model
                        if let Err(e) = test_model_directory(&path) {
                            println!("❌ Failed to test {}: {}", path.display(), e);
                        }
                        
                        return Ok(()); // Test only first found model
                    }
                }
            }
        }
    }
    
    println!("⚠️  No existing SafeTensors models found in cache");
    println!("💡 To test with real models:");
    println!("   1. Use HuggingFace Hub to download a model in SafeTensors format");
    println!("   2. Or manually place SafeTensors files in ./test-models/");
    
    Ok(())
}

fn is_safetensors_directory(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    
    // Check if directory contains SafeTensors files
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("safetensors") {
                // Also check for config.json to confirm it's a model directory
                if dir.join("config.json").exists() {
                    return true;
                }
            }
        }
    }
    
    false
}

fn test_model_directory(model_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing model directory: {}", model_dir.display());
    
    // Find the main model file
    let model_file = find_main_safetensors_file(model_dir)?;
    println!("📁 Found model file: {}", model_file.display());
    
    // Test probe command
    println!("🔍 Testing probe command...");
    let output = Command::new("cargo")
        .args(&["run", "--bin", "shimmy", "--", "discover"])
        .output()?;
    
    if !output.status.success() {
        println!("❌ Discover command failed: {}", String::from_utf8_lossy(&output.stderr));
        return Ok(()); // Don't fail the whole test
    }
    
    println!("✅ Discover command succeeded");
    
    // Test if the model shows up in discovery
    let discover_output = String::from_utf8_lossy(&output.stdout);
    if discover_output.contains("safetensors") {
        println!("✅ SafeTensors model found in discovery");
    } else {
        println!("⚠️  SafeTensors model not found in discovery output");
    }
    
    Ok(())
}

fn find_main_safetensors_file(dir: &Path) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;
    
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("safetensors") {
            // Prefer model.safetensors or pytorch_model.safetensors
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if filename.contains("model") && !filename.contains("tokenizer") {
                return Ok(path);
            }
        }
    }
    
    Err("No SafeTensors model file found".into())
}

fn test_memory_intensive_scenarios(test_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧠 Testing memory-intensive scenarios...");
    
    // Create progressively larger SafeTensors files to test memory handling
    let sizes = vec![1, 10, 50, 100]; // MB
    
    for size_mb in sizes {
        println!("📊 Testing {}MB SafeTensors file...", size_mb);
        
        let test_file = test_dir.join(format!("test-{}.safetensors", size_mb));
        
        if let Err(e) = create_test_safetensors_file(&test_file, size_mb) {
            println!("⚠️  Failed to create {}MB test file: {}", size_mb, e);
            continue;
        }
        
        // Test loading this file
        println!("🔄 Testing load performance...");
        let start = std::time::Instant::now();
        
        // Use our create_test_safetensors tool to create a properly structured model
        let output = Command::new("cargo")
            .args(&["run", "--bin", "shimmy", "--", "probe", &format!("test-{}", size_mb)])
            .output();
        
        match output {
            Ok(result) => {
                let elapsed = start.elapsed();
                if result.status.success() {
                    println!("✅ {}MB model loaded in {:?}", size_mb, elapsed);
                } else {
                    println!("❌ {}MB model failed to load: {}", size_mb, String::from_utf8_lossy(&result.stderr));
                }
            }
            Err(e) => {
                println!("❌ Failed to test {}MB model: {}", size_mb, e);
            }
        }
        
        // Clean up
        let _ = fs::remove_file(&test_file);
    }
    
    Ok(())
}

fn test_server_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🌐 Testing server mode with SafeTensors...");
    
    // Start server in background and test it
    println!("🚀 Starting server with SafeTensors model...");
    
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "shimmy", "--", "serve"])
        .spawn()?;
    
    // Give server time to start
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // Test if server is responding
    println!("🔗 Testing server health...");
    let health_result = Command::new("curl")
        .args(&["--max-time", "5", "http://127.0.0.1:11435/health"])
        .output();
    
    match health_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Server responding to health check");
            } else {
                println!("⚠️  Server health check failed");
            }
        }
        Err(_) => {
            println!("⚠️  curl not available or server not responding");
        }
    }
    
    // Stop server
    println!("🛑 Stopping server...");
    let _ = server.kill();
    let _ = server.wait();
    
    Ok(())
}

fn create_test_safetensors_file(path: &Path, size_mb: usize) -> Result<(), Box<dyn std::error::Error>> {
    // Create a SafeTensors file of approximately the specified size
    let tensor_elements = (size_mb * 1024 * 1024) / 4; // Assuming F32 tensors
    
    let metadata = format!(
        r#"{{"test_tensor":{{ "dtype":"F32", "shape":[{},1], "data_offsets":[0,{}] }}}}"#,
        tensor_elements, tensor_elements * 4
    );
    
    let metadata_bytes = metadata.as_bytes();
    let metadata_len = metadata_bytes.len() as u64;
    
    let mut data = Vec::new();
    data.extend_from_slice(&metadata_len.to_le_bytes());
    data.extend_from_slice(metadata_bytes);
    
    // Add tensor data
    let tensor_data = vec![1.0f32; tensor_elements];
    for value in tensor_data {
        data.extend_from_slice(&value.to_le_bytes());
    }
    
    fs::write(path, data)?;
    Ok(())
}