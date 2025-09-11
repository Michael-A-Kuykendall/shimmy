use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShimmyError {
    #[error("Model not found: {name}")]
    ModelNotFound { name: String },

    #[error("Model loading failed: {path}")]
    ModelLoadError {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    #[error("Generation failed: {reason}")]
    GenerationError { reason: String },

    #[error("Invalid configuration: {field} = {value}")]
    ConfigError { field: String, value: String },

    #[error("Backend not available: {backend}")]
    BackendNotAvailable { backend: String },

    #[error("Template rendering failed: {template}")]
    TemplateError {
        template: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Async runtime error")]
    AsyncError(#[from] tokio::task::JoinError),

    #[error("IO error")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error")]
    SerdeError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ShimmyError>;

impl From<anyhow::Error> for ShimmyError {
    fn from(err: anyhow::Error) -> Self {
        ShimmyError::GenerationError {
            reason: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::error::Error as StdError;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_model_not_found_error() {
        let error = ShimmyError::ModelNotFound {
            name: "test-model".to_string(),
        };

        assert_eq!(error.to_string(), "Model not found: test-model");
        assert!(format!("{:?}", error).contains("ModelNotFound"));
        assert!(format!("{:?}", error).contains("test-model"));
    }

    #[test]
    fn test_model_load_error() {
        let path = PathBuf::from("/path/to/model.bin");
        let source_error = anyhow!("Failed to read file");
        let error = ShimmyError::ModelLoadError {
            path: path.clone(),
            source: source_error,
        };

        assert_eq!(
            error.to_string(),
            "Model loading failed: /path/to/model.bin"
        );
        assert!(format!("{:?}", error).contains("ModelLoadError"));
        assert!(format!("{:?}", error).contains("model.bin"));

        // Test source error chaining
        assert!(error.source().is_some());
        assert_eq!(error.source().unwrap().to_string(), "Failed to read file");
    }

    #[test]
    fn test_generation_error() {
        let error = ShimmyError::GenerationError {
            reason: "Token limit exceeded".to_string(),
        };

        assert_eq!(error.to_string(), "Generation failed: Token limit exceeded");
        assert!(format!("{:?}", error).contains("GenerationError"));
        assert!(format!("{:?}", error).contains("Token limit exceeded"));
    }

    #[test]
    fn test_config_error() {
        let error = ShimmyError::ConfigError {
            field: "temperature".to_string(),
            value: "invalid".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Invalid configuration: temperature = invalid"
        );
        assert!(format!("{:?}", error).contains("ConfigError"));
        assert!(format!("{:?}", error).contains("temperature"));
        assert!(format!("{:?}", error).contains("invalid"));
    }

    #[test]
    fn test_backend_not_available_error() {
        let error = ShimmyError::BackendNotAvailable {
            backend: "ollama".to_string(),
        };

        assert_eq!(error.to_string(), "Backend not available: ollama");
        assert!(format!("{:?}", error).contains("BackendNotAvailable"));
        assert!(format!("{:?}", error).contains("ollama"));
    }

    #[test]
    fn test_template_error() {
        let template_name = "chat_template".to_string();
        let source_error: Box<dyn StdError + Send + Sync> = Box::new(std::io::Error::new(
            ErrorKind::InvalidData,
            "Invalid template syntax",
        ));

        let error = ShimmyError::TemplateError {
            template: template_name.clone(),
            source: source_error,
        };

        assert_eq!(
            error.to_string(),
            "Template rendering failed: chat_template"
        );
        assert!(format!("{:?}", error).contains("TemplateError"));
        assert!(format!("{:?}", error).contains("chat_template"));

        // Test source error chaining
        assert!(error.source().is_some());
        assert!(error
            .source()
            .unwrap()
            .to_string()
            .contains("Invalid template syntax"));
    }

    #[test]
    fn test_async_error_from_join_error() {
        // Create a task that panics to generate a JoinError
        let rt = tokio::runtime::Runtime::new().unwrap();
        let join_handle = rt.spawn(async {
            panic!("Task panicked");
        });

        let join_error = rt.block_on(join_handle).unwrap_err();
        let error = ShimmyError::AsyncError(join_error);

        assert_eq!(error.to_string(), "Async runtime error");
        assert!(format!("{:?}", error).contains("AsyncError"));

        // Test source error chaining
        assert!(error.source().is_some());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = IoError::new(ErrorKind::NotFound, "File not found");
        let error = ShimmyError::IoError(io_error);

        assert_eq!(error.to_string(), "IO error");
        assert!(format!("{:?}", error).contains("IoError"));

        // Test source error chaining
        assert!(error.source().is_some());
        assert!(error
            .source()
            .unwrap()
            .to_string()
            .contains("File not found"));
    }

    #[test]
    fn test_io_error_from_conversion() {
        let io_error = IoError::new(ErrorKind::PermissionDenied, "Permission denied");
        let error: ShimmyError = io_error.into();

        match error {
            ShimmyError::IoError(_) => {}
            _ => panic!("Expected IoError variant"),
        }

        assert_eq!(error.to_string(), "IO error");
        assert!(error.source().is_some());
    }

    #[test]
    fn test_serde_error_conversion() {
        let json_str = r#"{"invalid": json syntax}"#;
        let serde_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let error = ShimmyError::SerdeError(serde_error);

        assert_eq!(error.to_string(), "Serialization error");
        assert!(format!("{:?}", error).contains("SerdeError"));

        // Test source error chaining
        assert!(error.source().is_some());
    }

    #[test]
    fn test_serde_error_from_conversion() {
        let invalid_json = r#"{"missing_quote: "value"}"#;
        let serde_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let error: ShimmyError = serde_error.into();

        match error {
            ShimmyError::SerdeError(_) => {}
            _ => panic!("Expected SerdeError variant"),
        }

        assert_eq!(error.to_string(), "Serialization error");
    }

    #[test]
    fn test_anyhow_error_conversion() {
        let anyhow_error = anyhow!("Something went wrong");
        let error = ShimmyError::from(anyhow_error);

        match &error {
            ShimmyError::GenerationError { reason } => {
                assert_eq!(reason, "Something went wrong");
            }
            _ => panic!("Expected GenerationError variant"),
        }

        assert_eq!(error.to_string(), "Generation failed: Something went wrong");
    }

    #[test]
    fn test_anyhow_error_from_conversion() {
        let anyhow_error = anyhow!("Custom error message");
        let error: ShimmyError = anyhow_error.into();

        match &error {
            ShimmyError::GenerationError { reason } => {
                assert_eq!(reason, "Custom error message");
            }
            _ => panic!("Expected GenerationError variant"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_shimmy_result() -> Result<String> {
            Ok("success".to_string())
        }

        fn returns_shimmy_error() -> Result<String> {
            Err(ShimmyError::ModelNotFound {
                name: "missing-model".to_string(),
            })
        }

        assert!(returns_shimmy_result().is_ok());
        assert_eq!(returns_shimmy_result().unwrap(), "success");

        assert!(returns_shimmy_error().is_err());
        let error = returns_shimmy_error().unwrap_err();
        assert!(matches!(error, ShimmyError::ModelNotFound { .. }));
    }

    #[test]
    fn test_error_debug_formatting() {
        let errors = vec![
            ShimmyError::ModelNotFound {
                name: "test".to_string(),
            },
            ShimmyError::GenerationError {
                reason: "failed".to_string(),
            },
            ShimmyError::ConfigError {
                field: "field".to_string(),
                value: "value".to_string(),
            },
            ShimmyError::BackendNotAvailable {
                backend: "backend".to_string(),
            },
        ];

        for error in errors {
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());
            assert!(debug_str.len() > 10); // Check debug string has reasonable content
        }
    }

    #[test]
    fn test_error_display_formatting() {
        let test_cases = vec![
            (
                ShimmyError::ModelNotFound {
                    name: "llama".to_string(),
                },
                "Model not found: llama",
            ),
            (
                ShimmyError::GenerationError {
                    reason: "timeout".to_string(),
                },
                "Generation failed: timeout",
            ),
            (
                ShimmyError::ConfigError {
                    field: "max_tokens".to_string(),
                    value: "-1".to_string(),
                },
                "Invalid configuration: max_tokens = -1",
            ),
            (
                ShimmyError::BackendNotAvailable {
                    backend: "openai".to_string(),
                },
                "Backend not available: openai",
            ),
        ];

        for (error, expected_message) in test_cases {
            assert_eq!(error.to_string(), expected_message);
        }
    }

    #[test]
    fn test_error_source_chain() {
        // Test ModelLoadError with source chain
        let root_cause = anyhow!("Root cause error");
        let model_error = ShimmyError::ModelLoadError {
            path: PathBuf::from("test.bin"),
            source: root_cause,
        };

        assert!(model_error.source().is_some());
        assert_eq!(
            model_error.source().unwrap().to_string(),
            "Root cause error"
        );

        // Test TemplateError with source chain
        let io_error: Box<dyn StdError + Send + Sync> =
            Box::new(IoError::new(ErrorKind::NotFound, "Template file not found"));
        let template_error = ShimmyError::TemplateError {
            template: "missing.template".to_string(),
            source: io_error,
        };

        assert!(template_error.source().is_some());
        assert!(template_error
            .source()
            .unwrap()
            .to_string()
            .contains("Template file not found"));
    }

    #[test]
    fn test_error_variants_exhaustiveness() {
        // This test ensures all error variants can be created and matched
        let errors = vec![
            ShimmyError::ModelNotFound {
                name: "test".to_string(),
            },
            ShimmyError::ModelLoadError {
                path: PathBuf::from("test.bin"),
                source: anyhow!("test error"),
            },
            ShimmyError::GenerationError {
                reason: "test".to_string(),
            },
            ShimmyError::ConfigError {
                field: "test".to_string(),
                value: "test".to_string(),
            },
            ShimmyError::BackendNotAvailable {
                backend: "test".to_string(),
            },
            ShimmyError::TemplateError {
                template: "test".to_string(),
                source: Box::new(IoError::new(ErrorKind::Other, "test")),
            },
            ShimmyError::IoError(IoError::new(ErrorKind::Other, "test")),
            ShimmyError::SerdeError(serde_json::from_str::<i32>("invalid").unwrap_err()),
        ];

        for error in errors {
            match error {
                ShimmyError::ModelNotFound { .. } => {}
                ShimmyError::ModelLoadError { .. } => {}
                ShimmyError::GenerationError { .. } => {}
                ShimmyError::ConfigError { .. } => {}
                ShimmyError::BackendNotAvailable { .. } => {}
                ShimmyError::TemplateError { .. } => {}
                ShimmyError::AsyncError(_) => {}
                ShimmyError::IoError(_) => {}
                ShimmyError::SerdeError(_) => {}
            }
        }
    }
}
