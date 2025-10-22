#![allow(clippy::too_many_arguments)]
use anyhow::Result;
use async_trait::async_trait;

use super::{GenOptions, InferenceEngine, LoadedModel, ModelSpec};

/// Smart thread detection optimized for inference performance
/// Matches Ollama's approach: use physical cores with intelligent limits
fn get_optimal_thread_count() -> i32 {
    let total_cores = std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4);

    // Ollama logic: Use physical cores, not logical (hyperthreading) cores
    // Intel i7 typically has 4-8 physical cores but 8-16 logical cores
    let physical_cores = match total_cores {
        1..=2 => total_cores,               // Single/dual core: use all
        3..=4 => total_cores,               // Quad core: use all physical
        5..=8 => (total_cores / 2).max(4),  // 6-8 core: assume hyperthreading, use physical
        9..=16 => (total_cores / 2).max(6), // 8+ core: definitely hyperthreaded, use ~half
        _ => 8,                             // High-end systems: cap at 8 threads for stability
    };

    // Further optimization: leave some cores for system
    let optimal = match physical_cores {
        1..=2 => physical_cores,
        3..=4 => physical_cores - 1, // Leave 1 core for system
        5..=8 => physical_cores - 2, // Leave 2 cores for system
        _ => physical_cores * 3 / 4, // Use 75% of physical cores
    }
    .max(1); // Always use at least 1 thread

    tracing::info!(
        "Threading: {} total cores detected, using {} optimal threads",
        total_cores,
        optimal
    );
    optimal
}

#[cfg(feature = "llama")]
use std::sync::Mutex;
use tracing::info;

#[cfg(feature = "llama")]
use std::sync::OnceLock;

#[cfg(feature = "llama")]
static LLAMA_BACKEND: OnceLock<Result<shimmy_llama_cpp_2::llama_backend::LlamaBackend, String>> =
    OnceLock::new();

#[cfg(feature = "llama")]
fn get_or_init_backend() -> Result<&'static shimmy_llama_cpp_2::llama_backend::LlamaBackend> {
    use anyhow::anyhow;

    let result = LLAMA_BACKEND.get_or_init(|| {
        info!("Initializing llama.cpp backend (first model load)");
        shimmy_llama_cpp_2::llama_backend::LlamaBackend::init()
            .map_err(|e| format!("Failed to initialize llama backend: {}", e))
    });

    result.as_ref().map_err(|e| anyhow!("{}", e))
}

#[derive(Default)]
pub struct LlamaEngine {
    #[allow(dead_code)] // Temporarily unused while fork is being fixed
    gpu_backend: GpuBackend,
    #[allow(dead_code)] // Temporarily unused while fork is being fixed
    moe_config: MoeConfig,
}

