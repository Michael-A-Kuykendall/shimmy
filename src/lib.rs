pub mod api;
pub mod api_errors;
pub mod auto_discovery;
pub mod cli;
pub mod discovery;
pub mod engine;
pub mod main_integration;
pub mod metrics;
pub mod model_manager;
pub mod model_registry;
pub mod openai_compat;
pub mod rustchain_compat;
pub mod safetensors_adapter;
pub mod server;
pub mod templates;
pub mod tools;
pub mod util {
    pub mod diag;
}

pub struct AppState {
    pub engine: Box<dyn engine::InferenceEngine>,
    pub registry: model_registry::Registry,
}