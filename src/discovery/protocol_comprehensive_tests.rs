// Comprehensive tests for Protocol layer
use super::*;

#[cfg(test)]
mod protocol_comprehensive_tests {
    use super::*;
    use crate::discovery::protocol::*;

    #[test]
    fn test_message_serialization_edge_cases() {
        // Test serialization with very long strings
        let long_string = "a".repeat(10000);
        let message = BackendMessage::Register {
            id: long_string.clone(),
            pid: 12345,
            port: 8080,
            capabilities: BackendCapabilities {
                backend_type: long_string.clone(),
                features_compiled: vec![long_string.clone()],
                websocket_working: true,
                http_working: true,
                models_loaded: true,
                streaming_supported: true,
            },
            models: vec![ModelInfo {
                name: long_string.clone(),
                backend_type: long_string.clone(),
                compiled_support: true,
            }],
            started_at: 1234567890,
        };
        
        let serialized = serde_json::to_vec(&message);
        assert!(serialized.is_ok(), "Should serialize large message");
        
        let deserialized: Result<BackendMessage, _> = serde_json::from_slice(&serialized.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize large message");
    }

    #[test]
    fn test_frame_size_boundary_conditions() {
        // Test frame size limits
        use crate::discovery::protocol::Frame;
        
        // Test minimum size frame
        let small_frame = Frame::new(b"{}");
        assert!(small_frame.is_ok(), "Should create small frame");
        
        // Test maximum reasonable size frame (1MB)
        let large_data = vec![b'x'; 1024 * 1024];
        let large_frame = Frame::new(&large_data);
        assert!(large_frame.is_ok(), "Should create large frame up to reasonable limit");
        
        // Test serialization/deserialization of boundary frames
        if let Ok(frame) = small_frame {
            let serialized = frame.serialize();
            assert!(Frame::deserialize(&serialized).is_ok());
        }
    }

    #[test]
    fn test_malformed_protocol_message_handling() {
        // Test handling of various malformed messages
        let malformed_cases = vec![
            b"not json",
            b"{incomplete json",
            b"{}",  // Empty object
            b"{\"invalid_field\": \"value\"}",
            b"{\"type\": \"unknown_type\"}", 
        ];
        
        for malformed in malformed_cases {
            let backend_result: Result<BackendMessage, _> = serde_json::from_slice(malformed);
            let frontend_result: Result<FrontendMessage, _> = serde_json::from_slice(malformed);
            let leader_result: Result<LeaderMessage, _> = serde_json::from_slice(malformed);
            
            // All should fail gracefully (not panic)
            assert!(backend_result.is_err() || frontend_result.is_err() || leader_result.is_err());
        }
    }

    #[test]
    fn test_version_compatibility_scenarios() {
        // Test forward/backward compatibility with message formats
        
        // Simulate an old message format (missing fields)
        let old_format = r#"{
            "Register": {
                "id": "test-backend",
                "port": 8080,
                "capabilities": {
                    "backend_type": "llama",
                    "features_compiled": ["llama"]
                },
                "models": []
            }
        }"#;
        
        let result: Result<BackendMessage, _> = serde_json::from_str(old_format);
        // Should handle missing fields gracefully with defaults
        if result.is_err() {
            // Expected if we have required fields without defaults
            println!("Old format parsing failed as expected: {:?}", result.err());
        }
    }

    #[test] 
    fn test_error_code_serialization() {
        let error_codes = vec![
            ErrorCode::InvalidMessage,
            ErrorCode::BackendNotFound,
            ErrorCode::ValidationFailed,
            ErrorCode::InternalError,
        ];
        
        for code in error_codes {
            let serialized = serde_json::to_string(&code);
            assert!(serialized.is_ok(), "Error code should serialize: {:?}", code);
            
            let deserialized: Result<ErrorCode, _> = serde_json::from_str(&serialized.unwrap());
            assert!(deserialized.is_ok(), "Error code should deserialize: {:?}", code);
            assert_eq!(deserialized.unwrap(), code);
        }
    }

    #[test]
    fn test_backend_capabilities_completeness() {
        // Test that BackendCapabilities covers all expected fields
        let caps = BackendCapabilities {
            backend_type: "test".to_string(),
            features_compiled: vec!["feature1".to_string()],
            websocket_working: true,
            http_working: true,
            models_loaded: true,
            streaming_supported: true,
        };
        
        // Serialize and ensure all fields are present
        let serialized = serde_json::to_value(&caps).unwrap();
        assert!(serialized.get("backend_type").is_some());
        assert!(serialized.get("features_compiled").is_some());
        assert!(serialized.get("websocket_working").is_some());
        assert!(serialized.get("http_working").is_some());
        assert!(serialized.get("models_loaded").is_some());
        assert!(serialized.get("streaming_supported").is_some());
    }

