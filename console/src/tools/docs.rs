use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use serde_json::Value;
use std::process::Command;

// ── ExplainCommandTool ────────────────────────────────────────────────────────

pub struct ExplainCommandTool;

#[async_trait]
impl Tool for ExplainCommandTool {
    fn name(&self) -> &str {
        "explain_command"
    }

    fn description(&self) -> &str {
        "Run a command with --help and return the output. \
         Useful for understanding CLI tool usage without leaving the session."
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to explain (e.g. 'git', 'cargo', 'rustc')"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let command = args.require_str("command")?;

        // Split on whitespace so callers can pass "git commit" or just "git"
        let mut parts = command.split_whitespace();
        let program = match parts.next() {
            Some(p) => p,
            None => {
                return Err(ToolError::InvalidArgument(
                    "command".into(),
                    "must not be empty".into(),
                ))
            }
        };
        let mut sub_args: Vec<&str> = parts.collect();
        sub_args.push("--help");

        let result = Command::new(program).args(&sub_args).output();

        match result {
            Ok(output) => {
                // Many commands print help to stderr; combine both
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = if stdout.is_empty() {
                    stderr.into_owned()
                } else if stderr.is_empty() {
                    stdout.into_owned()
                } else {
                    format!("{}\n{}", stdout, stderr)
                };

                let text = if combined.trim().is_empty() {
                    format!("No help output from '{}'.", command)
                } else {
                    combined
                };

                Ok(ToolResult::success(text))
            }
            Err(e) => {
                // Command not found or not executable
                Ok(ToolResult::success(format!(
                    "Could not run '{}': {}. \
                     The command may not be installed or not on PATH.",
                    command, e
                )))
            }
        }
    }
}

// ── GetHelpTool ───────────────────────────────────────────────────────────────

pub struct GetHelpTool;

/// Static help catalog for all 14 tools.
fn tool_help_catalog() -> Vec<(&'static str, &'static str, &'static str)> {
    // (name, description, params summary)
    vec![
        (
            "read_file",
            "Read the contents of a file, optionally within a line range.",
            "path (required), start_line (optional), end_line (optional)",
        ),
        (
            "write_file",
            "Write content to a file, creating parent directories as needed.",
            "path (required), content (required)",
        ),
        (
            "list_files",
            "List files and directories at a path.",
            "path (required)",
        ),
        (
            "git_status",
            "Show git working tree status (porcelain format).",
            "(no parameters)",
        ),
        (
            "git_diff",
            "Show git diff for a specific file or the entire repository.",
            "file (optional)",
        ),
        (
            "git_commit",
            "Stage all changes and commit with a message.",
            "message (required)",
        ),
        (
            "git_log",
            "Show recent commit history in one-line format.",
            "limit (optional, default 10)",
        ),
        (
            "analyze_project",
            "Analyze project structure, build files, and dependencies.",
            "(no parameters)",
        ),
        (
            "syntax_check",
            "Check the syntax of a source file (multi-language).",
            "path (required)",
        ),
        (
            "shell_command",
            "Execute a shell command and return stdout, stderr, and exit code.",
            "command (required), args (optional array)",
        ),
        (
            "system_info",
            "Get OS, architecture, working directory, and home directory.",
            "(no parameters)",
        ),
        (
            "explain_command",
            "Run a command with --help and return its output.",
            "command (required)",
        ),
        (
            "get_help",
            "List all available tools or get details about a specific tool.",
            "topic (optional, default 'general')",
        ),
        (
            "read_image",
            "Read an image file and return base64 data, MIME type, and dimensions.",
            "path (required), mode (optional: 'base64'|'meta'|'ocr', default 'base64')",
        ),
    ]
}

#[async_trait]
impl Tool for GetHelpTool {
    fn name(&self) -> &str {
        "get_help"
    }

    fn description(&self) -> &str {
        "List all available tools (topic='general') or get details about a specific tool by name."
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "topic": {
                    "type": "string",
                    "description": "Tool name for specific help, or 'general' for the full list"
                }
            }
        })
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let topic = args.get_str("topic").unwrap_or("general");
        let catalog = tool_help_catalog();

        if topic == "general" {
            let mut lines = vec![
                "Shimmy Console — Available Tools".to_string(),
                "─────────────────────────────────".to_string(),
            ];
            for (name, desc, params) in &catalog {
                lines.push(format!("\n  {}", name));
                lines.push(format!("    {}", desc));
                lines.push(format!("    Parameters: {}", params));
            }
            lines.push("\n─────────────────────────────────".to_string());
            lines.push(format!("Total: {} tools", catalog.len()));
            return Ok(ToolResult::success(lines.join("\n")));
        }

        // Look up a specific tool by name
        if let Some((name, desc, params)) = catalog.iter().find(|(n, _, _)| *n == topic) {
            let text = format!(
                "Tool: {}\nDescription: {}\nParameters: {}",
                name, desc, params
            );
            return Ok(ToolResult::success(text));
        }

        // Unknown topic — suggest the tool list
        Ok(ToolResult::success(format!(
            "No tool named '{}'. Use topic='general' to see all {} available tools.",
            topic,
            catalog.len()
        )))
    }
}
