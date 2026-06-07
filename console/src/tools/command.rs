use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use serde_json::Value;

pub struct ShellCommandTool;

#[async_trait]
impl Tool for ShellCommandTool {
    fn name(&self) -> &str {
        "shell_command"
    }
    fn description(&self) -> &str {
        "Execute a shell command and return stdout, stderr, and exit code"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Command to execute" },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Command arguments"
                },
                "cwd": { "type": "string", "description": "Working directory (optional)" }
            },
            "required": ["command"]
        })
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let command = args.require_str("command")?;
        let cmd_args: Vec<String> = args
            .args
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut cmd = std::process::Command::new(command);
        cmd.args(&cmd_args);
        if let Some(cwd) = args.get_str("cwd") {
            cmd.current_dir(cwd);
        }

        let output = cmd.output().map_err(|e| {
            ToolError::ExecutionFailed(format!("Failed to run '{}': {}", command, e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        let data = serde_json::json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
            "success": output.status.success()
        });

        let display = if stderr.is_empty() {
            stdout.clone()
        } else {
            format!("{}\nSTDERR: {}", stdout, stderr)
        };

        if output.status.success() {
            Ok(ToolResult::success_with_data(display, data))
        } else {
            Ok(ToolResult::failure(display))
        }
    }
}
