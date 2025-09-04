use std::path::PathBuf;
use crate::engine::{GenOptions, ModelBackend, UniversalModelSpec};

/// Builder pattern for creating model specifications
#[derive(Debug, Default)]
pub struct ModelSpecBuilder {
    name: Option<String>,
    backend: Option<ModelBackend>,
    template: Option<String>,
    ctx_len: Option<usize>,
    device: Option<String>,
    n_threads: Option<i32>,
}

impl ModelSpecBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn llama_backend<P: Into<PathBuf>>(self, base_path: P) -> LlamaBackendBuilder {
        LlamaBackendBuilder {
            spec_builder: self,
            base_path: base_path.into(),
            lora_path: None,
        }
    }
    
    pub fn huggingface_backend<S: Into<String>>(self, model_id: S) -> HuggingFaceBackendBuilder {
        HuggingFaceBackendBuilder {
            spec_builder: self,
            base_model_id: model_id.into(),
            peft_path: None,
            use_local: false,
        }
    }
    
    pub fn template<S: Into<String>>(mut self, template: S) -> Self {
        self.template = Some(template.into());
        self
    }
    
    pub fn context_length(mut self, ctx_len: usize) -> Self {
        self.ctx_len = Some(ctx_len);
        self
    }
    
    pub fn device<S: Into<String>>(mut self, device: S) -> Self {
        self.device = Some(device.into());
        self
    }
    
    pub fn threads(mut self, n_threads: i32) -> Self {
        self.n_threads = Some(n_threads);
        self
    }
    
    pub fn build(self) -> crate::error::Result<UniversalModelSpec> {
        Ok(UniversalModelSpec {
            name: self.name.ok_or_else(|| crate::error::ShimmyError::ConfigError {
                field: "name".to_string(),
                value: "missing".to_string(),
            })?,
            backend: self.backend.ok_or_else(|| crate::error::ShimmyError::ConfigError {
                field: "backend".to_string(),
                value: "missing".to_string(),
            })?,
            template: self.template,
            ctx_len: self.ctx_len.unwrap_or(4096),
            device: self.device.unwrap_or_else(|| "cpu".to_string()),
            n_threads: self.n_threads,
        })
    }
}

/// Builder for LLAMA backend configuration
pub struct LlamaBackendBuilder {
    spec_builder: ModelSpecBuilder,
    base_path: PathBuf,
    lora_path: Option<PathBuf>,
}

impl LlamaBackendBuilder {
    pub fn lora_adapter<P: Into<PathBuf>>(mut self, lora_path: P) -> Self {
        self.lora_path = Some(lora_path.into());
        self
    }
    
    pub fn template<S: Into<String>>(mut self, template: S) -> Self {
        self.spec_builder = self.spec_builder.template(template);
        self
    }
    
    pub fn context_length(mut self, ctx_len: usize) -> Self {
        self.spec_builder = self.spec_builder.context_length(ctx_len);
        self
    }
    
    pub fn device<S: Into<String>>(mut self, device: S) -> Self {
        self.spec_builder = self.spec_builder.device(device);
        self
    }
    
    pub fn threads(mut self, n_threads: i32) -> Self {
        self.spec_builder = self.spec_builder.threads(n_threads);
        self
    }
    
    pub fn build(mut self) -> crate::error::Result<UniversalModelSpec> {
        self.spec_builder.backend = Some(ModelBackend::LlamaGGUF {
            base_path: self.base_path,
            lora_path: self.lora_path,
        });
        self.spec_builder.build()
    }
}

/// Builder for HuggingFace backend configuration
pub struct HuggingFaceBackendBuilder {
    spec_builder: ModelSpecBuilder,
    base_model_id: String,
    peft_path: Option<PathBuf>,
    use_local: bool,
}

impl HuggingFaceBackendBuilder {
    pub fn peft_adapter<P: Into<PathBuf>>(mut self, peft_path: P) -> Self {
        self.peft_path = Some(peft_path.into());
        self
    }
    
    pub fn use_local(mut self, use_local: bool) -> Self {
        self.use_local = use_local;
        self
    }
    
    pub fn template<S: Into<String>>(mut self, template: S) -> Self {
        self.spec_builder = self.spec_builder.template(template);
        self
    }
    
    pub fn context_length(mut self, ctx_len: usize) -> Self {
        self.spec_builder = self.spec_builder.context_length(ctx_len);
        self
    }
    
    pub fn device<S: Into<String>>(mut self, device: S) -> Self {
        self.spec_builder = self.spec_builder.device(device);
        self
    }
    
    pub fn build(mut self) -> crate::error::Result<UniversalModelSpec> {
        self.spec_builder.backend = Some(ModelBackend::HuggingFace {
            base_model_id: self.base_model_id,
            peft_path: self.peft_path,
            use_local: self.use_local,
        });
        self.spec_builder.build()
    }
}

/// Builder for generation options
#[derive(Debug, Default)]
pub struct GenOptionsBuilder {
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<i32>,
    repeat_penalty: Option<f32>,
    seed: Option<u32>,
    stream: Option<bool>,
}

impl GenOptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
    
    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }
    
    pub fn repeat_penalty(mut self, repeat_penalty: f32) -> Self {
        self.repeat_penalty = Some(repeat_penalty);
        self
    }
    
    pub fn seed(mut self, seed: u32) -> Self {
        self.seed = Some(seed);
        self
    }
    
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
    
    pub fn build(self) -> GenOptions {
        GenOptions {
            max_tokens: self.max_tokens.unwrap_or(1024),
            temperature: self.temperature.unwrap_or(0.7),
            top_p: self.top_p.unwrap_or(0.9),
            top_k: self.top_k.unwrap_or(40),
            repeat_penalty: self.repeat_penalty.unwrap_or(1.1),
            seed: self.seed,
            stream: self.stream.unwrap_or(false),
        }
    }
}
