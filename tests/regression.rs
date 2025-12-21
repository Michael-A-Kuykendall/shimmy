/// Regression Test Suite - User-Reported Issues
///
/// This module includes all individual regression test files from tests/regression/
/// Each file tests a specific user-reported issue to prevent regressions.
///
// Auto-discovered by CI/CD - just add new issue_NNN_*.rs files to tests/regression/
// Include all individual regression test modules (only files that exist)
#[path = "regression/issue_012_custom_model_dirs.rs"]
mod issue_012_custom_model_dirs;

#[path = "regression/issue_013_qwen_template.rs"]
mod issue_013_qwen_template;

#[path = "regression/issue_051_lmstudio_discovery.rs"]
mod issue_051_lmstudio_discovery;

#[path = "regression/issue_053_sse_duplicate_prefix.rs"]
mod issue_053_sse_duplicate_prefix;

#[path = "regression/issue_063_version_mismatch.rs"]
mod issue_063_version_mismatch;

#[path = "regression/issue_064_template_packaging.rs"]
mod issue_064_template_packaging;

#[path = "regression/issue_068_mlx_support.rs"]
mod issue_068_mlx_support;

#[path = "regression/issue_072_gpu_backend_flag.rs"]
mod issue_072_gpu_backend_flag;

#[path = "regression/issue_101_performance_fixes.rs"]
mod issue_101_performance_fixes;

#[path = "regression/issue_106_windows_crash.rs"]
mod issue_106_windows_crash;

#[path = "regression/issue_108_memory_allocation.rs"]
mod issue_108_memory_allocation;

#[path = "regression/issue_110_crates_io_build.rs"]
mod issue_110_crates_io_build;

#[path = "regression/issue_111_gpu_metrics.rs"]
mod issue_111_gpu_metrics;

#[path = "regression/issue_112_safetensors_engine.rs"]
mod issue_112_safetensors_engine;

#[path = "regression/issue_113_openai_api.rs"]
mod issue_113_openai_api;

#[path = "regression/issue_114_mlx_distribution.rs"]
mod issue_114_mlx_distribution;

#[path = "regression/issue_128_backend_reinitialization.rs"]
mod issue_128_backend_reinitialization;

#[path = "regression/issue_129_precompiled_gpu_support.rs"]
mod issue_129_precompiled_gpu_support;

#[path = "regression/issue_130_gpu_layer_offloading.rs"]
mod issue_130_gpu_layer_offloading;

#[path = "regression/issue_131_arm64_ci_support.rs"]
mod issue_131_arm64_ci_support;

#[path = "regression/issue_132_auto_stop_tokens.rs"]
mod issue_132_auto_stop_tokens;

#[path = "regression/issue_packaging_general.rs"]
mod issue_packaging_general;

#[path = "regression/issue_version_validation.rs"]
mod issue_version_validation;

// This test file is now executable via: cargo test --test regression
// CI/CD runs this automatically before main test suite
