use std::fs;
use tempfile::TempDir;
use shimmy::engine::llama::LlamaEngine;
use shimmy::engine::{GenOptions, ModelSpec};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_adaptive_batch_size() {
        // Test with small context (should use base size)
        let small_ctx = LlamaEngine::calculate_adaptive_batch_size(1024);
        assert_eq!(small_ctx, 2048, "Small contexts should use base batch size");

        // Test with medium context (should use base size)
        let medium_ctx = LlamaEngine::calculate_adaptive_batch_size(4096);
        assert_eq!(medium_ctx, 4096, "Medium contexts should scale up");

        // Test with large context (should be capped)
        let large_ctx = LlamaEngine::calculate_adaptive_batch_size(16384);
        assert_eq!(large_ctx, 8192, "Large contexts should be capped at 8192");

        // Test edge case at cap
        let at_cap = LlamaEngine::calculate_adaptive_batch_size(8192);
        assert_eq!(at_cap, 8192, "Context at cap should use cap value");
    }

    #[test]
    fn test_large_prompt_batch_size_calculation() {
        // This test ensures that contexts large enough to handle the reported issue
        // (DeepSeek-R1-Distill-Qwen-7B with large system prompts) work correctly

        // DeepSeek models typically use 4096 or 8192 context
        let deepseek_ctx = LlamaEngine::calculate_adaptive_batch_size(4096);
        assert!(deepseek_ctx >= 4096, "DeepSeek context should be supported");

        // With large system prompts, we might need more batch capacity
        // The original issue had n_batch = 2048, which was insufficient
        assert!(deepseek_ctx > 2048, "Batch size should exceed the problematic 2048 limit");
    }

    #[test]
    fn test_batch_size_reasonable_limits() {
        // Ensure we don't create excessively large batch sizes that would waste memory

        // Very large contexts should still be capped
        let huge_ctx = LlamaEngine::calculate_adaptive_batch_size(32768);
        assert_eq!(huge_ctx, 8192, "Huge contexts should be capped to prevent memory waste");

        // Edge case: context exactly at cap
        let exact_cap = LlamaEngine::calculate_adaptive_batch_size(8192);
        assert_eq!(exact_cap, 8192, "Exact cap should be allowed");
    }
}