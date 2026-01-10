// Model Loading Performance Benchmarks
// Measures performance of various model loading operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shimmy::auto_discovery::ModelAutoDiscovery;
use shimmy::model_registry::{ModelEntry, Registry};
use std::path::PathBuf;

fn benchmark_model_discovery(c: &mut Criterion) {
    c.bench_function("model_auto_discovery_scan", |b| {
        b.iter(|| {
            let discovery = ModelAutoDiscovery::new();
            let discovered = discovery.discover_models();
            black_box(discovered)
        })
    });
}

fn benchmark_model_registry(c: &mut Criterion) {
    let mut registry = Registry::new();

    c.bench_function("model_registry_register", |b| {
        b.iter(|| {
            let entry = ModelEntry {
                name: black_box("test-model".to_string()),
                base_path: black_box(PathBuf::from("test.gguf")),
                lora_path: None,
                template: Some("chatml".to_string()),
                ctx_len: Some(black_box(4096)),
                n_threads: Some(black_box(4)),
            };
            registry.register(black_box(entry));
        })
    });

    // Add some models for listing benchmark
    for i in 0..100 {
        let entry = ModelEntry {
            name: format!("model-{}", i),
            base_path: PathBuf::from(format!("model-{}.gguf", i)),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: Some(4096),
            n_threads: Some(4),
        };
        registry.register(entry);
    }

    c.bench_function("model_registry_list_100", |b| {
        b.iter(|| {
            let models = registry.list();
            black_box(models)
        })
    });

    c.bench_function("model_registry_get", |b| {
        b.iter(|| {
            let model = registry.get(black_box("model-50"));
            black_box(model)
        })
    });

    c.bench_function("model_registry_infer_template", |b| {
        b.iter(|| {
            let template = registry.infer_template(black_box("llama-3-8b"));
            black_box(template)
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
