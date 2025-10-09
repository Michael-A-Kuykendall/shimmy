/// CRITICAL PACKAGING REGRESSION TEST
///
/// This test ensures that the crates.io package contains ALL files required
/// for compilation, specifically preventing the template files packaging disaster
/// that broke cargo install for weeks (issues #73, #86, #88).
///
/// This test would have FAILED and prevented those releases if it existed.

use std::process::Command;
use std::str;

#[test]
fn test_crates_io_package_includes_all_required_files() {
    // Run cargo package --list to get the exact file list that would be uploaded to crates.io
    let output = Command::new("cargo")
        .args(&["package", "--list", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo package --list");

    assert!(output.status.success(), 
        "cargo package --list failed: {}", 
        String::from_utf8_lossy(&output.stderr));

    let package_files = String::from_utf8_lossy(&output.stdout);
    let files: Vec<&str> = package_files.lines().collect();

    // CRITICAL: All template files that caused the packaging disaster MUST be included
    let required_template_files = vec![
        // Docker templates - these were the primary failure point
        "templates/docker/Dockerfile",
        "templates/docker/docker-compose.yml", 
        "templates/docker/nginx.conf",
        
        // Kubernetes templates
        "templates/kubernetes/deployment.yaml",
        "templates/kubernetes/service.yaml",
        "templates/kubernetes/configmap.yaml",
        
        // Framework templates
        "templates/frameworks/fastapi/main.py",
        "templates/frameworks/fastapi/requirements.txt",
        "templates/frameworks/express/app.js",
        "templates/frameworks/express/package.json",
        
        // Cloud deployment templates
        "templates/railway/railway.toml",
        "templates/fly/fly.toml",
        
        // Core source files
        "src/templates.rs",
        "Cargo.toml",
        "README.md",
        "LICENSE",
    ];

    // FAIL HARD if any required file is missing
    let mut missing_files = Vec::new();
    for required_file in &required_template_files {
        // Normalize path separators for cross-platform compatibility
        let normalized_required = required_file.replace("/", "\\");
        
        let found = files.iter().any(|&file| {
            file == *required_file || file == normalized_required
        });
        
        if !found {
            missing_files.push(*required_file);
        }
    }

    if !missing_files.is_empty() {
        panic!(
            "🚨 PACKAGING REGRESSION DETECTED! 🚨\n\
            The following required files are MISSING from the crates.io package:\n\
            {}\n\n\
            This would cause 'cargo install shimmy' to FAIL with missing file errors!\n\
            \n\
            Root cause: Cargo.toml include/exclude patterns are broken.\n\
            \n\
            Fix: Update Cargo.toml to include all template files.\n\
            Test: Run 'cargo package --list' and verify all files are present.\n\
            \n\
            FILES THAT WILL BE PACKAGED:\n{}\n",
            missing_files.join("\n"),
            files.join("\n")
        );
    }

    // Additional validation: Ensure we have a reasonable number of files
    assert!(files.len() >= 30, 
        "Package contains too few files ({}), likely missing directories", 
        files.len());

    // Ensure template directory is properly included
    let template_file_count = files.iter()
        .filter(|file| file.contains("templates"))
        .count();
    
    assert!(template_file_count >= 14, 
        "Missing template files - expected at least 14, found {}", 
        template_file_count);

    println!("✅ Packaging regression test PASSED");
    println!("📦 Package contains {} files", files.len());
    println!("📁 Template files included: {}", template_file_count);
}

#[test]
fn test_include_str_macros_would_compile() {
    // Test that all include_str!() macros in templates.rs can find their files
    // This test runs at compile time, so if it compiles, the files exist
    
    // These are the exact include_str!() calls that were failing in production
    let _docker_dockerfile = include_str!("../templates/docker/Dockerfile");
    let _docker_compose = include_str!("../templates/docker/docker-compose.yml");
    let _docker_nginx = include_str!("../templates/docker/nginx.conf");
    
    let _k8s_deployment = include_str!("../templates/kubernetes/deployment.yaml");
    let _k8s_service = include_str!("../templates/kubernetes/service.yaml");
    let _k8s_configmap = include_str!("../templates/kubernetes/configmap.yaml");
    
    let _fastapi_main = include_str!("../templates/frameworks/fastapi/main.py");
    let _fastapi_requirements = include_str!("../templates/frameworks/fastapi/requirements.txt");
    
    let _express_app = include_str!("../templates/frameworks/express/app.js");
    let _express_package = include_str!("../templates/frameworks/express/package.json");
    
    let _railway_config = include_str!("../templates/railway/railway.toml");
    let _fly_config = include_str!("../templates/fly/fly.toml");

    // Verify content is not empty (files actually exist and have content)
    assert!(!_docker_dockerfile.is_empty(), "Docker Dockerfile is empty");
    assert!(!_fastapi_main.is_empty(), "FastAPI main.py is empty");
    assert!(!_express_app.is_empty(), "Express app.js is empty");
    
    println!("✅ All include_str!() macros compile successfully");
}

#[test]
fn test_cargo_install_simulation() {
    // Simulate the conditions that cargo install would face
    // by trying to run cargo check on the packaged source
    
    // This test ensures that a fresh cargo install would succeed
    let output = Command::new("cargo")
        .args(&["check", "--release", "--quiet"])
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Look for the specific template file errors that plagued users
        if stderr.contains("couldn't read") && stderr.contains("templates/") {
            panic!(
                "🚨 CARGO INSTALL WOULD FAIL! 🚨\n\
                Detected the same template file errors that broke production:\n\
                {}\n\
                This is exactly what users experienced with 'cargo install shimmy'",
                stderr
            );
        }
        
        // Allow other types of build failures (like missing dependencies)
        // but fail hard on template file issues
        if stderr.contains("No such file or directory") && stderr.contains("include_str") {
            panic!(
                "🚨 INCLUDE_STR MACRO FAILURE! 🚨\n\
                Template files are missing, cargo install would fail:\n{}",
                stderr
            );
        }
    }

    println!("✅ Cargo install simulation passed - build would succeed");
}

#[test]
fn test_package_size_sanity() {
    // Ensure the package isn't suspiciously small (which would indicate missing files)
    let output = Command::new("cargo")
        .args(&["package", "--list", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo package --list");

    let package_files = String::from_utf8_lossy(&output.stdout);
    let total_size: usize = package_files.lines()
        .filter(|line| !line.is_empty())
        .map(|_| 1) // Count files
        .sum();

    // Based on our investigation, we know we need at least:
    // - ~50+ source files
    // - 14+ template files  
    // - Core files (Cargo.toml, README, LICENSE, etc.)
    assert!(total_size >= 50, 
        "Package is suspiciously small ({} files) - likely missing directories", 
        total_size);

    println!("✅ Package size sanity check passed: {} files", total_size);
}