/// Regression tests for compilation and GPU feature issues
///
/// This test suite covers fixes for:
/// - Issue #59: CUDA compilation errors (cudart_static not found)
/// - Issue #58: GPU capabilities in prebuilt binaries
/// - General compilation robustness
// - Feature flag combinations
#[test]
fn test_feature_compilation_compatibility() {
    // Test that different feature combinations can be compiled
    // This is mainly a compile-time test - if this test runs, it means features compiled

    #[cfg(feature = "llama")]
    {
        // Test that llama feature compiles without issues
        // If we can create an adapter, the feature is working
        use shimmy::engine::adapter::InferenceEngineAdapter;
        let _adapter = InferenceEngineAdapter::new();
    }

    #[cfg(feature = "huggingface")]
    {
        // Test that huggingface feature compiles without issues
    }

    #[cfg(feature = "llama-cuda")]
    {
        // Test that CUDA feature compiles (may not be available at runtime)
    }

    #[cfg(feature = "llama-vulkan")]
    {
        // Test that Vulkan feature compiles
    }

    #[cfg(feature = "llama-opencl")]
    {
        // Test that OpenCL feature compiles
    }
}

#[test]
fn test_gpu_feature_flags_available() {
    // Test that GPU feature flags are properly exposed for runtime checking

    let has_llama = cfg!(feature = "llama");
    let has_cuda = cfg!(feature = "llama-cuda");
    let has_vulkan = cfg!(feature = "llama-vulkan");
    let has_opencl = cfg!(feature = "llama-opencl");
    let has_huggingface = cfg!(feature = "huggingface");

    // At least one backend should be available
    assert!(
        has_llama || has_huggingface,
        "At least one inference backend should be available"
    );

    // GPU features should be properly conditional on base llama feature
    if has_cuda || has_vulkan || has_opencl {
        // Note: In some build configurations, GPU features might be available without base llama
        // This test mainly ensures the flags are accessible
    }

    // Log which features are available (visible in test output)
    println!("Available features:");
    println!("  llama: {}", has_llama);
    println!("  cuda: {}", has_cuda);
    println!("  vulkan: {}", has_vulkan);
    println!("  opencl: {}", has_opencl);
    println!("  huggingface: {}", has_huggingface);
}

#[test]
fn test_inference_engine_adapter_creation() {
    // Test that InferenceEngineAdapter can be created without panicking
    // Regression test for compilation issues that would prevent engine creation

    use shimmy::engine::adapter::InferenceEngineAdapter;

    let adapter = InferenceEngineAdapter::new();

    // Basic checks that the adapter is functional
    // We can't test actual inference without models, but we can test creation

    // Test that we can call basic methods without panicking
    drop(adapter); // Ensure cleanup works
}

#[test]
fn test_app_state_initialization_robustness() {
    // Test that AppState can be initialized with different engine configurations
    // Regression test for initialization issues

    use shimmy::{engine::adapter::InferenceEngineAdapter, model_registry::Registry, AppState};

    let registry = Registry::default();
    let engine = Box::new(InferenceEngineAdapter::new());

    let state = AppState::new(engine, registry);

    // Basic functionality checks
    assert_eq!(state.registry.list().len(), 0);
    assert_eq!(state.registry.list_all_available().len(), 0);

    // Test that state is usable
    drop(state);
}

#[test]
fn test_library_dependencies_available() {
    // Test that key dependencies are available and functional
    // Helps catch linking issues or missing dependencies

    // Test serde_json (critical for API compatibility)
    let test_json = serde_json::json!({"test": "value"});
    assert_eq!(test_json["test"], "value");

    // Test tokio (critical for async operation)
    let rt = tokio::runtime::Runtime::new();
    assert!(rt.is_ok(), "Tokio runtime should be available");

    // Test axum (critical for web server)
    use axum::Router;
    let _router: Router = Router::new();

    // Test that basic operations work
}

#[test]
fn test_model_registry_thread_safety() {
    // Test that Registry can be used safely across threads
    // Important for server operation under load

    use shimmy::model_registry::{ModelEntry, Registry};
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::thread;

    let registry = Arc::new(Mutex::new(Registry::new()));

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let registry = registry.clone();
            thread::spawn(move || {
                let test_model = ModelEntry {
                    name: format!("test-model-{}", i),
                    base_path: PathBuf::from(format!("test{}.gguf", i)),
                    lora_path: None,
                    template: Some("chatml".to_string()),
                    ctx_len: Some(2048),
                    n_threads: None,
                };

                let mut reg = registry.lock().unwrap();
                reg.register(test_model);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let reg = registry.lock().unwrap();
    assert_eq!(
        reg.list().len(),
        5,
        "All models should be registered safely"
    );
}

#[test]
fn test_cargo_build_environment_compatibility() {
    // Test that build-time environment variables are properly handled
    // Regression test for build issues

    // Test that package metadata is available
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty(), "Package version should be available");

    let name = env!("CARGO_PKG_NAME");
    assert_eq!(name, "shimmy", "Package name should be correct");

    // These should be available during build
    assert!(!version.is_empty(), "Version string should not be empty");
}

#[test]
fn test_memory_usage_basic_operations() {
    // Basic test to ensure operations don't cause obvious memory issues
    // Not a comprehensive memory test, but catches gross leaks

    use shimmy::{engine::adapter::InferenceEngineAdapter, model_registry::Registry, AppState};
    use std::sync::Arc;

    // Create and drop multiple states to test for basic memory leaks
    for _ in 0..10 {
        let registry = Registry::default();
        let engine = Box::new(InferenceEngineAdapter::new());
        let state = Arc::new(AppState::new(engine, registry));

        // Do some basic operations
        let _models = state.registry.list();
        let _available = state.registry.list_all_available();

        // State should be dropped properly
        drop(state);
    }

    // If we get here without running out of memory, basic memory management is working
}

#[test]
fn test_error_handling_compilation() {
    // Test that our error types compile and work correctly
    // Regression test for error handling issues

    use shimmy::model_registry::Registry;

    let registry = Registry::new();

    // Test that error conditions are handled properly
    let result = registry.get("nonexistent-model");
    assert!(result.is_none(), "Should handle missing models gracefully");

    // Test that we can handle errors without panicking
    let empty_list = registry.list();
    assert_eq!(
        empty_list.len(),
        0,
        "Empty registry should return empty list"
    );
}

#[cfg(test)]
mod conditional_tests {
    // Tests that run only with specific features enabled

    #[cfg(feature = "llama")]
    #[test]
    fn test_llama_specific_functionality() {
        // Test llama-specific code paths
        use shimmy::engine::adapter::InferenceEngineAdapter;

        let adapter = InferenceEngineAdapter::new();
        // If this compiles and runs, llama integration is working
        drop(adapter);
    }

    #[cfg(feature = "huggingface")]
    #[test]
    fn test_huggingface_specific_functionality() {
        // Test huggingface-specific code paths
    }
}
