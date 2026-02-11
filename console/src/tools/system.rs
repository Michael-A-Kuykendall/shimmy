//! System information tool

use super::{Tool, ToolArgs, ToolError, ToolResult};
use async_trait::async_trait;

/// Tool for getting system information
pub struct SystemInfoTool;

#[async_trait]
impl Tool for SystemInfoTool {
    fn name(&self) -> &str {
        "system_info"
    }

    fn description(&self) -> &str {
        "Get information about the current system environment"
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "description": "Category of info to retrieve: os, env, cwd, all",
                    "enum": ["os", "env", "cwd", "all"]
                }
            },
            "required": []
        })
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError> {
        let category = args.get_str("category").unwrap_or("all");

        let mut info = String::new();

        if matches!(category, "os" | "all") {
            info.push_str("=== Operating System ===\n");
            info.push_str(&format!("OS: {}\n", std::env::consts::OS));
            info.push_str(&format!("Architecture: {}\n", std::env::consts::ARCH));
            info.push_str(&format!("Family: {}\n", std::env::consts::FAMILY));
        }

        if matches!(category, "cwd" | "all") {
            if !info.is_empty() {
                info.push('\n');
            }
            info.push_str("=== Working Directory ===\n");
            if let Ok(cwd) = std::env::current_dir() {
                info.push_str(&format!("CWD: {}\n", cwd.display()));
            } else {
                info.push_str("CWD: (unable to determine)\n");
            }
        }

        if matches!(category, "env" | "all") {
            if !info.is_empty() {
                info.push('\n');
            }
            info.push_str("=== Environment Variables ===\n");

            // Only show safe/relevant environment variables
            let safe_vars = ["PATH", "HOME", "USER", "SHELL", "TERM", "LANG", "PWD"];
            for var in &safe_vars {
                if let Ok(value) = std::env::var(var) {
                    // Truncate long values like PATH
                    let display_value = if value.len() > 100 {
                        format!("{}...", &value[..100])
                    } else {
                        value
                    };
                    info.push_str(&format!("{}: {}\n", var, display_value));
                }
            }
        }

        Ok(ToolResult::success(info))
    }
}
