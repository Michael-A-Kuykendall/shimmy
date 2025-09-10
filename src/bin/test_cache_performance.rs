// Test metadata caching performance
// Compares first load vs cached load times

use anyhow::Result;
use std::path::Path;
use std::time::Instant;
use tokio;

use shimmy::cache::ModelCache;
use shimmy::engine::{ModelSpec, InferenceEngine};
use shimmy::engine::safetensors_native::SafeTensorsEngine;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    // Find a test SafeTensors model
    let test_model_path = find_test_model()?;
    
    let spec = ModelSpec {
        name: "cache-test".to_string(),
        base_path: test_model_path.clone(),
        lora_path: None,
        template: None,
        ctx_len: 2048,
        n_threads: None,
    };
    
    let engine = SafeTensorsEngine::new();
    
    println!("Testing metadata caching performance");
    println!("Model: {}", test_model_path.display());
    
    // Clear any existing cache
    let mut cache = ModelCache::new()?;
    cache.clear()?;
    
    // First load (no cache)
    println!("\n=== First Load (No Cache) ===");
    let start = Instant::now();
    let _model1 = engine.load(&spec).await?;
    let first_load_time = start.elapsed();
    println!("First load time: {:?}", first_load_time);
    
    // Second load (with cache)
    println!("\n=== Second Load (With Cache) ===");
    let start = Instant::now();
    let _model2 = engine.load(&spec).await?;
    let cached_load_time = start.elapsed();
    println!("Cached load time: {:?}", cached_load_time);
    
    // Calculate improvement
    let improvement = first_load_time.as_secs_f64() / cached_load_time.as_secs_f64();
    println!("\n=== Performance Results ===");
    println!("Speedup: {:.2}x faster", improvement);
    println!("Time saved: {:?}", first_load_time - cached_load_time);
    
    // Show cache stats
    let stats = cache.stats();
    println!("\n=== Cache Stats ===");
    println!("Cached entries: {}", stats.entries);
    println!("Cache directory: {}", stats.cache_dir.display());
    
    Ok(())
}

fn find_test_model() -> Result<std::path::PathBuf> {
    // Look for any SafeTensors model in common locations
    let test_paths = [
        "test-huggingface-model/model.safetensors",  // Larger test model
        "test-realistic-safetensors-model/model.safetensors",
        "test-safetensors-model/model.safetensors",
        "models/model.safetensors",
    ];
    
    for path in &test_paths {
        let path_buf = std::path::PathBuf::from(path);
        if path_buf.exists() {
            return Ok(path_buf);
        }
    }
    
    Err(anyhow::anyhow!("No test SafeTensors model found. Create one first with: cargo run --bin create_test_safetensors"))
}