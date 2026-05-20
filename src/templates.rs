use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateFamily {
    ChatML,
    Llama3,
    OpenChat,
}

impl TemplateFamily {
    pub fn render(
        &self,
        system: Option<&str>,
        messages: &[(String, String)],
        input: Option<&str>,
    ) -> String {
        match self {
            TemplateFamily::ChatML => {
                let mut s = String::new();
                if let Some(sys) = system {
                    s.push_str(&format!("<|im_start|>system\n{}<|im_end|>\n", sys));
                }
                for (role, content) in messages {
                    s.push_str(&format!("<|im_start|>{}\n{}<|im_end|>\n", role, content));
                }
                if let Some(inp) = input {
                    s.push_str(&format!(
                        "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                        inp
                    ));
                }
                s
            }
            TemplateFamily::Llama3 => {
                let mut s = String::new();
                if let Some(sys) = system {
                    s.push_str(&format!(
                        "<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n{}<|eot_id|>",
                        sys
                    ));
                }
                for (role, content) in messages {
                    s.push_str(&format!(
                        "<|start_header_id|>{}<|end_header_id|>\n{}<|eot_id|>",
                        role, content
                    ));
                }
                if let Some(inp) = input {
                    s.push_str(&format!("<|start_header_id|>user<|end_header_id|>\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n", inp));
                }
                s
            }
            TemplateFamily::OpenChat => {
                let mut s = String::new();
                for (role, content) in messages {
                    s.push_str(&format!("{}: {}\n", role, content));
                }
                if let Some(inp) = input {
                    s.push_str(&format!("user: {}\nassistant: ", inp));
                } else {
                    s.push_str("assistant: ");
                }
                s
            }
        }
    }

    /// Get the stop tokens for this template family
    pub fn stop_tokens(&self) -> Vec<String> {
        match self {
            TemplateFamily::ChatML => vec!["<|im_end|>".to_string(), "<|im_start|>".to_string()],
            TemplateFamily::Llama3 => vec!["<|eot_id|>".to_string(), "<|end_of_text|>".to_string()],
            TemplateFamily::OpenChat => vec![],
        }
    }
}

// Template generation functions for deployment platforms

/// Generate Docker deployment template
pub fn generate_docker_template(output_dir: &str, project_name: Option<&str>) -> Result<()> {
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)?;

    // Copy Dockerfile
    let dockerfile_content = include_str!("../templates/docker/Dockerfile");
    fs::write(output_path.join("Dockerfile"), dockerfile_content)?;

    // Copy docker-compose.yml
    let compose_content = include_str!("../templates/docker/docker-compose.yml");
    let customized_compose = if let Some(name) = project_name {
        compose_content.replace("shimmy-ai", &format!("{}-shimmy", name))
    } else {
        compose_content.to_string()
    };
    fs::write(output_path.join("docker-compose.yml"), customized_compose)?;

    // Copy nginx.conf
    let nginx_content = include_str!("../templates/docker/nginx.conf");
    fs::write(output_path.join("nginx.conf"), nginx_content)?;

    // Create .dockerignore
    let dockerignore_content = r#"target/
Cargo.lock
*.md
docs/
tests/
.git/
.gitignore
README.md
"#;
    fs::write(output_path.join(".dockerignore"), dockerignore_content)?;

    Ok(())
}