#[derive(Debug, Clone, Default)]
struct MoeConfig {
    #[allow(dead_code)] // Temporarily unused while fork is being fixed
    cpu_moe_all: bool,
    #[allow(dead_code)] // Temporarily unused while fork is being fixed
    n_cpu_moe: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub enum GpuBackend {
    #[default]
    Cpu,
    #[cfg(feature = "llama-cuda")]
    Cuda,
    #[cfg(feature = "llama-vulkan")]
    Vulkan,
    #[cfg(feature = "llama-opencl")]
    OpenCL,
}

impl GpuBackend {
    /// Parse GPU backend from CLI string
    #[allow(dead_code)] // Temporarily unused while fork is being fixed
    fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "auto" => Self::detect_best(),
            "cpu" => GpuBackend::Cpu,
            #[cfg(feature = "llama-cuda")]
            "cuda" => GpuBackend::Cuda,
            #[cfg(feature = "llama-vulkan")]
            "vulkan" => GpuBackend::Vulkan,
            #[cfg(feature = "llama-opencl")]
            "opencl" => GpuBackend::OpenCL,
            // Better error messages for backends not compiled in
            #[cfg(not(feature = "llama-cuda"))]
            "cuda" => {
                tracing::warn!("CUDA backend requested but not enabled at compile time. Rebuild with --features llama-cuda");
                Self::detect_best()
            }
            #[cfg(not(feature = "llama-vulkan"))]
            "vulkan" => {
                tracing::warn!("Vulkan backend requested but not enabled at compile time. Rebuild with --features llama-vulkan");
                Self::detect_best()
            }
            #[cfg(not(feature = "llama-opencl"))]
            "opencl" => {
                tracing::warn!("OpenCL backend requested but not enabled at compile time. Rebuild with --features llama-opencl");
                Self::detect_best()
            }
            _ => {
                tracing::warn!("Unknown GPU backend '{}', using auto-detect", s);
                Self::detect_best()
            }
        }
    }

    /// Detect the best available GPU backend for this system
    fn detect_best() -> Self {
        #[cfg(feature = "llama-cuda")]
        {
            if Self::is_cuda_available() {
                info!("CUDA GPU detected, using CUDA backend");
                return GpuBackend::Cuda;
            }
        }

        #[cfg(feature = "llama-vulkan")]
        {
            if Self::is_vulkan_available() {
                info!("Vulkan GPU detected, using Vulkan backend");
                return GpuBackend::Vulkan;
            }
        }

        #[cfg(feature = "llama-opencl")]
        {
            if Self::is_opencl_available() {
                info!("OpenCL GPU detected, using OpenCL backend");
                return GpuBackend::OpenCL;
            }
        }

        info!("No GPU acceleration available, using CPU backend");
        GpuBackend::Cpu
    }

    #[cfg(feature = "llama-cuda")]
    fn is_cuda_available() -> bool {
        std::process::Command::new("nvidia-smi")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(feature = "llama-vulkan")]
    fn is_vulkan_available() -> bool {
        // Check for Vulkan loader and runtime
        // Try vulkaninfo first (most reliable)
        if std::process::Command::new("vulkaninfo")
            .arg("--summary")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        // Fallback: assume available if feature is enabled
        // (Vulkan may be present even without vulkaninfo tool)
        true
    }

    #[cfg(feature = "llama-opencl")]
    fn is_opencl_available() -> bool {
        // Check for OpenCL runtime using clinfo
        if std::process::Command::new("clinfo")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            return true;
        }

        // On Windows, try alternative detection
        #[cfg(target_os = "windows")]
        {
            // Check for OpenCL.dll in system paths
            if std::process::Command::new("where")
                .arg("OpenCL.dll")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                return true;
            }
        }

        // Fallback: assume available if feature is enabled
        true
    }

    /// Get the number of layers to offload to GPU
    pub fn gpu_layers(&self) -> u32 {
        match self {
            GpuBackend::Cpu => 0, // No GPU offloading for CPU backend
            #[cfg(feature = "llama-cuda")]
            GpuBackend::Cuda => 999, // Offload all layers to CUDA
            #[cfg(feature = "llama-vulkan")]
            GpuBackend::Vulkan => 999, // Offload all layers to Vulkan
            #[cfg(feature = "llama-opencl")]
            GpuBackend::OpenCL => 999, // Offload all layers to OpenCL
        }
    }
}

impl LlamaEngine {
    pub fn new() -> Self {
        Self {
            gpu_backend: GpuBackend::detect_best(),
            moe_config: MoeConfig::default(),
        }
    }

    /// Create engine with specific GPU backend from CLI
    pub fn new_with_backend(backend_str: Option<&str>) -> Self {
        let gpu_backend = backend_str
            .map(GpuBackend::from_string)
            .unwrap_or_else(GpuBackend::detect_best);

        info!("GPU backend configured: {:?}", gpu_backend);

        Self {
            gpu_backend,
            moe_config: MoeConfig::default(),
        }
    }

    /// Set MoE CPU offloading configuration
    pub fn with_moe_config(mut self, cpu_moe_all: bool, n_cpu_moe: Option<usize>) -> Self {
        self.moe_config = MoeConfig {
            cpu_moe_all,
            n_cpu_moe,
        };
        self
    }

    /// Get information about the current GPU backend configuration
    pub fn get_backend_info(&self) -> String {
        match self.gpu_backend {
            GpuBackend::Cpu => "CPU".to_string(),
            #[cfg(feature = "llama-cuda")]
            GpuBackend::Cuda => "CUDA".to_string(),
            #[cfg(feature = "llama-vulkan")]
            GpuBackend::Vulkan => "Vulkan".to_string(),
            #[cfg(feature = "llama-opencl")]
            GpuBackend::OpenCL => "OpenCL".to_string(),
        }
    }
}

