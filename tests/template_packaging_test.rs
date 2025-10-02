use std::process::Command;

#[test]
fn test_template_packaging_regression() {
    // Regression test for Issues #78, #77, #74
    // Ensure all template files are included in the package
    
    let output = Command::new("cargo")
        .args(&["package", "--list", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo package --list");
    
    let package_list = String::from_utf8_lossy(&output.stdout);
    
    // Check required template files are included
    let required_templates = [
        "templates/docker/Dockerfile",
        "templates/docker/docker-compose.yml", 
        "templates/docker/nginx.conf",
        "templates/kubernetes/deployment.yaml",
        "templates/kubernetes/service.yaml",
        "templates/kubernetes/configmap.yaml",
        "templates/railway/railway.toml",
        "templates/fly/fly.toml",
        "templates/frameworks/fastapi/main.py",
        "templates/frameworks/fastapi/requirements.txt",
        "templates/frameworks/express/app.js",
        "templates/frameworks/express/package.json",
    ];
    
    for template in &required_templates {
        assert!(
            package_list.contains(template),
            "Required template missing from package: {} (Issue #78 regression!)",
            template
        );
    }
}

#[test]
fn test_template_files_exist() {
    // Ensure template files actually exist in the repo
    let required_templates = [
        "templates/docker/Dockerfile",
        "templates/docker/docker-compose.yml", 
        "templates/docker/nginx.conf",
        "templates/kubernetes/deployment.yaml",
        "templates/kubernetes/service.yaml",
        "templates/kubernetes/configmap.yaml",
        "templates/railway/railway.toml",
        "templates/fly/fly.toml", 
        "templates/frameworks/fastapi/main.py",
        "templates/frameworks/fastapi/requirements.txt",
        "templates/frameworks/express/app.js",
        "templates/frameworks/express/package.json",
    ];
    
    for template in &required_templates {
        let path = std::path::Path::new(template);
        assert!(
            path.exists(),
            "Template file does not exist: {} (Issue #78 regression!)",
            template
        );
    }
}