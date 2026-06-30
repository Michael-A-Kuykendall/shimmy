/// Regression Test Suite - User-Reported Issues
///
/// Each file tests a specific user-reported issue to prevent regressions.
#[path = "regression/issue_012_custom_model_dirs.rs"]
mod issue_012_custom_model_dirs;

#[path = "regression/issue_013_qwen_template.rs"]
mod issue_013_qwen_template;

#[path = "regression/issue_053_sse_duplicate_prefix.rs"]
mod issue_053_sse_duplicate_prefix;

#[path = "regression/issue_063_version_mismatch.rs"]
mod issue_063_version_mismatch;

#[path = "regression/issue_064_template_packaging.rs"]
mod issue_064_template_packaging;

#[path = "regression/issue_068_mlx_support.rs"]
mod issue_068_mlx_support;

#[path = "regression/issue_110_crates_io_build.rs"]
mod issue_110_crates_io_build;

#[path = "regression/issue_111_gpu_metrics.rs"]
mod issue_111_gpu_metrics;

#[path = "regression/issue_112_safetensors_engine.rs"]
mod issue_112_safetensors_engine;

#[path = "regression/issue_113_openai_api.rs"]
mod issue_113_openai_api;

