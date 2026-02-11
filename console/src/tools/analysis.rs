//! Code analysis tools

use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use std::path::Path;

/// Tool for analyzing project structure
pub struct ProjectAnalysisTool;

#[async_trait]
impl Tool for ProjectAnalysisTool {
    fn name(&self) -> &str {
        "analyze_project"
    }

    fn description(&self) -> &str {
        "Analyze project structure, dependencies, and architecture"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the project root"
                },
                "include_dependencies": {
                    "type": "boolean",
                    "description": "Whether to analyze dependencies"
                }
            },
            "required": ["path"]
        })
    }

    fn requires_license(&self) -> bool {
        true
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let include_deps = args.get_bool("include_dependencies", true);

        let project_path = Path::new(path);
        if !project_path.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "Project path does not exist: {}",
                path
            )));
        }

        let mut analysis = String::new();
        analysis.push_str(&format!("Project Analysis: {}\n", path));
        analysis.push_str("=".repeat(50).as_str());
        analysis.push('\n');

        // Check for common project files
        if project_path.join("Cargo.toml").exists() {
            analysis.push_str("Type: Rust project\n");
            if include_deps {
                if let Ok(contents) = std::fs::read_to_string(project_path.join("Cargo.toml")) {
                    analysis.push_str("\nCargo.toml detected\n");
                    // Count dependencies (simple heuristic)
                    let dep_count = contents.matches("[dependencies]").count()
                        + contents.matches("[dev-dependencies]").count();
                    analysis.push_str(&format!("Dependency sections: {}\n", dep_count));
                }
            }
        } else if project_path.join("package.json").exists() {
            analysis.push_str("Type: Node.js project\n");
        } else if project_path.join("pyproject.toml").exists() {
            analysis.push_str("Type: Python project\n");
        } else {
            analysis.push_str("Type: Unknown\n");
        }

        // Count source files
        let rs_count = count_files_with_extension(project_path, "rs");
        let js_count = count_files_with_extension(project_path, "js");
        let ts_count = count_files_with_extension(project_path, "ts");
        let py_count = count_files_with_extension(project_path, "py");

        analysis.push_str("\nFile counts:\n");
        if rs_count > 0 {
            analysis.push_str(&format!("  Rust (.rs): {}\n", rs_count));
        }
        if js_count > 0 {
            analysis.push_str(&format!("  JavaScript (.js): {}\n", js_count));
        }
        if ts_count > 0 {
            analysis.push_str(&format!("  TypeScript (.ts): {}\n", ts_count));
        }
        if py_count > 0 {
            analysis.push_str(&format!("  Python (.py): {}\n", py_count));
        }

        Ok(ToolResult::success(analysis))
    }
}

fn count_files_with_extension(dir: &Path, ext: &str) -> usize {
    walkdir(dir)
        .filter(|p| p.extension().map_or(false, |e| e == ext))
        .count()
}

fn walkdir(dir: &Path) -> impl Iterator<Item = std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip common ignored directories
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !matches!(name, "target" | "node_modules" | ".git" | "__pycache__") {
                    files.extend(walkdir(&path));
                }
            } else {
                files.push(path);
            }
        }
    }
    files.into_iter()
}

/// Tool for checking syntax of code files
pub struct SyntaxCheckTool;

#[async_trait]
impl Tool for SyntaxCheckTool {
    fn name(&self) -> &str {
        "syntax_check"
    }

    fn description(&self) -> &str {
        "Check syntax of source code files"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to check"
                },
                "language": {
                    "type": "string",
                    "description": "Programming language (auto-detected if not specified)"
                }
            },
            "required": ["path"]
        })
    }

    fn requires_license(&self) -> bool {
        true
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let file_path = Path::new(path);

        if !file_path.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "File does not exist: {}",
                path
            )));
        }

        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let language = args.get_str("language").unwrap_or(extension);

        // Basic syntax validation (placeholder - real implementation would use tree-sitter or similar)
        let contents = std::fs::read_to_string(file_path)?;

        let result = match language {
            "json" => check_json_syntax(&contents),
            "rs" | "rust" => check_rust_syntax(&contents),
            _ => Ok("Syntax check not implemented for this language".to_string()),
        };

        match result {
            Ok(msg) => Ok(ToolResult::success(msg)),
            Err(e) => Ok(ToolResult::failure(format!("Syntax error: {}", e))),
        }
    }
}

fn check_json_syntax(contents: &str) -> Result<String, String> {
    match serde_json::from_str::<serde_json::Value>(contents) {
        Ok(_) => Ok("JSON syntax is valid".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn check_rust_syntax(contents: &str) -> Result<String, String> {
    // Basic bracket matching check
    let mut brace_count = 0i32;
    let mut paren_count = 0i32;
    let mut bracket_count = 0i32;

    for ch in contents.chars() {
        match ch {
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            _ => {}
        }

        if brace_count < 0 || paren_count < 0 || bracket_count < 0 {
            return Err("Unmatched closing bracket".to_string());
        }
    }

    if brace_count != 0 {
        return Err(format!("Unmatched braces: {} unclosed", brace_count));
    }
    if paren_count != 0 {
        return Err(format!("Unmatched parentheses: {} unclosed", paren_count));
    }
    if bracket_count != 0 {
        return Err(format!("Unmatched brackets: {} unclosed", bracket_count));
    }

    Ok("Basic Rust syntax check passed".to_string())
}
