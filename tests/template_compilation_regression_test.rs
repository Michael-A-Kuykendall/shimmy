/// Regression test for template file compilation
///
/// This test ensures that all template files required by include_str!() macros
/// are properly included in the crates.io package and can be compiled.
///
/// Issue: #73, #86, #88 - Missing template files in v1.6.0 and v1.7.0 from crates.io
/// Users were getting compilation errors like:
/// `error: couldn't read '../templates/docker/Dockerfile': No such file or directory`
use shimmy::templates::generate_template;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_all_template_types_compile() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    // Test all supported template types to ensure include_str!() macros work
    let template_types = vec![
        "docker",
        "kubernetes",
        "railway",
        "fly",
        "fastapi",
        "express",
    ];

    for template_type in template_types {
        let result = generate_template(template_type, temp_path, Some("test-project"));

        // Should not panic or fail due to missing template files
        assert!(
            result.is_ok(),
            "Template generation failed for {}: {:?}",
            template_type,
            result.err()
        );

        // Verify that files were actually created
        let template_dir = std::path::Path::new(temp_path);
        assert!(template_dir.exists(), "Template directory should exist");

        // Check that at least one file was created
        let entries: Vec<_> = fs::read_dir(template_dir)
            .expect("Failed to read template directory")
            .collect();
        assert!(
            !entries.is_empty(),
            "Template generation for {} should create at least one file",
            template_type
        );
    }
}

#[test]
fn test_docker_template_contains_required_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let result = generate_template("docker", temp_path, Some("test-project"));
    assert!(result.is_ok(), "Docker template generation should succeed");

    // Verify specific files that were causing compilation failures
    let expected_files = vec![
        "Dockerfile",
        "docker-compose.yml",
        "nginx.conf",
        ".dockerignore",
    ];

    for file in expected_files {
        let file_path = std::path::Path::new(temp_path).join(file);
        assert!(
            file_path.exists(),
            "Docker template should create {}: {}",
            file,
            file_path.display()
        );

        // Verify file is not empty (template content was included)
        let content =
            fs::read_to_string(&file_path).unwrap_or_else(|_| panic!("Failed to read {}", file));
        assert!(
            !content.trim().is_empty(),
            "{} should contain template content, not be empty",
            file
        );
    }
}

#[test]
fn test_kubernetes_template_contains_required_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let result = generate_template("kubernetes", temp_path, Some("test-project"));
    assert!(
        result.is_ok(),
        "Kubernetes template generation should succeed"
    );

    // Verify specific files that were causing compilation failures
    let expected_files = vec!["deployment.yaml", "service.yaml", "configmap.yaml"];

    for file in expected_files {
        let file_path = std::path::Path::new(temp_path).join(file);
        assert!(
            file_path.exists(),
            "Kubernetes template should create {}: {}",
            file,
            file_path.display()
        );

        // Verify file contains YAML content
        let content =
            fs::read_to_string(&file_path).unwrap_or_else(|_| panic!("Failed to read {}", file));
        assert!(
            content.contains("apiVersion") || content.contains("kind"),
            "{} should contain valid Kubernetes YAML content",
            file
        );
    }
}

#[test]
fn test_fastapi_template_contains_required_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let result = generate_template("fastapi", temp_path, Some("test-project"));
    assert!(result.is_ok(), "FastAPI template generation should succeed");

    // Verify specific files that were causing compilation failures
    let expected_files = vec!["main.py", "requirements.txt"];

    for file in expected_files {
        let file_path = std::path::Path::new(temp_path).join(file);
        assert!(
            file_path.exists(),
            "FastAPI template should create {}: {}",
            file,
            file_path.display()
        );

        // Verify file contains expected content
        let content =
            fs::read_to_string(&file_path).unwrap_or_else(|_| panic!("Failed to read {}", file));

        if file == "main.py" {
            assert!(
                content.contains("fastapi") || content.contains("FastAPI"),
                "main.py should contain FastAPI imports/usage"
            );
        } else if file == "requirements.txt" {
            assert!(
                content.contains("fastapi"),
                "requirements.txt should list fastapi dependency"
            );
        }
    }
}

#[test]
fn test_express_template_contains_required_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let result = generate_template("express", temp_path, Some("test-project"));
    assert!(result.is_ok(), "Express template generation should succeed");

    // Verify specific files that were causing compilation failures
    let expected_files = vec!["app.js", "package.json"];

    for file in expected_files {
        let file_path = std::path::Path::new(temp_path).join(file);
        assert!(
            file_path.exists(),
            "Express template should create {}: {}",
            file,
            file_path.display()
        );

        // Verify file contains expected content
        let content =
            fs::read_to_string(&file_path).unwrap_or_else(|_| panic!("Failed to read {}", file));

        if file == "app.js" {
            assert!(
                content.contains("express"),
                "app.js should contain express usage"
            );
        } else if file == "package.json" {
            assert!(
                content.contains("express"),
                "package.json should list express dependency"
            );
        }
    }
}

#[test]
fn test_template_files_are_not_empty() {
    // This test verifies that all include_str!() calls actually include content
    // and that the template files exist in the package

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    // Generate a template and verify content is substantial
    let result = generate_template("docker", temp_path, None);
    assert!(result.is_ok(), "Template generation should succeed");

    let dockerfile_path = std::path::Path::new(temp_path).join("Dockerfile");
    let dockerfile_content =
        fs::read_to_string(&dockerfile_path).expect("Failed to read Dockerfile");

    // Dockerfile should be substantial (more than just a few lines)
    assert!(
        dockerfile_content.len() > 100,
        "Dockerfile content should be substantial, got {} characters",
        dockerfile_content.len()
    );

    // Should contain typical Dockerfile commands
    assert!(
        dockerfile_content.contains("FROM") || dockerfile_content.contains("RUN"),
        "Dockerfile should contain typical Docker commands"
    );
}
