use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

pub struct AnalyzeProjectTool;

#[async_trait]
impl Tool for AnalyzeProjectTool {
    fn name(&self) -> &str {
        "analyze_project"
    }
    fn description(&self) -> &str {
        "Analyze project structure, dependencies, and type"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Project root path (default: current dir)" }
            }
        })
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.get_str("path").unwrap_or(".");
        let dir = Path::new(path);

        let mut project_type = "unknown";
        let mut deps: Vec<String> = vec![];
        let mut files: Vec<String> = vec![];

        if dir.join("Cargo.toml").exists() {
            project_type = "rust";
            let content = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
            // Extract dependency names from [dependencies] section
            let mut in_deps = false;
            for line in content.lines() {
                if line.trim() == "[dependencies]" {
                    in_deps = true;
                    continue;
                }
                if in_deps && line.starts_with('[') {
                    in_deps = false;
                }
                if in_deps {
                    if let Some(name) = line.split('=').next() {
                        let name = name.trim().to_string();
                        if !name.is_empty() {
                            deps.push(name);
                        }
                    }
                }
            }
        } else if dir.join("package.json").exists() {
            project_type = "node";
        } else if dir.join("pyproject.toml").exists() || dir.join("setup.py").exists() {
            project_type = "python";
        }

        // List top-level files
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                files.push(entry.file_name().to_string_lossy().to_string());
            }
            files.sort();
        }

        let data = serde_json::json!({
            "project_type": project_type,
            "path": path,
            "dependencies": deps,
            "top_level_files": files
        });
        let output = format!(
            "Project type: {}\nDependencies: {}\nFiles: {}",
            project_type,
            deps.len(),
            files.len()
        );
        Ok(ToolResult::success_with_data(output, data))
    }
}

pub struct SyntaxCheckTool;

#[async_trait]
impl Tool for SyntaxCheckTool {
    fn name(&self) -> &str {
        "syntax_check"
    }
    fn description(&self) -> &str {
        "Check syntax of a source file using the appropriate language tool"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to source file" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let result = match ext {
            "rs" => {
                // Use rustfmt --check for syntax validation
                let output = std::process::Command::new("rustfmt")
                    .args(["--check", "--edition", "2021", path])
                    .output();
                match output {
                    Ok(o) if o.status.success() => "Syntax OK".to_string(),
                    Ok(o) => format!("Issues found:\n{}", String::from_utf8_lossy(&o.stdout)),
                    Err(_) => "rustfmt not available".to_string(),
                }
            }
            "json" => {
                let content = std::fs::read_to_string(path)?;
                match serde_json::from_str::<Value>(&content) {
                    Ok(_) => "Valid JSON".to_string(),
                    Err(e) => format!("JSON parse error: {}", e),
                }
            }
            _ => format!("No syntax checker for .{} files", ext),
        };

        Ok(ToolResult::success(result))
    }
}
