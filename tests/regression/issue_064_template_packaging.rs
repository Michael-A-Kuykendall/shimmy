/// Regression test for Issue #64: Template files must be included in crates.io package
///
/// This test ensures that all template files referenced by include_str! macros
/// are properly included in the crates.io package and accessible during compilation.
///
/// Without this test, template files could be excluded from the package, causing
/// compilation failures for users installing via `cargo install shimmy`.

#[test]
fn test_template_files_are_included_in_package() {
    // Test that all template files used in src/templates.rs are accessible

    // Docker templates
    let _dockerfile = include_str!("../../templates/docker/Dockerfile");
    let _compose = include_str!("../../templates/docker/docker-compose.yml");
    let _nginx = include_str!("../../templates/docker/nginx.conf");

    // Kubernetes templates
    let _deployment = include_str!("../../templates/kubernetes/deployment.yaml");
    let _service = include_str!("../../templates/kubernetes/service.yaml");
    let _configmap = include_str!("../../templates/kubernetes/configmap.yaml");

    // Cloud platform templates
    let _railway = include_str!("../../templates/railway/railway.toml");
    let _fly = include_str!("../../templates/fly/fly.toml");

    // Framework templates
    let _fastapi_main = include_str!("../../templates/frameworks/fastapi/main.py");
    let _fastapi_requirements = include_str!("../../templates/frameworks/fastapi/requirements.txt");
    let _express_app = include_str!("../../templates/frameworks/express/app.js");
    let _express_package = include_str!("../../templates/frameworks/express/package.json");

    // If any template file is missing, this test will fail at compile time
    // preventing the issue from reaching users

    assert!(
        !_dockerfile.is_empty(),
        "Dockerfile template should not be empty"
    );
    assert!(
        !_compose.is_empty(),
        "docker-compose.yml template should not be empty"
    );
    assert!(
        !_nginx.is_empty(),
        "nginx.conf template should not be empty"
    );
    assert!(
        !_deployment.is_empty(),
        "deployment.yaml template should not be empty"
    );
    assert!(
        !_service.is_empty(),
        "service.yaml template should not be empty"
    );
    assert!(
        !_configmap.is_empty(),
        "configmap.yaml template should not be empty"
    );
    assert!(
        !_railway.is_empty(),
        "railway.toml template should not be empty"
    );
    assert!(!_fly.is_empty(), "fly.toml template should not be empty");
    assert!(
        !_fastapi_main.is_empty(),
        "FastAPI main.py template should not be empty"
    );
    assert!(
        !_fastapi_requirements.is_empty(),
        "FastAPI requirements.txt template should not be empty"
    );
    assert!(
        !_express_app.is_empty(),
        "Express app.js template should not be empty"
    );
    assert!(
        !_express_package.is_empty(),
        "Express package.json template should not be empty"
    );
}

#[test]
fn test_template_content_validity() {
    // Basic validation that templates contain expected content

    let dockerfile = include_str!("../../templates/docker/Dockerfile");
    assert!(
        dockerfile.contains("FROM"),
        "Dockerfile should contain FROM instruction"
    );

    let compose = include_str!("../../templates/docker/docker-compose.yml");
    assert!(
        compose.contains("version:") || compose.contains("services:"),
        "docker-compose.yml should contain version or services"
    );

    let deployment = include_str!("../../templates/kubernetes/deployment.yaml");
    assert!(
        deployment.contains("apiVersion:"),
        "deployment.yaml should contain apiVersion"
    );

    let fastapi_main = include_str!("../../templates/frameworks/fastapi/main.py");
    assert!(
        fastapi_main.contains("FastAPI"),
        "FastAPI template should reference FastAPI"
    );

    let express_app = include_str!("../../templates/frameworks/express/app.js");
    assert!(
        express_app.contains("express"),
        "Express template should reference express"
    );
}