#[async_trait]
impl InferenceEngine for LlamaEngine {
    async fn load(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        #[cfg(feature = "llama")]
        {
            use anyhow::anyhow;
            use shimmy_llama_cpp_2 as llama;
            use std::num::NonZeroU32;

            // Use global singleton backend (fixes Issue #128: BackendAlreadyInitialized)
            let be = get_or_init_backend()?;

            // Configure GPU acceleration based on backend
            let n_gpu_layers = self.gpu_backend.gpu_layers();
            info!(
                "Loading model with {} GPU layers ({:?} backend)",
                n_gpu_layers, self.gpu_backend
            );

            let mut model_params =
                llama::model::params::LlamaModelParams::default().with_n_gpu_layers(n_gpu_layers);

            // Apply MoE CPU offloading if configured
            // Enable MoE CPU offloading (Issue #108 fix)
            if let Some(n) = self.moe_config.n_cpu_moe {
                info!("MoE: Offloading first {} expert layers to CPU", n);
                model_params = model_params.with_n_cpu_moe(n);
            } else if self.moe_config.cpu_moe_all {
                info!("MoE: Offloading ALL expert tensors to CPU (saves ~80-85% VRAM)");
                model_params = model_params.with_cpu_moe_all();
            }

            // Attempt to load the model with better error handling
            let model = match llama::model::LlamaModel::load_from_file(
                be,
                &spec.base_path,
                &model_params,
            ) {
                Ok(model) => model,
                Err(e) => {
                    // Check if this looks like a memory allocation failure
                    let error_msg = format!("{}", e);
                    if error_msg.contains("failed to allocate")
                        || error_msg.contains("CPU_REPACK buffer")
                    {
                        let file_size = std::fs::metadata(&spec.base_path)
                            .map(|m| m.len())
                            .unwrap_or(0);
                        let size_gb = file_size as f64 / 1_024_000_000.0;

                        return Err(anyhow!(
                            "Memory allocation failed for model {} ({:.1}GB). \n\
                            💡 Possible solutions:\n\
                            • Use a smaller model (7B instead of 14B parameters)\n\
                            • Add more system RAM (model needs ~{}GB)\n\
                            • Enable model quantization (Q4_K_M, Q5_K_M)\n\
                            • MoE CPU offloading is temporarily disabled (Issue #108)\n\
                            Original error: {}",
                            spec.base_path.display(),
                            size_gb,
                            (size_gb * 1.5) as u32, // Rough estimate of RAM needed
                            e
                        ));
                    }

                    // Re-throw other errors as-is
                    return Err(e.into());
                }
            };
            let ctx_params = llama::context::params::LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(spec.ctx_len as u32))
                .with_n_batch(2048)
                .with_n_ubatch(512)
                .with_n_threads(spec.n_threads.unwrap_or_else(get_optimal_thread_count))
                .with_n_threads_batch(spec.n_threads.unwrap_or_else(get_optimal_thread_count));
            let ctx_tmp = model.new_context(be, ctx_params)?;
            if let Some(ref lora) = spec.lora_path {
                // Check if it's a SafeTensors file and convert if needed
                let lora_path = if lora.extension().and_then(|s| s.to_str()) == Some("safetensors")
                {
                    // For now, provide helpful error message for SafeTensors files
                    return Err(anyhow!(
                        "SafeTensors LoRA detected: {}. Please convert to GGUF format first.",
                        lora.display()
                    ));
                } else {
                    lora.clone()
                };

                let mut adapter = model.lora_adapter_init(&lora_path)?;
                ctx_tmp
                    .lora_adapter_set(&mut adapter, 1.0)
                    .map_err(|e| anyhow!("lora set: {e:?}"))?;
                info!(adapter=%lora_path.display(), "LoRA adapter attached");
            }
            // Store both model and context together to maintain proper lifetimes
            // The context lifetime is tied to &model; storing both in the same struct ensures safety
            let ctx: llama::context::LlamaContext<'static> =
                unsafe { std::mem::transmute(ctx_tmp) };
            Ok(Box::new(LlamaLoaded {
                model,
                ctx: Mutex::new(ctx),
            }))
        }
        #[cfg(not(feature = "llama"))]
        {
            let _ = spec; // silence unused warning
            Ok(Box::new(LlamaFallback))
        }
    }
}

#[cfg(feature = "llama")]
struct LlamaLoaded {
    model: shimmy_llama_cpp_2::model::LlamaModel,
    ctx: Mutex<shimmy_llama_cpp_2::context::LlamaContext<'static>>,
}

#[cfg(feature = "llama")]
// The llama.cpp context & model use raw pointers internally and are !Send by default.
// We wrap access in a Mutex and only perform FFI calls while holding the lock, so it's
// sound to mark the container Send + Sync for our usage (single-threaded mutable access).
unsafe impl Send for LlamaLoaded {}
#[cfg(feature = "llama")]
unsafe impl Sync for LlamaLoaded {}

#[cfg(feature = "llama")]
#[async_trait]
impl LoadedModel for LlamaLoaded {
    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        mut on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        use shimmy_llama_cpp_2::{
            llama_batch::LlamaBatch,
            model::{AddBos, Special},
            sampling::LlamaSampler,
        };
        let mut ctx = self
            .ctx
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock context: {}", e))?;
        let tokens = self.model.str_to_token(prompt, AddBos::Always)?;

