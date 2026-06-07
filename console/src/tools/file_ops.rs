use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read the contents of a file, optionally within a line range"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file" },
                "start_line": { "type": "integer", "description": "Start line (1-based, inclusive)" },
                "end_line": { "type": "integer", "description": "End line (1-based, inclusive)" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let file_path = Path::new(path);
        if !file_path.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "File not found: {}",
                path
            )));
        }
        let contents = std::fs::read_to_string(file_path)?;
        let start = args
            .get_i64("start_line")
            .map(|n| (n as usize).saturating_sub(1));
        let end = args.get_i64("end_line").map(|n| n as usize);
        let result = match (start, end) {
            (Some(s), Some(e)) => {
                let lines: Vec<&str> = contents.lines().collect();
                lines[s..e.min(lines.len())].join("\n")
            }
            (Some(s), None) => contents.lines().skip(s).collect::<Vec<_>>().join("\n"),
            (None, Some(e)) => contents.lines().take(e).collect::<Vec<_>>().join("\n"),
            (None, None) => contents,
        };
        Ok(ToolResult::success(result))
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    fn description(&self) -> &str {
        "Write content to a file, creating parent directories as needed"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to write to" },
                "content": { "type": "string", "description": "Content to write" }
            },
            "required": ["path", "content"]
        })
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let content = args.require_str("content")?;
        let file_path = Path::new(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file_path, content)?;
        Ok(ToolResult::success(format!(
            "Written {} bytes to {}",
            content.len(),
            path
        )))
    }
}

pub struct ListFilesTool;

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }
    fn description(&self) -> &str {
        "List files and directories at a path"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Directory path to list" }
            },
            "required": ["path"]
        })
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let dir = Path::new(path);
        if !dir.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "Path not found: {}",
                path
            )));
        }
        let mut entries: Vec<String> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    format!("{}/", name)
                } else {
                    name
                }
            })
            .collect();
        entries.sort();
        let output = entries.join("\n");
        let data = serde_json::json!({ "entries": entries });
        Ok(ToolResult::success_with_data(output, data))
    }
}
