/// Integration tests to validate the release gate system itself works correctly
/// This ensures our release gates properly catch real issues and block releases
use std::process::Command;
use std::time::Duration;

#[test]
fn test_release_gate_system_exists() {
    // Validate that release.yml contains the mandatory gates
    let workflow_content = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Failed to read release.yml");
    
    assert!(workflow_content.contains("🚧 Release Gates - MANDATORY VALIDATION"), 
        "Release workflow missing mandatory gate job");
    assert!(workflow_content.contains("GATE 1/6: Core Build Validation"), 
        "Missing Gate 1 (Core Build)");
    assert!(workflow_content.contains("GATE 2/6: CUDA Build Timeout Detection"), 
        "Missing Gate 2 (CUDA Timeout)");
    assert!(workflow_content.contains("GATE 3/6: Template Packaging Validation"), 
        "Missing Gate 3 (Template Packaging)");
    assert!(workflow_content.contains("GATE 4/6: Binary Size Constitutional Limit"), 
        "Missing Gate 4 (Binary Size)");
    assert!(workflow_content.contains("GATE 5/6: Test Suite Validation"), 
        "Missing Gate 5 (Test Suite)");
    assert!(workflow_content.contains("GATE 6/6: Documentation Validation"), 
        "Missing Gate 6 (Documentation)");
}

#[test]
fn test_conditional_execution_logic() {
    // Validate that downstream jobs require preflight gate passage
    let workflow_content = std::fs::read_to_string(".github/workflows/release.yml")
        .expect("Failed to read release.yml");
    
    assert!(workflow_content.contains("needs: preflight"), 
        "Build job doesn't depend on preflight gates");
    assert!(workflow_content.contains("needs.preflight.outputs.should_publish == 'true'"), 
        "Missing conditional execution logic");
    assert!(workflow_content.contains("needs: [preflight, build]"), 
        "Release job doesn't depend on both preflight and build");
}

#[test]
fn test_gate_1_core_build_validation() {
    // Test that core build (huggingface features) works
    let output = Command::new("cargo")
        .args(&["build", "--release", "--no-default-features", "--features", "huggingface"])
        .output()
        .expect("Failed to run cargo build");
    
    assert!(output.status.success(), 
        "Gate 1 (Core Build) should pass: {}",
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_gate_3_template_packaging_protection() {
    // Test that templates are properly included (Issue #60 protection)
    let output = Command::new("cargo")
        .args(&["package", "--list", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo package --list");
    
    let package_list = String::from_utf8_lossy(&output.stdout);
    
    // Check for any of the valid Docker template paths (Issue #60 protection)
    let has_dockerfile = package_list.lines().any(|line| {
        line == "Dockerfile" || 
        line == "packaging/docker/Dockerfile" || 
        line == "templates/docker/Dockerfile"
    });
    
    assert!(
        has_dockerfile,
        "Required Docker template missing from package: {} (Issue #60 regression!)", 
        package_list
    );
}

#[test]
fn test_gate_4_binary_size_constitutional_limit() {
    // First ensure we have a binary to test
    let build_output = Command::new("cargo")
        .args(&["build", "--release", "--no-default-features", "--features", "huggingface"])
        .output()
        .expect("Failed to build binary for size test");
    
    assert!(build_output.status.success(), "Failed to build binary for size test");
    
    // Test constitutional 20MB limit
    let binary_path = if cfg!(windows) {
        "target/release/shimmy.exe"
    } else {
        "target/release/shimmy"
    };
    
    if let Ok(metadata) = std::fs::metadata(binary_path) {
        let size = metadata.len();
        let max_size = 20 * 1024 * 1024; // 20MB constitutional limit
        
        assert!(size <= max_size, 
            "Binary size {} bytes exceeds constitutional limit of {} bytes (Gate 4 failure)", 
            size, max_size);
    } else {
        panic!("Binary not found at {}", binary_path);
    }
}

#[test]
fn test_gate_5_test_suite_validation() {
    // Validate that test suite runs successfully (this test is part of it!)
    let output = Command::new("cargo")
        .args(&["test", "--lib", "--bins"])
        .output()
        .expect("Failed to run test suite");
    
    assert!(output.status.success(), 
        "Gate 5 (Test Suite) should pass: {}",
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_gate_6_documentation_validation() {
    // Test that documentation builds successfully
    let output = Command::new("cargo")
        .args(&["doc", "--no-deps", "--no-default-features", "--features", "huggingface"])
        .output()
        .expect("Failed to run cargo doc");
    
    assert!(output.status.success(), 
        "Gate 6 (Documentation) should pass: {}",
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_local_validation_scripts_exist() {
    // Ensure local validation scripts exist and are executable
    assert!(std::path::Path::new("scripts/validate-release.ps1").exists(),
        "PowerShell validation script missing");
    
    // Note: Not testing bash script existence on Windows, but it should exist for Unix systems
}

#[test] 
#[ignore] // Only run this test manually as it involves timeouts
fn test_gate_2_cuda_timeout_detection_manual() {
    // Manual test for CUDA timeout detection (Issue #59 protection)
    // This test is ignored by default because it involves long timeouts
    
    use std::time::Instant;
    let start = Instant::now();
    
    let output = Command::new("timeout")
        .args(&["180", "cargo", "build", "--release", "--no-default-features", "--features", "llama"])
        .output();
    
    let duration = start.elapsed();
    
    match output {
        Ok(output) => {
            if !output.status.success() && duration >= Duration::from_secs(180) {
                println!("✅ Gate 2 correctly detected CUDA timeout (Issue #59 protection)");
            } else if output.status.success() && duration < Duration::from_secs(180) {
                println!("✅ Gate 2 passed - CUDA build completed within 3 minutes");
            } else {
                panic!("Gate 2 unexpected behavior: success={}, duration={:?}", 
                    output.status.success(), duration);
            }
        }
        Err(e) => {
            // timeout command might not be available on all systems
            println!("⚠️ Could not test CUDA timeout (timeout command unavailable): {}", e);
        }
    }
}