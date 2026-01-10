// Suppress function pointer comparison warnings from auto-generated bindings
#![allow(unpredictable_function_pointer_comparisons)]

pub mod anthropic_compat;
pub mod api;
pub mod api_errors;
pub mod auto_discovery;
pub mod cache;
pub mod cli;
pub mod discovery;
pub mod engine;
pub mod error;
pub mod main_integration;
pub mod metrics;
pub mod model_manager;
pub mod model_registry;
pub mod observability;
pub mod openai_compat;
pub mod port_manager;
pub mod rustchain_compat;
pub mod safetensors_adapter;
pub mod server;
pub mod templates;
pub mod tools;
#[cfg(feature = "vision")]
pub mod vision;
#[cfg(feature = "vision")]
pub mod vision_license;
pub mod util {
    pub mod diag;
    pub mod memory;
}
pub mod invariant_ppt;
pub mod workflow;

#[cfg(test)]
pub mod tests;

pub mod test_utils;

// Note: Mock infrastructure removed - use real testing with local models
// PPT + Invariant Testing System ensures semantic integrity under high-visibility development

pub struct AppState {
    pub engine: Box<dyn engine::InferenceEngine>,
    pub registry: model_registry::Registry,
    pub observability: observability::ObservabilityManager,
    pub response_cache: cache::ResponseCache,
    #[cfg(feature = "vision")]
    pub vision_license_manager: Option<crate::vision_license::VisionLicenseManager>,
}

impl AppState {
    pub fn new(
        engine: Box<dyn engine::InferenceEngine>,
        registry: model_registry::Registry,
    ) -> Self {
        Self {
            engine,
            registry,
            observability: observability::ObservabilityManager::new(),
            response_cache: cache::ResponseCache::new(),
            #[cfg(feature = "vision")]
            vision_license_manager: Some(crate::vision_license::VisionLicenseManager::new()),
        }
    }
}