/// Generate Kubernetes deployment template
pub fn generate_kubernetes_template(output_dir: &str, project_name: Option<&str>) -> Result<()> {
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)?;

    let name = project_name.unwrap_or("shimmy");

    // Generate deployment.yaml
    let deployment_content = include_str!("../templates/kubernetes/deployment.yaml")
        .replace("shimmy-deployment", &format!("{}-deployment", name))
        .replace("app: shimmy", &format!("app: {}", name));
    fs::write(output_path.join("deployment.yaml"), deployment_content)?;

    // Generate service.yaml
    let service_content = include_str!("../templates/kubernetes/service.yaml")
        .replace("shimmy-service", &format!("{}-service", name))
        .replace("shimmy-loadbalancer", &format!("{}-loadbalancer", name))
        .replace("app: shimmy", &format!("app: {}", name));
    fs::write(output_path.join("service.yaml"), service_content)?;

    // Generate configmap.yaml
    let configmap_content = include_str!("../templates/kubernetes/configmap.yaml")
        .replace("shimmy-config", &format!("{}-config", name))
        .replace("shimmy-models-pvc", &format!("{}-models-pvc", name))
        .replace("app: shimmy", &format!("app: {}", name));
    fs::write(output_path.join("configmap.yaml"), configmap_content)?;

    Ok(())
}

/// Generate Railway deployment template
pub fn generate_railway_template(output_dir: &str, _project_name: Option<&str>) -> Result<()> {
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)?;

    let railway_content = include_str!("../templates/railway/railway.toml");
    fs::write(output_path.join("railway.toml"), railway_content)?;

    // Generate Dockerfile for Railway
    let dockerfile_content = include_str!("../templates/docker/Dockerfile");
    fs::write(output_path.join("Dockerfile"), dockerfile_content)?;

    Ok(())
}

/// Generate Fly.io deployment template
pub fn generate_fly_template(output_dir: &str, project_name: Option<&str>) -> Result<()> {
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)?;

    let fly_content = include_str!("../templates/fly/fly.toml");
    let customized_fly = if let Some(name) = project_name {
        fly_content.replace("shimmy-ai", &format!("{}-ai", name))
    } else {
        fly_content.to_string()
    };
    fs::write(output_path.join("fly.toml"), customized_fly)?;

    // Generate Dockerfile for Fly
    let dockerfile_content = include_str!("../templates/docker/Dockerfile");
    fs::write(output_path.join("Dockerfile"), dockerfile_content)?;

    Ok(())
}

/// Generate FastAPI integration template
pub fn generate_fastapi_template(output_dir: &str, _project_name: Option<&str>) -> Result<()> {
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)?;

    let main_content = include_str!("../templates/frameworks/fastapi/main.py");
    fs::write(output_path.join("main.py"), main_content)?;

    let requirements_content = include_str!("../templates/frameworks/fastapi/requirements.txt");
    fs::write(output_path.join("requirements.txt"), requirements_content)?;

    Ok(())
}

/// Generate Express.js integration template
pub fn generate_express_template(output_dir: &str, project_name: Option<&str>) -> Result<()> {
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)?;

    let app_content = include_str!("../templates/frameworks/express/app.js");
    fs::write(output_path.join("app.js"), app_content)?;

    let package_content = include_str!("../templates/frameworks/express/package.json");
    let customized_package = if let Some(name) = project_name {
        package_content.replace(
            "shimmy-express-integration",
            &format!("{}-shimmy-integration", name),
        )
    } else {
        package_content.to_string()
    };
    fs::write(output_path.join("package.json"), customized_package)?;

    Ok(())
}

