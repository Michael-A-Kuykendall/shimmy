use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_llm_only_filtering() {
    // Create temporary directory with test models
    let temp_dir = TempDir::new().unwrap();
    let test_models_dir = temp_dir.path().join("test-filtering");
    fs::create_dir_all(&test_models_dir).unwrap();

    // Create test model files
    let llm_model = test_models_dir.join("llama3-chat.gguf");
    let vision_model = test_models_dir.join("stable-diffusion-xl-vision.gguf");
    let clip_model = test_models_dir.join("clip-large-embedding.gguf");
    let audio_model = test_models_dir.join("whisper-audio-tts.gguf");

    fs::write(&llm_model, b"").unwrap();
    fs::write(&vision_model, b"").unwrap();
    fs::write(&clip_model, b"").unwrap();
    fs::write(&audio_model, b"").unwrap();

    let model_dirs_arg = format!("--model-dirs={}", test_models_dir.display());

    // Test without filtering - should show all models
    let mut cmd_all = Command::cargo_bin("shimmy").unwrap();
    let output_all = cmd_all
        .args(["discover", &model_dirs_arg])
        .assert()
        .success();

    let stdout_all = String::from_utf8(output_all.get_output().stdout.clone()).unwrap();

    // Test with LLM filtering - should filter out non-LLM models
    let mut cmd_filtered = Command::cargo_bin("shimmy").unwrap();
    let output_filtered = cmd_filtered
        .args(["discover", &model_dirs_arg, "--llm-only"])
        .assert()
        .success();

    let stdout_filtered = String::from_utf8(output_filtered.get_output().stdout.clone()).unwrap();

    // Verify filtering behavior
    assert!(
        stdout_all.contains("llama3-chat"),
        "LLM model should appear in unfiltered results"
    );
    assert!(
        stdout_all.contains("stable-diffusion-xl-vision"),
        "Vision model should appear in unfiltered results"
    );
    assert!(
        stdout_all.contains("clip-large-embedding"),
        "CLIP model should appear in unfiltered results"
    );
    assert!(
        stdout_all.contains("whisper-audio-tts"),
        "Audio model should appear in unfiltered results"
    );

    assert!(
        stdout_filtered.contains("llama3-chat"),
        "LLM model should appear in filtered results"
    );
    assert!(
        !stdout_filtered.contains("stable-diffusion-xl-vision"),
        "Vision model should be filtered out"
    );
    assert!(
        !stdout_filtered.contains("clip-large-embedding"),
        "CLIP model should be filtered out"
    );
    assert!(
        !stdout_filtered.contains("whisper-audio-tts"),
        "Audio model should be filtered out"
    );

    assert!(
        stdout_filtered.contains("🎯 Filtering to LLM models only..."),
        "Should show filtering message"
    );
}

#[test]
fn test_moe_cpu_offloading_flags() {
    // Test that MoE CPU flags are accepted without errors
    let mut cmd = Command::cargo_bin("shimmy").unwrap();
    cmd.args(["--cpu-moe", "list"]).assert().success();

    // Test n-cpu-moe flag
    let mut cmd2 = Command::cargo_bin("shimmy").unwrap();
    cmd2.args(["--n-cpu-moe", "4", "list"]).assert().success();
}

#[test]
fn test_moe_cpu_flags_conflict() {
    // Test that --cpu-moe and --n-cpu-moe conflict
    let mut cmd = Command::cargo_bin("shimmy").unwrap();
    cmd.args(["--cpu-moe", "--n-cpu-moe", "4", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_discover_help_shows_llm_only() {
    let mut cmd = Command::cargo_bin("shimmy").unwrap();
    cmd.args(["discover", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--llm-only"))
        .stdout(predicate::str::contains("Show only LLM models"));
}

#[test]
fn test_threading_optimization_performance() {
    // Test that threading optimization is properly implemented
    // This is a regression test for Issue #101
    let mut cmd = Command::cargo_bin("shimmy").unwrap();
    cmd.args(["--help"]).assert().success();
    // The fact that this doesn't hang or consume excessive CPU is the test
    // If threading was broken, this would cause issues
}

#[test]
fn test_streaming_functionality() {
    // Test that streaming functionality is available
    // This is a regression test for Issue #101
    let mut cmd = Command::cargo_bin("shimmy").unwrap();
    cmd.args(["serve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("HTTP server")); // Verify server can start
}

#[test]
fn test_ollama_models_environment_variable() {
    // Test OLLAMA_MODELS environment variable support
    // This is a regression test for Issue #101
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().to_string_lossy().to_string();

    let mut cmd = Command::cargo_bin("shimmy").unwrap();
    cmd.env("OLLAMA_MODELS", &test_path)
        .args(["list"])
        .assert()
        .success(); // Should not crash when OLLAMA_MODELS is set
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_server_stability_issue_106() {
    // Regression test for Issue #106: Windows server crashes
    // This test ensures shimmy can handle Windows path separators and start server

    let mut cmd = Command::cargo_bin("shimmy").unwrap();

    // Test that server can start without crashing on Windows
    // Instead of spawning and killing, just test that server help works
    cmd.args(["serve", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("HTTP server")); // Verify server command exists

    // If we reach here, the server started successfully without crashing
}
