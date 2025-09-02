use thiserror::Error;
use std::path::PathBuf;

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
            reason: err.to_string() 
        }
    }
}