/// Generic template generation function that dispatches to specific template generators
pub fn generate_template(
    template: &str,
    output_dir: &str,
    project_name: Option<&str>,
) -> Result<String> {
    match template.to_lowercase().as_str() {
        "docker" => {
            generate_docker_template(output_dir, project_name)?;
            Ok(format!("✅ Docker template generated in {}", output_dir))
        }
        "kubernetes" | "k8s" => {
            generate_kubernetes_template(output_dir, project_name)?;
            Ok(format!(
                "✅ Kubernetes template generated in {}",
                output_dir
            ))
        }
        "railway" => {
            generate_railway_template(output_dir, project_name)?;
            Ok(format!("✅ Railway template generated in {}", output_dir))
        }
        "fly" => {
            generate_fly_template(output_dir, project_name)?;
            Ok(format!("✅ Fly.io template generated in {}", output_dir))
        }
        "fastapi" => {
            generate_fastapi_template(output_dir, project_name)?;
            Ok(format!("✅ FastAPI template generated in {}", output_dir))
        }
        "express" => {
            generate_express_template(output_dir, project_name)?;
            Ok(format!(
                "✅ Express.js template generated in {}",
                output_dir
            ))
        }
        _ => {
            anyhow::bail!("Unknown template type: {}. Available: docker, kubernetes, railway, fly, fastapi, express", template)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chatml_render() {
        let template = TemplateFamily::ChatML;
        let messages = vec![("user".to_string(), "Hello".to_string())];
        let result = template.render(None, &messages, None);
        assert!(result.contains("<|im_start|>user"));
        assert!(result.contains("Hello"));
        assert!(result.contains("<|im_end|>"));
    }

    #[test]
    fn test_llama3_render() {
        let template = TemplateFamily::Llama3;
        let messages = vec![("user".to_string(), "Test".to_string())];
        let result = template.render(None, &messages, None);
        assert!(result.contains("<|start_header_id|>user<|end_header_id|>"));
        assert!(result.contains("Test"));
        assert!(result.contains("<|eot_id|>"));
    }

    #[test]
    fn test_openchat_render() {
        let template = TemplateFamily::OpenChat;
        let messages = vec![("user".to_string(), "Hi".to_string())];
        let result = template.render(None, &messages, None);
        assert!(result.contains("user: Hi"));
        assert!(result.contains("assistant: "));
    }

    #[test]
    fn test_chatml_stop_tokens() {
        let template = TemplateFamily::ChatML;
        let stop_tokens = template.stop_tokens();
        assert_eq!(stop_tokens.len(), 2);
        assert!(stop_tokens.contains(&"<|im_end|>".to_string()));
        assert!(stop_tokens.contains(&"<|im_start|>".to_string()));
    }

    #[test]
    fn test_llama3_stop_tokens() {
        let template = TemplateFamily::Llama3;
        let stop_tokens = template.stop_tokens();
        assert_eq!(stop_tokens.len(), 2);
        assert!(stop_tokens.contains(&"<|eot_id|>".to_string()));
        assert!(stop_tokens.contains(&"<|end_of_text|>".to_string()));
    }

    #[test]
    fn test_openchat_stop_tokens() {
        let template = TemplateFamily::OpenChat;
        let stop_tokens = template.stop_tokens();
        assert_eq!(stop_tokens.len(), 0);
    }

    #[test]
    fn test_chatml_render_with_system_and_input() {
        let template = TemplateFamily::ChatML;
        let messages = vec![("user".to_string(), "Hello".to_string())];
        let result = template.render(Some("You are helpful"), &messages, Some("New question"));
        assert!(result.contains("<|im_start|>system\nYou are helpful<|im_end|>"));
        assert!(result.contains("<|im_start|>user\nNew question<|im_end|>"));
        assert!(result.contains("<|im_start|>assistant\n"));
    }

    #[test]
    fn test_llama3_render_with_system_and_input() {
        let template = TemplateFamily::Llama3;
        let messages = vec![("user".to_string(), "Hi".to_string())];
        let result = template.render(Some("Be concise"), &messages, Some("Ask me anything"));
        assert!(result.contains("<|begin_of_text|><|start_header_id|>system<|end_header_id|>"));
        assert!(result.contains("Be concise<|eot_id|>"));
        assert!(result.contains("<|start_header_id|>assistant<|end_header_id|>"));
    }

    #[test]
    fn test_openchat_render_with_input() {
        let template = TemplateFamily::OpenChat;
        let messages = vec![("user".to_string(), "Hello".to_string())];
        let result = template.render(None, &messages, Some("Final question"));
        assert!(result.contains("user: Final question\nassistant: "));
    }

    #[test]
    fn test_openchat_render_no_input() {
        let template = TemplateFamily::OpenChat;
        let messages: Vec<(String, String)> = vec![];
        let result = template.render(None, &messages, None);
        assert_eq!(result, "assistant: ");
    }

    #[test]
    fn test_generate_docker_template_creates_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output = temp_dir.path().to_str().unwrap();
        generate_docker_template(output, None).unwrap();
        assert!(temp_dir.path().join("Dockerfile").exists());
        assert!(temp_dir.path().join("docker-compose.yml").exists());
        assert!(temp_dir.path().join("nginx.conf").exists());
        assert!(temp_dir.path().join(".dockerignore").exists());
    }

    #[test]
    fn test_generate_docker_template_project_name_substitution() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output = temp_dir.path().to_str().unwrap();
        generate_docker_template(output, Some("myproject")).unwrap();
        let compose = fs::read_to_string(temp_dir.path().join("docker-compose.yml")).unwrap();
        assert!(compose.contains("myproject-shimmy"));
    }

    #[test]
    fn test_generate_kubernetes_template_creates_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output = temp_dir.path().to_str().unwrap();
        generate_kubernetes_template(output, Some("testapp")).unwrap();
        assert!(temp_dir.path().join("deployment.yaml").exists());
        assert!(temp_dir.path().join("service.yaml").exists());
        assert!(temp_dir.path().join("configmap.yaml").exists());
        let deployment = fs::read_to_string(temp_dir.path().join("deployment.yaml")).unwrap();
        assert!(deployment.contains("testapp-deployment"));
    }

    #[test]
    fn test_generate_railway_template_creates_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output = temp_dir.path().to_str().unwrap();
        generate_railway_template(output, None).unwrap();
        assert!(temp_dir.path().join("railway.toml").exists());
        assert!(temp_dir.path().join("Dockerfile").exists());
    }

    #[test]
    fn test_generate_fly_template_creates_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output = temp_dir.path().to_str().unwrap();
        generate_fly_template(output, Some("myfly")).unwrap();
        assert!(temp_dir.path().join("fly.toml").exists());
        assert!(temp_dir.path().join("Dockerfile").exists());
        let fly = fs::read_to_string(temp_dir.path().join("fly.toml")).unwrap();
        assert!(fly.contains("myfly-ai"));
    }

    #[test]
    fn test_generate_fastapi_template_creates_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output = temp_dir.path().to_str().unwrap();
        generate_fastapi_template(output, None).unwrap();
        assert!(temp_dir.path().join("main.py").exists());
        assert!(temp_dir.path().join("requirements.txt").exists());
    }

    #[test]
    fn test_generate_express_template_creates_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output = temp_dir.path().to_str().unwrap();
        generate_express_template(output, Some("myexpress")).unwrap();
        assert!(temp_dir.path().join("app.js").exists());
        assert!(temp_dir.path().join("package.json").exists());
        let pkg = fs::read_to_string(temp_dir.path().join("package.json")).unwrap();
        assert!(pkg.contains("myexpress-shimmy-integration"));
    }

    #[test]
    fn test_generate_template_dispatcher_docker() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output = temp_dir.path().to_str().unwrap();
        let msg = generate_template("docker", output, None).unwrap();
        assert!(msg.contains("Docker template generated"));
    }

    #[test]
    fn test_generate_template_dispatcher_kubernetes_alias() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let output = temp_dir.path().to_str().unwrap();
        let msg = generate_template("k8s", output, Some("myapp")).unwrap();
        assert!(msg.contains("Kubernetes template generated"));
    }

    #[test]
    fn test_generate_template_dispatcher_all_types() {
        let types = ["railway", "fly", "fastapi", "express"];
        for template_type in &types {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let output = temp_dir.path().to_str().unwrap();
            let result = generate_template(template_type, output, None);
            assert!(result.is_ok(), "Template type '{}' failed", template_type);
        }
    }

    #[test]
    fn test_generate_template_unknown_type() {
        let result = generate_template("bogustype", "/tmp", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown template type"));
    }
}
