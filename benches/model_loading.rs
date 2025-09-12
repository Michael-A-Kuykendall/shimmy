// Model Loading Performance Benchmarks
// Measures performance of various model loading operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shimmy::discovery::*;
use shimmy::model_registry::*;
use std::path::PathBuf;
use tempfile::TempDir;

fn benchmark_model_discovery(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    c.bench_function("model_discovery_empty_dir", |b| {
        b.iter(|| {
            let discovered = discover_models_from_directory(black_box(temp_path));
            black_box(discovered)
        })
    });
}

fn benchmark_model_registry(c: &mut Criterion) {
    let mut registry = ModelRegistry::new();

    c.bench_function("model_registry_add", |b| {
        b.iter(|| {
            let model_spec = ModelSpec {
                name: black_box("test-model".to_string()),
                base_path: black_box(PathBuf::from("test.gguf")),
                lora_path: None,
                template: None,
                ctx_len: black_box(4096),
                n_threads: black_box(4),
            };
            registry.add_model(black_box(model_spec));
        })
    });

    // Add some models for listing benchmark
    for i in 0..100 {
        let model_spec = ModelSpec {
            name: format!("model-{}", i),
            base_path: PathBuf::from(format!("model-{}.gguf", i)),
            lora_path: None,
            template: None,
            ctx_len: 4096,
            n_threads: 4,
        };
        registry.add_model(model_spec);
    }

    c.bench_function("model_registry_list_100", |b| {
        b.iter(|| {
            let models = registry.list_models();
            black_box(models)
        })
    });
}

fn benchmark_safetensors_detection(c: &mut Criterion) {
    c.bench_function("safetensors_file_detection", |b| {
        b.iter(|| {
            let paths = vec![
                "model.safetensors",
                "model.gguf",
                "model.bin",
                "pytorch_model.bin",
                "model.pt",
            ];

            for path in paths {
                let path_buf = PathBuf::from(black_box(path));
                let is_safetensors = path_buf
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "safetensors")
                    .unwrap_or(false);
                black_box(is_safetensors);
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_model_discovery,
    benchmark_model_registry,
    benchmark_safetensors_detection
);
criterion_main!(benches);
