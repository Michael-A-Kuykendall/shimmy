//! File operation tools

use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use std::fs;
use std::path::Path;

/// Tool for reading file contents
pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Starting line number (1-based, optional)"
                },
                "end_line": {
                    "type": "integer",
                    "description": "Ending line number (1-based, optional)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let start_line = args.get_i64("start_line").map(|n| n as usize);
        let end_line = args.get_i64("end_line").map(|n| n as usize);

        let file_path = Path::new(path);
        if !file_path.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "File does not exist: {}",
                path
            )));
        }

        let contents = fs::read_to_string(file_path)?;

        let result = match (start_line, end_line) {
            (Some(start), Some(end)) => {
                let lines: Vec<&str> = contents.lines().collect();
                let start_idx = start.saturating_sub(1);
                let end_idx = end.min(lines.len());
                lines[start_idx..end_idx].join("\n")
            }
            (Some(start), None) => {
                let lines: Vec<&str> = contents.lines().collect();
                let start_idx = start.saturating_sub(1);
                lines[start_idx..].join("\n")
            }
            (None, Some(end)) => {
                let lines: Vec<&str> = contents.lines().collect();
                let end_idx = end.min(lines.len());
                lines[..end_idx].join("\n")
            }
            (None, None) => contents,
        };

        Ok(ToolResult::success(result))
    }
}

/// Tool for writing file contents
pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "append": {
                    "type": "boolean",
                    "description": "Whether to append instead of overwrite"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let content = args.require_str("content")?;
        let append = args.get_bool("append", false);

        let file_path = Path::new(path);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if append {
            use std::fs::OpenOptions;
            use std::io::Write;

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?;

            file.write_all(content.as_bytes())?;
        } else {
            fs::write(file_path, content)?;
        }

        Ok(ToolResult::success(format!(
            "Successfully {} file: {}",
            if append { "appended to" } else { "wrote" },
            path
        )))
    }
}

/// Tool for listing directory contents
pub struct ListFilesTool;

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "List files and directories in a path"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to list contents of"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to list recursively"
                },
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to filter files"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let path = args.require_str("path")?;
        let recursive = args.get_bool("recursive", false);
        let _pattern = args.get_str("pattern");

        let dir_path = Path::new(path);
        if !dir_path.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "Path does not exist: {}",
                path
            )));
        }

        if !dir_path.is_dir() {
            return Err(ToolError::ExecutionFailed(format!(
                "Path is not a directory: {}",
                path
            )));
        }

        let mut entries = Vec::new();

        if recursive {
            collect_entries_recursive(dir_path, dir_path, &mut entries)?;
        } else {
            for entry in fs::read_dir(dir_path)? {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.path().is_dir();
                entries.push(if is_dir {
                    format!("{}/", name)
                } else {
                    name
                });
            }
        }

        entries.sort();
        Ok(ToolResult::success(entries.join("\n")))
    }
}

fn collect_entries_recursive(
    base: &Path,
    current: &Path,
    entries: &mut Vec<String>,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);
        let display = relative.to_string_lossy().to_string();

        if path.is_dir() {
            entries.push(format!("{}/", display));
            // Skip common ignored directories
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !matches!(name, "target" | "node_modules" | ".git" | "__pycache__") {
                collect_entries_recursive(base, &path, entries)?;
            }
        } else {
            entries.push(display);
        }
    }
    Ok(())
}
