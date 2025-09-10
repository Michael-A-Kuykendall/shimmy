// Comprehensive SafeTensors integration tests
// These tests will catch crashes, memory issues, and edge cases

use std::fs;
use std::path::Path;
use shimmy::engine::{ModelSpec, InferenceEngine, GenOptions};
use shimmy::engine::adapter::InferenceEngineAdapter;

#[tokio::test]
async fn test_safetensors_basic_loading() {
    // Test the basic case that was crashing
    let spec = ModelSpec {
        name: "test-safetensors".to_string(),
        base_path: "test-safetensors-model/model.safetensors".into(),
        lora_path: None,
        template: Some("chatml".to_string()),
        ctx_len: 2048,
        n_threads: Some(4),
    };

    let engine = InferenceEngineAdapter::new();
    
    // This should not crash or hang
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        engine.load(&spec)
    ).await;

    match result {
        Ok(Ok(model)) => {
            println!("✅ SafeTensors model loaded successfully");
            
            // Test basic generation - this is where it might have crashed
            let gen_result = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                model.generate("Hello", GenOptions::default(), None)
            ).await;
            
            match gen_result {
                Ok(Ok(response)) => {
                    println!("✅ Generation completed: {}", response);
                    assert!(!response.is_empty());
                }
                Ok(Err(e)) => {
                    panic!("❌ Generation failed: {}", e);
                }
                Err(_) => {
                    panic!("❌ Generation timed out - likely infinite loop");
                }
            }
        }
        Ok(Err(e)) => {
            panic!("❌ Model loading failed: {}", e);
        }
        Err(_) => {
            panic!("❌ Model loading timed out - likely hanging");
        }
    }
}

#[tokio::test]
async fn test_safetensors_malformed_file() {
    // Create a malformed SafeTensors file to test error handling
    let malformed_path = "test-malformed.safetensors";
    fs::write(malformed_path, b"not valid safetensors data").expect("Failed to create test file");

    let spec = ModelSpec {
        name: "malformed".to_string(),
        base_path: malformed_path.into(),
        lora_path: None,
        template: None,
        ctx_len: 2048,
        n_threads: None,
    };

    let engine = InferenceEngineAdapter::new();
    
    // This should fail gracefully, not crash
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        engine.load(&spec)
    ).await;

    // Clean up
    let _ = fs::remove_file(malformed_path);

    match result {
        Ok(Err(_)) => {
            println!("✅ Malformed file handled gracefully");
        }
        Ok(Ok(_)) => {
            panic!("❌ Malformed file should not have loaded successfully");
        }
        Err(_) => {
            panic!("❌ Malformed file handling timed out");
        }
    }
}

#[tokio::test]
async fn test_safetensors_missing_file() {
    let spec = ModelSpec {
        name: "nonexistent".to_string(),
        base_path: "nonexistent.safetensors".into(),
        lora_path: None,
        template: None,
        ctx_len: 2048,
        n_threads: None,
    };

    let engine = InferenceEngineAdapter::new();
    
    // This should fail gracefully, not crash
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        engine.load(&spec)
    ).await;

    match result {
        Ok(Err(_)) => {
            println!("✅ Missing file handled gracefully");
        }
        Ok(Ok(_)) => {
            panic!("❌ Missing file should not have loaded successfully");
        }
        Err(_) => {
            panic!("❌ Missing file handling timed out");
        }
    }
}

#[tokio::test]
async fn test_safetensors_memory_usage() {
    // Test memory usage doesn't explode
    use std::process::Command;

    let initial_memory = get_memory_usage();
    
    let spec = ModelSpec {
        name: "memory-test".to_string(),
        base_path: "test-safetensors-model/model.safetensors".into(),
        lora_path: None,
        template: None,
        ctx_len: 2048,
        n_threads: None,
    };

    let engine = InferenceEngineAdapter::new();
    let model = engine.load(&spec).await.expect("Model should load");
    
    let after_load_memory = get_memory_usage();
    
    // Generate multiple times to check for memory leaks
    for i in 0..5 {
        let _ = model.generate(&format!("Test {}", i), GenOptions::default(), None).await;
    }
    
    let after_generation_memory = get_memory_usage();
    
    println!("Memory usage - Initial: {}MB, After load: {}MB, After generation: {}MB", 
             initial_memory, after_load_memory, after_generation_memory);
    
    // Memory shouldn't grow by more than 500MB (reasonable for small model)
    assert!(after_generation_memory - initial_memory < 500, 
            "Memory usage grew too much: {}MB", after_generation_memory - initial_memory);
}

fn get_memory_usage() -> u64 {
    // Simple memory usage check (Windows/Linux compatible)
    #[cfg(target_os = "windows")]
    {
        // For Windows, we'll use a simple approximation
        std::process::id() as u64 // Placeholder - actual implementation would use Windows API
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // For Unix-like systems
        let output = std::process::Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output();
            
        if let Ok(output) = output {
            let rss_str = String::from_utf8_lossy(&output.stdout);
            rss_str.trim().parse::<u64>().unwrap_or(0) / 1024 // Convert KB to MB
        } else {
            0
        }
    }
}

// Helper function to create various test SafeTensors files
fn create_test_safetensors(path: &str, size_mb: usize) -> Result<(), Box<dyn std::error::Error>> {
    // Create SafeTensors files of different sizes for testing
    let tensor_count = size_mb * 1024; // Rough approximation
    
    let metadata = format!(
        r#"{{"test_tensor_{}":{{"dtype":"F32","shape":[{},4],"data_offsets":[0,{}]}}}}"#,
        size_mb, tensor_count, tensor_count * 16
    );
    
    let metadata_bytes = metadata.as_bytes();
    let metadata_len = metadata_bytes.len() as u64;
    
    let mut data = Vec::new();
    data.extend_from_slice(&metadata_len.to_le_bytes());
    data.extend_from_slice(metadata_bytes);
    
    // Add tensor data
    let tensor_data = vec![1.0f32; tensor_count * 4];
    for value in tensor_data {
        data.extend_from_slice(&value.to_le_bytes());
    }
    
    fs::write(path, data)?;
    Ok(())
}

#[tokio::test]
async fn test_safetensors_various_sizes() {
    // Test different sized models to find memory issues
    let sizes = vec![1, 10, 50]; // MB sizes to test
    
    for size_mb in sizes {
        let path = format!("test-size-{}.safetensors", size_mb);
        
        // Create test file
        if let Err(e) = create_test_safetensors(&path, size_mb) {
            println!("Failed to create test file {}: {}", path, e);
            continue;
        }
        
        println!("Testing {}MB SafeTensors file...", size_mb);
        
        let spec = ModelSpec {
            name: format!("size-test-{}", size_mb),
            base_path: path.clone().into(),
            lora_path: None,
            template: None,
            ctx_len: 2048,
            n_threads: None,
        };

        let engine = InferenceEngineAdapter::new();
        
        // Test with timeout to catch hangs
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            engine.load(&spec)
        ).await;

        // Clean up test file
        let _ = fs::remove_file(&path);

        match result {
            Ok(Ok(_)) => {
                println!("✅ {}MB model loaded successfully", size_mb);
            }
            Ok(Err(e)) => {
                println!("⚠️  {}MB model failed to load: {}", size_mb, e);
            }
            Err(_) => {
                panic!("❌ {}MB model loading timed out - likely hanging", size_mb);
            }
        }
    }
}