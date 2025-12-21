pub mod api;
pub mod api_errors;
pub mod auto_discovery;
pub mod cli;
pub mod discovery;
pub mod discovery_invariants_only;
pub mod engine;
pub mod error;
pub mod frontend;
pub mod main_integration;
pub mod metrics;
pub mod model_manager;
pub mod model_registry;
pub mod openai_compat;
pub mod port_manager;
pub mod rustchain_compat;
pub mod safetensors_adapter;
pub mod server;
pub mod templates;
pub mod token_meter;
pub mod test_utils;
pub mod tools;
pub mod orchestrator;
// Canonical dispatcher (WS primary; HTTP adapter routes through this)
pub mod dispatcher;
pub mod util {
    pub mod diag;
    pub mod memory;
}
pub mod workflow;

// HTTP-to-WebSocket adapter (optional feature)
#[cfg(feature = "http-adapter")]
pub mod http_adapter;

pub struct AppState {
    pub engine: Box<dyn engine::InferenceEngine>,
    pub registry: model_registry::Registry,
    pub metrics: std::sync::Arc<metrics::MetricsCollector>,
    pub token_meter: std::sync::Arc<token_meter::TokenMeter>,
    pub sessions: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
    #[cfg(feature = "console")]
    pub history: std::sync::Arc<shimmy_console::history::HistoryStorage>,
}

impl AppState {
    pub fn new(
        engine: Box<dyn engine::InferenceEngine>,
        registry: model_registry::Registry,
    ) -> Self {
        #[cfg(feature = "console")]
        let history = {
            let db_base = dirs::data_dir()
                .expect("Failed to get data directory")
                .join("shimmy");
            let db_path = db_base.join("history.redb");

            std::fs::create_dir_all(&db_base).expect("Failed to create history db directory");

            // HistoryStorage::new now handles stale locks and fallback internally
            let history = shimmy_console::history::HistoryStorage::new(Some(db_path))
                .expect("Failed to create history storage (even with fallback)");

            std::sync::Arc::new(history)
        };

        // Initialize token meter
        let token_meter = {
            let db_base = dirs::data_dir()
                .expect("Failed to get data directory")
                .join("shimmy");
            let db_path = db_base.join("tokens.redb");

            std::fs::create_dir_all(&db_base).expect("Failed to create token meter directory");

            // TokenMeter::new now handles stale locks and fallback internally
            let meter = token_meter::TokenMeter::new(Some(db_path))
                .expect("Failed to create token meter (even with fallback)");

            std::sync::Arc::new(meter)
        };

        Self {
            engine,
            registry,
            metrics: metrics::MetricsCollector::new(),
            token_meter,
            sessions: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
            #[cfg(feature = "console")]
            history,
        }
    }

    pub async fn get_session_model(&self, session_id: &str) -> Option<String> {
        self.sessions.read().await.get(session_id).cloned()
    }

    pub async fn set_session_model(&self, session_id: String, model_name: String) {
        self.sessions.write().await.insert(session_id, model_name);
    }
}

// Implement Default to handle existing constructors
impl Default for AppState {
    fn default() -> Self {
        Self::new(
            Box::new(crate::engine::adapter::InferenceEngineAdapter::new()),
            model_registry::Registry::default(),
        )
    }
}

