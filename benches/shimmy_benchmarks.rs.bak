use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use shimmy::engine::{GenOptions, ModelBackend, UniversalModelSpec};
use shimmy::model_registry::{Registry, UniversalModelEntry, ModelBackendConfig};
use std::path::PathBuf;

fn bench_registry_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Registry Operations");
    
    // Benchmark model registration
    group.bench_function("register_universal_model", |b| {
        b.iter(|| {
            let mut registry = Registry::default();
            let model = UniversalModelEntry {
                name: black_box("test-model".to_string()),
                backend: ModelBackendConfig::HuggingFace {
                    base_model_id: black_box("microsoft/DialoGPT-small".to_string()),
                    peft_path: Some(PathBuf::from("./adapter")),
                    use_local: Some(true),
                },
                template: Some("chatml".to_string()),
                ctx_len: Some(4096),
                device: Some("cuda".to_string()),
                n_threads: None,
            };
            registry.register_universal(model);
        });
    });
    
    // Benchmark model lookup
    let mut registry = Registry::default();
    for i in 0..1000 {
        registry.register_universal(UniversalModelEntry {
            name: format!("model-{}", i),
            backend: ModelBackendConfig::HuggingFace {
                base_model_id: "test/model".to_string(),
                peft_path: None,
                use_local: Some(true),
            },
            template: None,
            ctx_len: Some(4096),
            device: Some("cuda".to_string()),
            n_threads: None,
        });
    }
    
    group.bench_function("lookup_universal_model", |b| {
        b.iter(|| {
            registry.get_universal(black_box("model-500"));
        });
    });
    
    group.bench_function("convert_to_universal_spec", |b| {
        b.iter(|| {
            registry.to_universal_spec(black_box("model-500"));
        });
    });
    
    group.finish();
}

fn bench_spec_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Spec Conversions");
    
    group.bench_function("backend_config_to_backend", |b| {
        let config = ModelBackendConfig::HuggingFace {
            base_model_id: "microsoft/Phi-3-mini-4k-instruct".to_string(),
            peft_path: Some(PathBuf::from("./adapter")),
            use_local: Some(true),
        };
        
        b.iter(|| {
            let backend: ModelBackend = black_box(config.clone()).into();
            black_box(backend);
        });
    });
    
    group.bench_function("universal_spec_creation", |b| {
        b.iter(|| {
            let spec = UniversalModelSpec {
                name: black_box("test-model".to_string()),
                backend: ModelBackend::HuggingFace {
                    base_model_id: black_box("microsoft/Phi-3-mini-4k-instruct".to_string()),
                    peft_path: Some(PathBuf::from("./adapter")),
                    use_local: true,
                },
                template: Some("chatml".to_string()),
                ctx_len: 4096,
                device: "cuda".to_string(),
                n_threads: Some(8),
            };
            black_box(spec);
        });
    });
    
    group.finish();
}

fn bench_gen_options(c: &mut Criterion) {
    let mut group = c.benchmark_group("GenOptions");
    
    group.bench_function("default_creation", |b| {
        b.iter(|| {
            let opts = GenOptions::default();
            black_box(opts);
        });
    });
    
    group.bench_function("custom_creation", |b| {
        b.iter(|| {
            let opts = GenOptions {
                max_tokens: black_box(512),
                temperature: black_box(0.8),
                top_p: black_box(0.95),
                top_k: black_box(50),
                repeat_penalty: black_box(1.2),
                seed: Some(black_box(42)),
                stream: black_box(false),
            };
            black_box(opts);
        });
    });
    
    group.bench_function("clone_options", |b| {
        let original = GenOptions {
            max_tokens: 1024,
            temperature: 0.9,
            top_p: 0.8,
            top_k: 60,
            repeat_penalty: 1.3,
            seed: Some(123),
            stream: true,
        };
        
        b.iter(|| {
            let cloned = black_box(original.clone());
            black_box(cloned);
        });
    });
    
    group.finish();
}

fn bench_template_rendering(c: &mut Criterion) {
    use shimmy::templates::TemplateFamily;
    
    let mut group = c.benchmark_group("Template Rendering");
    
    let messages = vec![
        ("user".to_string(), "Hello, how are you today?".to_string()),
        ("assistant".to_string(), "I'm doing well, thank you for asking! How can I help you?".to_string()),
        ("user".to_string(), "I need help with some Rust code. Can you explain closures?".to_string()),
        ("assistant".to_string(), "Closures in Rust are anonymous functions that can capture variables from their surrounding scope.".to_string()),
    ];
    
    let system_prompt = "You are a helpful programming assistant specializing in Rust.";
    
    for template in &[TemplateFamily::ChatML, TemplateFamily::Llama3, TemplateFamily::OpenChat] {
        group.bench_with_input(
            BenchmarkId::new("render_template", format!("{:?}", template)),
            template,
            |b, template| {
                b.iter(|| {
                    let rendered = template.render(
                        Some(black_box(system_prompt)),
                        black_box(&messages),
                        None,
                    );
                    black_box(rendered);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_concurrent_registry_access(c: &mut Criterion) {
    use std::sync::Arc;
    
    let mut group = c.benchmark_group("Concurrent Registry");
    
    let mut registry = Registry::default();
    for i in 0..100 {
        registry.register_universal(UniversalModelEntry {
            name: format!("concurrent-model-{}", i),
            backend: ModelBackendConfig::HuggingFace {
                base_model_id: "test/model".to_string(),
                peft_path: None,
                use_local: Some(true),
            },
            template: None,
            ctx_len: Some(4096),
            device: Some("cpu".to_string()),
            n_threads: None,
        });
    }
    let registry = Arc::new(registry);
    
    group.bench_function("concurrent_lookups", |b| {
        b.iter(|| {
            // Simulate concurrent access by doing multiple lookups
            for i in 0..10 {
                black_box(registry.get_universal(&format!("concurrent-model-{}", black_box(i * 5))));
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_registry_operations,
    bench_spec_conversions,
    bench_gen_options,
    bench_template_rendering,
    bench_concurrent_registry_access,
);

criterion_main!(benches);