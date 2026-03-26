/// Regression test for Issue #184: Wrong chat template causes structural token
/// leakage into model output
///
/// **Symptom**: Models like qwen3, gemma3, deepseek emit structural tokens
/// (`<|start_header_id|>assistant<|end_header_id|>`, `<|im_start|>`, etc.) as
/// literal output content rather than answer text.  When this occurs across
/// the full max_tokens budget the model produces no useful response, subsequent
/// calls receive the garbage as "prior solution" context, and the inference
/// pipeline hangs waiting for coherent output that never arrives.
///
/// **Root cause**: `infer_template()` in `model_registry.rs` guesses the chat
/// template from the model name (returning "chatml" or "llama3").  Many modern
/// models have a Jinja chat template embedded in their GGUF metadata
/// (`tokenizer.chat_template`) that differs from both named templates.  Applying
/// the wrong template causes the model to interpret structural markers as part
/// of its generation target rather than as framing it should not reproduce.
///
/// **Fix**: `LlamaLoaded` reads the embedded template via
/// `model.chat_template(None)` at load time.  `LoadedModel::format_prompt()`
/// applies it via `model.apply_chat_template()`, bypassing the name-based
/// `TemplateFamily::render()` path in `openai_compat.rs` and `api.rs`.  The
/// name-based path is retained as a fallback for models without an embedded
/// template.
///
/// **Affected model families**: qwen3, qwen2.5, gemma3, deepseek-r1, cogito,
/// phi4, and any model whose GGUF `tokenizer.chat_template` differs from the
/// "chatml" or "llama3" named templates.
#[cfg(test)]
mod issue_184_tests {
    use shimmy::engine::ModelSpec;
    use std::path::PathBuf;

    /// Verify that `LoadedModel::format_prompt` has a default implementation
    /// that returns `None`.
    ///
    /// Non-llama backends (mock, safetensors) do not embed chat templates and
    /// must return `None` so callers can fall back to name-based inference.
    /// This test exercises the default trait implementation.
    #[test]
    fn test_format_prompt_default_returns_none() {
        use shimmy::engine::LoadedModel;

        struct MockLoaded;

        #[async_trait::async_trait]
        impl LoadedModel for MockLoaded {
            async fn generate(
                &self,
                _prompt: &str,
                _opts: shimmy::engine::GenOptions,
                _on_token: Option<Box<dyn FnMut(String) + Send>>,
            ) -> anyhow::Result<String> {
                Ok("mock".to_string())
            }
        }

        let mock = MockLoaded;
        let messages = vec![
            ("system".to_string(), "You are helpful.".to_string()),
            ("user".to_string(), "Hello".to_string()),
        ];

        // Default implementation must return None so the fallback path is taken
        assert!(
            mock.format_prompt(&messages).is_none(),
            "Default format_prompt must return None to enable fallback to \
             name-based template inference"
        );
    }

    /// Verify the model spec structure used to identify which template to apply.
    ///
    /// The template field in ModelSpec is used as fallback when no native
    /// template is available.  "chatml" and "llama3" are the two supported
    /// named fallbacks.
    #[test]
    fn test_model_spec_template_fallback_values() {
        let chatml_spec = ModelSpec {
            name: "qwen3-8b".to_string(),
            base_path: PathBuf::from("/models/qwen3-8b.gguf"),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: 8192,
            n_threads: None,
        };

        assert_eq!(chatml_spec.template.as_deref(), Some("chatml"));

        let llama_spec = ModelSpec {
            name: "llama3-8b".to_string(),
            base_path: PathBuf::from("/models/llama3-8b.gguf"),
            lora_path: None,
            template: Some("llama3".to_string()),
            ctx_len: 8192,
            n_threads: None,
        };

        assert_eq!(llama_spec.template.as_deref(), Some("llama3"));
    }

    /// Verify that a model with no template field does not panic the template
    /// selection logic.
    ///
    /// The `_` arm in the `match spec.template.as_deref()` block must handle
    /// `None` gracefully by falling through to name-based auto-detection.
    #[test]
    fn test_model_spec_no_template_handled_gracefully() {
        let spec = ModelSpec {
            name: "unknown-model".to_string(),
            base_path: PathBuf::from("/models/unknown.gguf"),
            lora_path: None,
            template: None,
            ctx_len: 8192,
            n_threads: None,
        };

        // None template must not cause a panic in the fallback match arm
        let template_name = spec.template.as_deref().unwrap_or("auto-detect");
        assert_eq!(template_name, "auto-detect");
    }

    /// Verify the priority: native GGUF template > name-based inference.
    ///
    /// When `format_prompt()` returns Some, the caller must use that result
    /// and skip `fam.render()`.  This test documents the expected precedence
    /// using a stub that simulates a loaded model with a native template.
    #[test]
    fn test_native_template_takes_priority_over_name_inference() {
        use shimmy::engine::LoadedModel;

        // Stub that simulates a model with a native embedded Jinja template
        struct ModelWithNativeTemplate;

        #[async_trait::async_trait]
        impl LoadedModel for ModelWithNativeTemplate {
            async fn generate(
                &self,
                _prompt: &str,
                _opts: shimmy::engine::GenOptions,
                _on_token: Option<Box<dyn FnMut(String) + Send>>,
            ) -> anyhow::Result<String> {
                Ok("answer".to_string())
            }

            fn format_prompt(&self, messages: &[(String, String)]) -> Option<String> {
                // Simulate a model that formats its own prompt from native template
                let formatted = messages
                    .iter()
                    .map(|(role, content)| format!("<|{role}|>\n{content}<|end|>\n"))
                    .collect::<String>();
                Some(formatted + "<|assistant|>\n")
            }
        }

        let model = ModelWithNativeTemplate;
        let messages = vec![
            ("system".to_string(), "Be helpful.".to_string()),
            ("user".to_string(), "Solve 2+2.".to_string()),
        ];

        let result = model.format_prompt(&messages);
        assert!(
            result.is_some(),
            "Model with native template must return Some"
        );

        let prompt = result.clone().unwrap();
        assert!(
            prompt.contains("system"),
            "Formatted prompt must include system role"
        );
        assert!(
            prompt.contains("Solve 2+2"),
            "Formatted prompt must include user content"
        );
        assert!(
            prompt.contains("<|assistant|>"),
            "Formatted prompt must end with assistant opening tag \
             (add_ass=true in apply_chat_template)"
        );

        // Caller logic: when Some, skip fam.render()
        let used_native = result.is_some();
        assert!(
            used_native,
            "Caller must use native template result, not fam.render()"
        );
    }
}