#[cfg(feature = "console")]
#[async_trait::async_trait]
impl shimmy_console::websocket::InferenceBackend for AppState {
    async fn generate_response(&self, model_name: &str, prompt: &str) -> anyhow::Result<String> {
        use crate::templates::TemplateFamily;

        eprintln!(
            "DEBUG AppState::generate_response: model={}, prompt_len={}",
            model_name,
            prompt.len()
        );

        // Create model spec for the selected model
        let model_entry = self.registry.get(model_name).ok_or_else(|| {
            let err = anyhow::anyhow!("Model '{}' not found in registry", model_name);
            eprintln!("DEBUG: Model lookup FAILED: {}", err);
            err
        })?;

        eprintln!(
            "DEBUG: Model found in registry: base_path={} template={:?}",
            model_entry.base_path.display(),
            model_entry.template
        );

        let model_spec = engine::ModelSpec {
            name: model_name.to_string(),
            base_path: model_entry.base_path.clone(),
            lora_path: model_entry.lora_path.clone(),
            template: model_entry.template.clone(),
            ctx_len: model_entry.ctx_len.unwrap_or(2048),
            n_threads: model_entry.n_threads,
        };

        // Determine template family from model config or auto-detect from model name
        let template_family = match model_entry.template.as_deref() {
            Some("llama3") | Some("llama-3") => TemplateFamily::Llama3,
            Some("phi3") | Some("phi-3") => TemplateFamily::Phi3,
            Some("phi4") | Some("phi-4") => TemplateFamily::Phi4,
            Some("mistral") => TemplateFamily::Mistral,
            Some("gemma") => TemplateFamily::Gemma,
            Some("deepseek") => TemplateFamily::DeepSeek,
            Some("vicuna") => TemplateFamily::Vicuna,
            Some("alpaca") => TemplateFamily::Alpaca,
            Some("command-r") | Some("commandr") => TemplateFamily::CommandR,
            Some("openchat") => TemplateFamily::OpenChat,
            Some("chatml") => TemplateFamily::ChatML,
            Some("raw") => TemplateFamily::Raw,
            _ => {
                let detected = TemplateFamily::detect(model_name);
                eprintln!(
                    "[DEBUG] Auto-detected template '{}' for model '{}' (non-streaming)",
                    detected.name(),
                    model_name
                );
                detected
            }
        };

        // Format the prompt using the appropriate chat template
        let system_prompt = "You are a helpful AI assistant with access to various tools for file operations, git management, and system tasks. You have tools available and can use your judgment to call them when needed to help the user. Available tools include: list_dir, read_file, run_terminal_cmd, and others for file operations.";
        let formatted_prompt = template_family.render(
            Some(system_prompt),
            &[],
            Some(prompt),
        );
        eprintln!(
            "[DEBUG] (non-streaming) formatted prompt first 200 chars: {}",
            &formatted_prompt[..formatted_prompt.len().min(200)]
        );

        eprintln!(
            "DEBUG: Loading model with spec: ctx_len={}, n_threads={:?}",
            model_spec.ctx_len, model_spec.n_threads
        );

        // Load the model using shimmy's engine
        let load_start = std::time::Instant::now();
        let loaded_model = self.engine.load(&model_spec).await.map_err(|e| {
            eprintln!("DEBUG: Model load FAILED: {}", e);
            e
        })?;
        eprintln!("DEBUG: Model loaded in {:?}", load_start.elapsed());

        // Use consistent defaults between streaming and non-streaming, relying on templates/stop tokens
        let mut gen_options = engine::GenOptions::default();
        gen_options.stream = false;
        gen_options.temperature = 0.7;
        gen_options.top_p = 0.9;
        gen_options.top_k = 40;
        gen_options.repeat_penalty = 1.1;
        gen_options.stop_tokens = template_family.stop_tokens();

        eprintln!(
            "DEBUG: Starting non-streaming generation with max_tokens={} stop_tokens={:?}",
            gen_options.max_tokens,
            gen_options.stop_tokens
        );
        let gen_start = std::time::Instant::now();

        // Generate response
        let response = loaded_model
            .generate(&formatted_prompt, gen_options, None)
            .await
            .map_err(|e| {
                eprintln!("DEBUG: Generation FAILED: {}", e);
                e
            })?;
        eprintln!(
            "DEBUG: Generation completed in {:?}, response_len={}",
            gen_start.elapsed(),
            response.len()
        );

        Ok(response)
    }

    async fn get_session_model(&self, session_id: &str) -> Option<String> {
        self.sessions.read().await.get(session_id).cloned()
    }

