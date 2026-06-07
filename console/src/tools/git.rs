use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use serde_json::Value;
use std::process::Command;

fn run_git(args: &[&str]) -> Result<String, ToolError> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| ToolError::ExecutionFailed(format!("git command failed: {}", e)))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(ToolError::ExecutionFailed(stderr))
    }
}

pub struct GitStatusTool;

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }
    fn description(&self) -> &str {
        "Show working directory git status"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }
    async fn execute(&self, _args: ToolArgs) -> Result<ToolResult, ToolError> {
        let out = run_git(&["status", "--porcelain"])?;
        let lines: Vec<&str> = out.lines().collect();
        let data = serde_json::json!({ "files": lines });
        Ok(ToolResult::success_with_data(
            if out.is_empty() {
                "Working tree clean".into()
            } else {
                out
            },
            data,
        ))
    }
}

pub struct GitDiffTool;

#[async_trait]
impl Tool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }
    fn description(&self) -> &str {
        "Show git diff for the repo or a specific file"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file": { "type": "string", "description": "Specific file to diff (optional)" }
            }
        })
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let out = if let Some(file) = args.get_str("file") {
            run_git(&["diff", file])?
        } else {
            run_git(&["diff"])?
        };
        Ok(ToolResult::success(if out.is_empty() {
            "No changes".into()
        } else {
            out
        }))
    }
}

pub struct GitCommitTool;

#[async_trait]
impl Tool for GitCommitTool {
    fn name(&self) -> &str {
        "git_commit"
    }
    fn description(&self) -> &str {
        "Stage all changes and commit with a message"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string", "description": "Commit message" }
            },
            "required": ["message"]
        })
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let message = args.require_str("message")?;
        run_git(&["add", "."])?;
        let out = run_git(&["commit", "-m", message])?;
        Ok(ToolResult::success(out))
    }
}

pub struct GitLogTool;

#[async_trait]
impl Tool for GitLogTool {
    fn name(&self) -> &str {
        "git_log"
    }
    fn description(&self) -> &str {
        "Show recent commit history"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Number of commits (default 10)" }
            }
        })
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let limit = args.get_i64("limit").unwrap_or(10).to_string();
        let out = run_git(&["log", "--oneline", &format!("-{}", limit)])?;
        let commits: Vec<&str> = out.lines().collect();
        let data = serde_json::json!({ "commits": commits });
        Ok(ToolResult::success_with_data(out, data))
    }
}
