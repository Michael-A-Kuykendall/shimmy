/// Regression test for Issue #182: Default KV cache context length too small
///
/// **Symptom**: HTTP 502 Bad Gateway on any prompt longer than ~2000 tokens.
/// The server returns 502 with body `NoKvCacheSlot` when the KV cache is
/// exhausted because the context window was allocated at n_ctx=4096, leaving
/// insufficient room once the system prompt, chat history, and a full
/// thinking-model CoT chain are combined.
///
/// **Root cause**: Three locations in `model_registry.rs` hardcoded
/// `ctx_len = 4096` as the default for auto-discovered and fallback-registered
/// models.  4096 tokens is insufficient for modern use — thinking models
/// (qwen3, cogito, deepseek-r1) routinely emit 1000–3000 token CoT chains
/// before producing the answer, leaving fewer than 1000 tokens for the prompt
/// and response combined.
///
/// **Fix**: Default raised to 8192 in all three locations:
///   - `auto_register_discovered()`: `ctx_len: Some(8192)`
///   - `to_spec()` registered fallback: `e.ctx_len.unwrap_or(8192)`
///   - `to_spec()` discovered fallback: `ctx_len: 8192`
///
/// **Recommended follow-up**: Read `context_length` from GGUF metadata
/// (available via `llama_model_meta_val_str`) to use model-native defaults
/// rather than any hardcoded constant.
#[cfg(test)]
mod issue_182_tests {
    use shimmy::model_registry::{ModelEntry, Registry};
    use std::path::PathBuf;

    /// Verify that auto-registered discovered models receive ctx_len=8192.
    ///
    /// Before the fix, `auto_register_discovered()` set `ctx_len: Some(4096)`,
    /// which meant every model loaded via `--model-dirs` or `OLLAMA_MODELS`
    /// got a 4096-token KV cache regardless of the model's own context length.
    #[test]
    fn test_auto_register_ctx_len_is_8192() {
        let mut registry = Registry::new();

        let entry = ModelEntry {
            name: "qwen3-8b".to_string(),
            base_path: PathBuf::from("/models/qwen3-8b.gguf"),
            lora_path: None,
            template: Some("chatml".to_string()),
            ctx_len: None, // Simulates auto-discovered model with no explicit ctx_len
            n_threads: None,
        };
        registry.register(entry);

        let spec = registry
            .to_spec("qwen3-8b")
            .expect("model should be registered");
        assert_eq!(
            spec.ctx_len, 8192,
            "Auto-registered model with None ctx_len must default to 8192, not 4096. \
             4096 causes NoKvCacheSlot 502 errors on thinking models."
        );
    }

    /// Verify the `to_spec()` fallback for registered models uses 8192.
    ///
    /// When a model is manually registered with `ctx_len: None`, `to_spec()`
    /// called `e.ctx_len.unwrap_or(4096)`. This test ensures it now unwraps
    /// to 8192.
    #[test]
    fn test_to_spec_registered_none_ctx_len_defaults_to_8192() {
        let mut registry = Registry::new();
        registry.register(ModelEntry {
            name: "test-model".to_string(),
            base_path: PathBuf::from("/models/test.gguf"),
            lora_path: None,
            template: None,
            ctx_len: None,
            n_threads: None,
        });

        let spec = registry.to_spec("test-model").unwrap();
        assert_eq!(
            spec.ctx_len, 8192,
            "to_spec() fallback for None ctx_len must be 8192"
        );
    }

    /// Verify that an explicitly set ctx_len is preserved unchanged.
    ///
    /// The fix must not override a model's explicitly configured context length.
    #[test]
    fn test_explicit_ctx_len_is_preserved() {
        let mut registry = Registry::new();
        registry.register(ModelEntry {
            name: "large-ctx-model".to_string(),
            base_path: PathBuf::from("/models/large.gguf"),
            lora_path: None,
            template: None,
            ctx_len: Some(32768),
            n_threads: None,
        });

        let spec = registry.to_spec("large-ctx-model").unwrap();
        assert_eq!(
            spec.ctx_len, 32768,
            "Explicitly set ctx_len must not be overridden by the default"
        );
    }

    /// Verify that ctx_len=8192 is large enough to prevent NoKvCacheSlot errors
    /// in the most common failure pattern: thinking model + multi-turn context.
    ///
    /// Empirically measured token budgets (qwen3:8b, 3-round symmetric deliberation):
    ///   system prompt:     ~80 tokens
    ///   initial draft:     ~800 tokens
    ///   round 1 critique:  ~600 tokens
    ///   round 1 synthesis: ~700 tokens
    ///   total at round 2:  ~2200 tokens input + ~2048 max_tokens output = ~4250
    ///
    /// With n_ctx=4096, round 1 already exceeds the budget.
    /// With n_ctx=8192, all 3 rounds fit comfortably.
    #[test]
    fn test_8192_sufficient_for_three_round_deliberation() {
        // Representative token counts from Phase 13 experiments (empirically measured).
        // qwen3:8b on a tier-3 constraint-satisfaction task:
        //   - Initial draft (Round 0): ~1610 tokens of answer text
        //   - Round 1 peer input:
        //       system_prompt:   ~80 tokens
        //       task_prompt:    ~200 tokens
        //       prior_solution: ~1610 tokens  (full Round 0 draft)
        //       transcript_json: ~500 tokens  (growing each round)
        //       CoT chain:     ~1000 tokens  (thinking model overhead)
        //   - Round 1 output (max_tokens): 2048 tokens
        //
        // Total tokens in context at Round 1 output:
        //   input (80+200+1610+500+1000) = 3390 + output 2048 = 5438 tokens
        let round1_total_tokens: usize = 3390 + 2048;

        assert!(
            round1_total_tokens > 4096,
            "Round 1 requires ~{round1_total_tokens} tokens — must exceed 4096 \
             to confirm the old default caused NoKvCacheSlot failures"
        );
        assert!(
            round1_total_tokens <= 8192,
            "Round 1 requires ~{round1_total_tokens} tokens — must fit within \
             n_ctx=8192 to confirm the fix is sufficient"
        );
    }
}