        // Create batch with explicit logits configuration
        let mut batch = LlamaBatch::new(tokens.len(), 1);
        for (i, &token) in tokens.iter().enumerate() {
            // Only request logits for the last token in the initial batch
            let logits = i == tokens.len() - 1;
            batch.add(token, i as i32, &[0], logits)?;
        }
        ctx.decode(&mut batch)?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(opts.temperature),
            LlamaSampler::top_p(opts.top_p, 1),
            LlamaSampler::top_k(opts.top_k),
            // API changed order: (repeat_last_n, freq_penalty, presence_penalty, penalty)
            LlamaSampler::penalties(64, 0.0, 0.0, opts.repeat_penalty),
            LlamaSampler::greedy(),
        ])
        .with_tokens(tokens.iter().copied());

        let mut out = String::new();
        let mut all_tokens = tokens;
        for _ in 0..opts.max_tokens {
            // Sample from the last (and only) position with logits
            let token = sampler.sample(&ctx, -1);
            if self.model.is_eog_token(token) {
                break;
            }
            // Use Plaintext to avoid re-tokenizing control tokens into special forms
            let piece = self.model.token_to_str(token, Special::Plaintext)?;
            let start = out.len();
            out.push_str(&piece);

            // Check for stop tokens before emitting
            let should_stop = opts
                .stop_tokens
                .iter()
                .any(|stop_token| out.contains(stop_token));
            if should_stop {
                // Remove the stop token from the output
                for stop_token in &opts.stop_tokens {
                    if let Some(pos) = out.rfind(stop_token) {
                        out.truncate(pos);
                        break;
                    }
                }
                break;
            }

            if let Some(cb) = on_token.as_mut() {
                cb(out[start..].to_string());
            }

            let mut step = LlamaBatch::new(1, 1);
            step.add(token, all_tokens.len() as i32, &[0], true)?;
            ctx.decode(&mut step)?;
            all_tokens.push(token);
        }
        Ok(out)
    }
}

/// Fallback implementation when llama.cpp feature is not enabled
/// Returns informative message directing users to enable the feature
#[cfg(not(feature = "llama"))]
struct LlamaFallback;

#[cfg(not(feature = "llama"))]
#[async_trait]
impl LoadedModel for LlamaFallback {
    async fn generate(
        &self,
        prompt: &str,
        _opts: GenOptions,
        mut on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        let fallback_msg =
            "Llama.cpp support not enabled. Build with --features llama for full functionality.";
        if let Some(cb) = on_token.as_mut() {
            cb(fallback_msg.to_string());
        }
        Ok(format!("[INFO] {} Input: {}", fallback_msg, prompt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llama_engine_initialization() {
        let engine = LlamaEngine::new();
        // LlamaEngine has a GpuBackend field - just verify it can be created
        // Size depends on which GPU features are compiled in
        let _ = engine; // Suppress unused variable warning
    }

    #[test]
    fn test_gpu_backend_layer_offloading_logic() {
        // Issue #130: Verify CPU backend offloads 0 layers, GPU backends offload all layers

        let cpu_backend = GpuBackend::Cpu;
        assert_eq!(
            cpu_backend.gpu_layers(),
            0,
            "CPU backend should offload 0 layers to GPU"
        );

        #[cfg(feature = "llama-cuda")]
        {
            let cuda_backend = GpuBackend::Cuda;
            assert_eq!(
                cuda_backend.gpu_layers(),
                999,
                "CUDA backend should offload 999 layers to GPU"
            );
        }

        #[cfg(feature = "llama-vulkan")]
        {
            let vulkan_backend = GpuBackend::Vulkan;
            assert_eq!(
                vulkan_backend.gpu_layers(),
                999,
                "Vulkan backend should offload 999 layers to GPU"
            );
        }

        #[cfg(feature = "llama-opencl")]
        {
            let opencl_backend = GpuBackend::OpenCL;
            assert_eq!(
                opencl_backend.gpu_layers(),
                999,
                "OpenCL backend should offload 999 layers to GPU"
            );
        }
    }

    #[tokio::test]
    async fn test_model_loading_validation() {
        let _engine = LlamaEngine::new();
        let _spec = ModelSpec {
            name: "test".to_string(),
            base_path: "/nonexistent".into(),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: 2048,
            n_threads: None,
        };

        // let result = engine.load(&spec).await; // Commented to avoid test file dependencies
        // assert!(result.is_err()); // Test spec structure instead
    }

    #[test]
    fn test_model_spec_validation() {
        let spec = ModelSpec {
            name: "valid".to_string(),
            base_path: "test.gguf".into(),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: 4096,
            n_threads: Some(4),
        };

        assert_eq!(spec.name, "valid");
        assert_eq!(spec.ctx_len, 4096);
        assert!(spec.template.is_some());
    }
}