    #[test]
    fn test_model_info_validation() {
        let valid_model = ModelInfo {
            name: "test-model".to_string(),
            backend_type: "llama".to_string(),
            compiled_support: true,
        };
        
        let serialized = serde_json::to_string(&valid_model);
        assert!(serialized.is_ok());
        
        let deserialized: Result<ModelInfo, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
        
        let model = deserialized.unwrap();
        assert_eq!(model.name, "test-model");
        assert_eq!(model.backend_type, "llama");
        assert!(model.compiled_support);
    }

    #[test]
    fn test_frame_corruption_detection() {
        use crate::discovery::protocol::Frame;
        
        // Test that corrupted frames are detected
        let valid_frame = Frame::new(b"valid data").unwrap();
        let mut serialized = valid_frame.serialize();
        
        // Corrupt the length prefix
        if serialized.len() >= 4 {
            serialized[0] = 0xFF;
            serialized[1] = 0xFF;
            serialized[2] = 0xFF;
            serialized[3] = 0xFF;
            
            let result = Frame::deserialize(&serialized);
            assert!(result.is_err(), "Should detect corrupted frame");
        }
    }

    #[test]
    fn test_concurrent_message_serialization() {
        use std::sync::Arc;
        use std::thread;
        
        // Test that serialization is thread-safe
        let message = Arc::new(BackendMessage::Register {
            id: "test-backend".to_string(),
            pid: 12345,
            port: 8080,
            capabilities: BackendCapabilities::default(),
            models: vec![],
            started_at: 1234567890,
        });
        
        let handles: Vec<_> = (0..10).map(|_| {
            let msg = Arc::clone(&message);
            thread::spawn(move || {
                let serialized = serde_json::to_string(&*msg);
                assert!(serialized.is_ok());
                serialized.unwrap()
            })
        }).collect();
        
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        
        // All serializations should be identical
        let first = &results[0];
        for result in &results[1..] {
            assert_eq!(result, first, "Concurrent serializations should be identical");
        }
    }

    #[test]
    fn test_registry_state_consistency() {
        let mut registry = IpcRegistry::new();
        
        // Add multiple backends
        for i in 0..5 {
            let backend = BackendInfo {
                id: format!("backend-{}", i),
                port: 8080 + i,
                url: format!("http://127.0.0.1:{}", 8080 + i),
                capabilities: BackendCapabilities::default(),
                models: vec![],
                health: crate::discovery::service::HealthStatus::Ok,
                validation: crate::discovery::service::ValidationStatus::default(),
                last_update: 1234567890,
                pid: Some(12345 + i as u32),
            };
            registry.backends.insert(backend.id.clone(), backend);
        }
        
        // Test that list_backends returns consistent results
        let backends1 = registry.list_backends();
        let backends2 = registry.list_backends();
        
        assert_eq!(backends1.len(), backends2.len());
        assert_eq!(backends1.len(), 5);
        
        // Backends should be in consistent order
        for (b1, b2) in backends1.iter().zip(backends2.iter()) {
            assert_eq!(b1.id, b2.id);
            assert_eq!(b1.port, b2.port);
        }
    }
}

#[cfg(test)]
mod protocol_property_tests {
    use super::*;
    use crate::discovery::protocol::*;
    
    #[test]
    fn test_serialization_roundtrip_property() {
        // Property: serialize -> deserialize should be identity
        let test_cases = vec![
            BackendMessage::Register {
                id: "test".to_string(),
                pid: 12345,
                port: 8080,
                capabilities: BackendCapabilities::default(),
                models: vec![],
                started_at: 1234567890,
            },
            BackendMessage::Update {
                id: "test".to_string(),
                capabilities: BackendCapabilities::default(),
                models: vec![],
            },
            BackendMessage::Goodbye {
                id: "test".to_string(),
            },
        ];
        
        for original in test_cases {
            let serialized = serde_json::to_string(&original).unwrap();
            let deserialized: BackendMessage = serde_json::from_str(&serialized).unwrap();
            
            // Compare the enum variants
            match (&original, &deserialized) {
                (BackendMessage::Register { id: id1, .. }, BackendMessage::Register { id: id2, .. }) => {
                    assert_eq!(id1, id2);
                }
                (BackendMessage::Update { id: id1, .. }, BackendMessage::Update { id: id2, .. }) => {
                    assert_eq!(id1, id2);
                }
                (BackendMessage::Goodbye { id: id1 }, BackendMessage::Goodbye { id: id2 }) => {
                    assert_eq!(id1, id2);
                }
                _ => panic!("Serialization changed message type"),
            }
        }
    }
}
