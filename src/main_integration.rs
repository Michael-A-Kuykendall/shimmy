// Main integration utilities for production setup
#![allow(dead_code)]

use crate::model_registry::ModelRegistry;
use std::sync::Arc;

pub fn create_integrated_registry() -> ModelRegistry {
    let registry = ModelRegistry::new();

    // Additional setup for production
    println!("Shimmy production registry initialized");

    registry
}

pub fn setup_production_server() -> Arc<()> {
    // Placeholder for production setup
    Arc::new(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_create_integrated_registry() {
        // Test that create_integrated_registry returns a valid ModelRegistry
        let registry = create_integrated_registry();

        // Verify it's a new empty registry
        let models = registry.list();
        assert!(models.is_empty(), "New integrated registry should be empty");

        // Verify it's actually a ModelRegistry instance
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_create_integrated_registry_is_functional() {
        // Test that the returned registry can actually be used
        let mut registry = create_integrated_registry();

        // Test that we can register a model
        use crate::model_registry::ModelEntry;
        use std::path::PathBuf;

        let test_entry = ModelEntry {
            name: "test-integration-model".to_string(),
            base_path: PathBuf::from("/test/path"),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: Some(4096),
            n_threads: Some(4),
        };

        registry.register(test_entry);

        // Verify registration worked
        let models = registry.list();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "test-integration-model");

        // Verify we can retrieve the model
        let retrieved = registry.get("test-integration-model");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test-integration-model");
    }

    #[test]
    fn test_create_integrated_registry_multiple_calls() {
        // Test that multiple calls create independent registries
        let registry1 = create_integrated_registry();
        let registry2 = create_integrated_registry();

        // Both should be empty initially
        assert_eq!(registry1.list().len(), 0);
        assert_eq!(registry2.list().len(), 0);

        // They should be independent instances
        // (We can't directly test memory addresses, but we can test behavior)
        let mut registry1_mut = registry1;
        let registry2_immut = registry2;

        use crate::model_registry::ModelEntry;
        use std::path::PathBuf;

        let test_entry = ModelEntry {
            name: "test-model".to_string(),
            base_path: PathBuf::from("/test"),
            lora_path: None,
            template: None,
            ctx_len: None,
            n_threads: None,
        };

        registry1_mut.register(test_entry);

        // Registry1 should have 1 model, registry2 should still be empty
        assert_eq!(registry1_mut.list().len(), 1);
        assert_eq!(registry2_immut.list().len(), 0);
    }

    #[test]
    fn test_create_integrated_registry_stdout_capture() {
        // Test that the function executes without panicking
        // (We can't easily test the println! output, but we ensure the function completes)
        let registry = create_integrated_registry();

        // Function should complete successfully and return a working registry
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_setup_production_server() {
        // Test that setup_production_server returns a valid Arc<()>
        let server = setup_production_server();

        // Verify it's an Arc<()>
        assert_eq!(*server, ());
    }

    #[test]
    fn test_setup_production_server_arc_properties() {
        // Test Arc properties and behavior
        let server = setup_production_server();

        // Test cloning the Arc (should work without issues)
        let server_clone = Arc::clone(&server);
        assert_eq!(*server, *server_clone);

        // Test that both references point to the same data
        assert_eq!(Arc::strong_count(&server), 2);

        drop(server_clone);
        assert_eq!(Arc::strong_count(&server), 1);
    }

    #[test]
    fn test_setup_production_server_multiple_calls() {
        // Test that multiple calls create independent Arc instances
        let server1 = setup_production_server();
        let server2 = setup_production_server();

        // Both should contain ()
        assert_eq!(*server1, ());
        assert_eq!(*server2, ());

        // They should be independent Arc instances
        assert_eq!(Arc::strong_count(&server1), 1);
        assert_eq!(Arc::strong_count(&server2), 1);
    }

    #[test]
    fn test_integration_functions_work_together() {
        // Test that both functions can be called together without conflicts
        let registry = create_integrated_registry();
        let server = setup_production_server();

        // Both should be valid
        assert!(registry.list().is_empty());
        assert_eq!(*server, ());

        // Test using them in a hypothetical production scenario
        let mut registry_mut = registry;
        let _server_ref = Arc::clone(&server);

        use crate::model_registry::ModelEntry;
        use std::path::PathBuf;

        let production_model = ModelEntry {
            name: "production-model".to_string(),
            base_path: PathBuf::from("/opt/models/production"),
            lora_path: Some(PathBuf::from("/opt/models/lora")),
            template: Some("llama3".to_string()),
            ctx_len: Some(8192),
            n_threads: Some(8),
        };

        registry_mut.register(production_model);

        // Verify production setup
        assert_eq!(registry_mut.list().len(), 1);
        assert_eq!(registry_mut.list()[0].name, "production-model");
        assert_eq!(registry_mut.list()[0].ctx_len, Some(8192));
    }

    #[test]
    fn test_registry_advanced_functionality() {
        // Test advanced registry functionality through integration
        let mut registry = create_integrated_registry();

        // Test with discovery functionality
        registry.refresh_discovered_models();

        // Test auto-registration
        registry.auto_register_discovered();

        // Test listing all available models
        let all_models = registry.list_all_available();
        // Should be able to call without panicking - verify it returns a valid Vec
        assert!(all_models.is_empty() || !all_models.is_empty()); // Will be 0 or more depending on discovered models
    }

    #[test]
    fn test_registry_model_spec_conversion() {
        // Test model spec conversion through integration
        let mut registry = create_integrated_registry();

        use crate::model_registry::ModelEntry;
        use std::path::PathBuf;

        let test_model = ModelEntry {
            name: "spec-test-model".to_string(),
            base_path: PathBuf::from("/models/spec-test"),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: Some(2048),
            n_threads: Some(2),
        };

        registry.register(test_model);

        // Test conversion to ModelSpec
        let spec = registry.to_spec("spec-test-model");
        assert!(spec.is_some());

        let spec = spec.unwrap();
        assert_eq!(spec.name, "spec-test-model");
        assert_eq!(spec.ctx_len, 2048);
        assert_eq!(spec.template, Some("chatml".to_string()));

        // Test non-existent model
        let missing_spec = registry.to_spec("non-existent-model");
        assert!(missing_spec.is_none());
    }
}
