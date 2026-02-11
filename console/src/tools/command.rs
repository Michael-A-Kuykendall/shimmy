//! Shell command execution tool

use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use std::process::Command;

/// Tool for executing shell commands
pub struct ShellCommandTool;

#[async_trait]
impl Tool for ShellCommandTool {
    fn name(&self) -> &str {
        "shell_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command and return its output"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory for the command"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Timeout in seconds (default: 30)"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let command_str = args.require_str("command")?;
        let working_dir = args.get_str("working_dir");
        let _timeout = args.get_i64("timeout_secs").unwrap_or(30);

        // Use appropriate shell based on platform
        let (shell, shell_arg) = if cfg!(windows) {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        let mut cmd = Command::new(shell);
        cmd.arg(shell_arg).arg(command_str);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();

        if !stdout.is_empty() {
            result.push_str(&stdout);
        }

        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push_str("\n--- stderr ---\n");
            }
            result.push_str(&stderr);
        }

        if output.status.success() {
            Ok(ToolResult::success(result))
        } else {
            Ok(ToolResult::failure(format!(
                "Command exited with status {}\n{}",
                output.status.code().unwrap_or(-1),
                result
            )))
        }
    }
}