    async fn list_models(&self) -> anyhow::Result<Vec<(String, serde_json::Value)>> {
        // Return both manually registered models AND auto-discovered models
        let list = self.registry.list_all_available();
        let models = list
            .into_iter()
            .filter_map(|name| {
                // Try to get from manual registry first, then from discovered
                let manual_entry = self.registry.get(&name);
                let discovered = self.registry.discovered_models.get(&name);
                
                // If we have neither manual nor discovered, skip this model
                if manual_entry.is_none() && discovered.is_none() {
                    return None;
                }
                
                // Build model info from available data
                let (base_path, lora_path, template, ctx_len, n_threads) = if let Some(m) = manual_entry {
                    (m.base_path.clone(), m.lora_path.clone(), m.template.clone(), m.ctx_len, m.n_threads)
                } else if let Some(d) = discovered {
                    (d.path.clone(), d.lora_path.clone(), Some(self.registry.infer_template(&name)), Some(4096), None)
                } else {
                    return None; // Should never happen due to check above
                };
                
                let size_bytes = discovered
                    .map(|d| d.size_bytes)
                    .or_else(|| std::fs::metadata(&base_path).ok().map(|meta| meta.len()))
                    .unwrap_or(0);
                
                let parameter_count = discovered
                    .and_then(|d| d.parameter_count.clone())
                    .or_else(|| template.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                let quantization = discovered
                    .and_then(|d| d.quantization.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                // Transform to match HTTP API contract (Frontend Contract schema)
                let info = serde_json::json!({
                    "name": name.clone(),
                    "parameter_count": parameter_count,
                    "quantization": quantization,
                    "context_length": ctx_len.unwrap_or(2048),
                    "size_bytes": size_bytes,
                    "model_type": "gguf",
                    "loaded": false,
                    "supported_features": ["generate", "stream"],
                    "source": if manual_entry.is_some() { "registered" } else { "discovered" },
                    // Legacy fields for backwards compatibility
                    "base_path": base_path.to_string_lossy(),
                    "lora_path": lora_path.as_ref().map(|p| p.to_string_lossy()),
                    "template": template,
                    "ctx_len": ctx_len,
                    "n_threads": n_threads,
                });
                Some((name.clone(), info))
            })
            .collect();
        Ok(models)
    }

    async fn set_session_model(&self, session_id: &str, model_name: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.to_string(), model_name.to_string());
        Ok(())
    }

    async fn get_metrics(&self) -> anyhow::Result<serde_json::Value> {
        let metrics = self.metrics.get_metrics();
        Ok(serde_json::to_value(&metrics).unwrap_or_default())
    }

    async fn generate_stream(&self, model_name: &str, prompt: &str, tx: tokio::sync::mpsc::Sender<String>) -> anyhow::Result<()> {
        use std::sync::atomic::Ordering;
        use crate::templates::TemplateFamily;

        // Find model entry
        let model_entry = self.registry.get(model_name).ok_or_else(|| {
            let err = anyhow::anyhow!(format!("Model '{}' not found in registry", model_name));
            eprintln!("DEBUG: Model lookup FAILED: {}", err);
            err
        })?;

        let model_spec = engine::ModelSpec {
            name: model_name.to_string(),
            base_path: model_entry.base_path.clone(),
            lora_path: model_entry.lora_path.clone(),
            template: model_entry.template.clone(),
            ctx_len: model_entry.ctx_len.unwrap_or(2048),
            n_threads: model_entry.n_threads,
        };

        // Determine template family from model config or auto-detect from model name
        let template_family = match model_entry.template.as_deref() {
            Some("llama3") | Some("llama-3") => TemplateFamily::Llama3,
            Some("phi3") | Some("phi-3") => TemplateFamily::Phi3,
            Some("phi4") | Some("phi-4") => TemplateFamily::Phi4,
            Some("mistral") => TemplateFamily::Mistral,
            Some("gemma") => TemplateFamily::Gemma,
            Some("deepseek") => TemplateFamily::DeepSeek,
            Some("vicuna") => TemplateFamily::Vicuna,
            Some("alpaca") => TemplateFamily::Alpaca,
            Some("command-r") | Some("commandr") => TemplateFamily::CommandR,
            Some("openchat") => TemplateFamily::OpenChat,
            Some("chatml") => TemplateFamily::ChatML,
            Some("raw") => TemplateFamily::Raw,
            _ => {
                // Auto-detect template from model name
                let detected = TemplateFamily::detect(model_name);
                eprintln!("[DEBUG] Auto-detected template '{}' for model '{}'", detected.name(), model_name);
                detected
            }
        };

        // Format the prompt using the appropriate chat template
        let system_prompt = "You are a helpful AI assistant with access to various tools for file operations, git management, and system tasks. You have tools available and can use your judgment to call them when needed to help the user. Available tools include: list_dir, read_file, run_terminal_cmd, and others for file operations.";
        let formatted_prompt = template_family.render(
            Some(system_prompt),
            &[],
            Some(prompt),
        );
        eprintln!("[DEBUG] Formatted prompt (first 200 chars): {}", &formatted_prompt[..formatted_prompt.len().min(200)]);

        // Load model
        let loaded = self.engine.load(&model_spec).await.map_err(|e| {
            eprintln!("DEBUG: Model load FAILED: {}", e);
            e
        })?;

        // Options for streaming generation - use template-specific stop tokens
        let mut gen_options = engine::GenOptions::default();
        gen_options.stream = true;
        // Remove hard caps to allow unlimited generation as requested
        gen_options.max_tokens = usize::MAX;
        // Adjust generation settings for better response quality
        gen_options.temperature = 0.1;  // Low but not zero for some creativity
        gen_options.top_p = 0.9;
        gen_options.top_k = 40;  // Allow more token choices
        gen_options.repeat_penalty = 1.1;  // Lighter penalty
        // Keep stop tokens so model can stop naturally, but allow unlimited length
        gen_options.stop_tokens = template_family.stop_tokens();

        // Rough input token estimate for token-meter accounting
        let input_tokens = (formatted_prompt.split_whitespace().count() as f64 * 1.3) as u64;

        // Spawn a task to run generation and forward tokens into tx
        let metrics_clone = self.metrics.clone();
        let token_meter_clone = self.token_meter.clone();
        let session_id = "websocket-session".to_string();
        let prompt_owned = formatted_prompt;  // Use the formatted prompt with chat template

        tokio::spawn(async move {
            let token_count = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
            let token_count_cb = token_count.clone();

            // Callback invoked inline by engine - use try_send to avoid async inside callback
            let tx_for_cb = tx.clone();
            let on_token: Option<Box<dyn FnMut(String) + Send>> = Some(Box::new(move |tok: String| {
                token_count_cb.fetch_add(1, Ordering::Relaxed);
                // Emit canonical streaming shape for console/themes
                let msg = serde_json::json!({"token": tok}).to_string();
                let _ = tx_for_cb.try_send(msg);
            }));

            if let Err(e) = loaded.generate(&prompt_owned, gen_options, on_token).await {
                eprintln!("Streaming generation failed: {}", e);
                metrics_clone.record_error();
                // Send error token in canonical shape
                let err_msg = serde_json::json!({"token": format!("Error: {}", e)}).to_string();
                let _ = tx.send(err_msg).await;
            } else {
                let output_tokens = token_count.load(Ordering::Relaxed);
                metrics_clone.record_tokens(output_tokens);
                let _ = token_meter_clone.record_generation(&session_id, input_tokens, output_tokens);
            }

            // Inform receiver that stream is complete using canonical shape
            let done_msg = serde_json::json!({"done": true}).to_string();
            let _ = tx.send(done_msg).await;
        });

        Ok(())
    }
}
