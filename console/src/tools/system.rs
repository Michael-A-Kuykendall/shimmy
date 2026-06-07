use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;
use serde_json::Value;

pub struct SystemInfoTool;

#[async_trait]
impl Tool for SystemInfoTool {
    fn name(&self) -> &str {
        "system_info"
    }
    fn description(&self) -> &str {
        "Get OS, architecture, working directory, and environment info"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }
    async fn execute(&self, _args: ToolArgs) -> Result<ToolResult, ToolError> {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".into());

        let data = serde_json::json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "cwd": cwd,
            "home": std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default(),
        });

        let output = format!(
            "OS: {} ({})\nCWD: {}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            cwd
        );
        Ok(ToolResult::success_with_data(output, data))
    }
}
