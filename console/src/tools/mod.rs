//! Tool system for shimmy console
//!
//! Tools are executable actions that the console can invoke to interact
//! with the filesystem, run commands, analyze code, etc.

pub mod analysis;
pub mod command;
pub mod file_ops;
pub mod image;
pub mod loader;
pub mod system;

#[cfg(test)]
mod analysis_tests;
#[cfg(test)]
mod command_git_tests;
#[cfg(test)]
mod docs_tests;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Arguments passed to a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArgs {
    /// Named arguments
    #[serde(flatten)]
    pub args: HashMap<String, serde_json::Value>,
}

impl ToolArgs {
    /// Create empty args
    pub fn new() -> Self {
        Self {
            args: HashMap::new(),
        }
    }

    /// Get a string argument
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.args.get(key).and_then(|v| v.as_str())
    }

    /// Get a required string argument
    pub fn require_str(&self, key: &str) -> Result<&str, ToolError> {
        self.get_str(key)
            .ok_or_else(|| ToolError::MissingArgument(key.to_string()))
    }

    /// Get a boolean argument with default
    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.args
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// Get an integer argument
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.args.get(key).and_then(|v| v.as_i64())
    }
}

impl Default for ToolArgs {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool succeeded
    pub success: bool,
    /// Output from the tool
    pub output: String,
    /// Optional structured data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            data: None,
        }
    }

    /// Create a successful result with data
    pub fn success_with_data(output: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            output: output.into(),
            data: Some(data),
        }
    }

    /// Create a failure result
    pub fn failure(output: impl Into<String>) -> Self {
        Self {
            success: false,
            output: output.into(),
            data: None,
        }
    }
}

/// Errors that can occur during tool execution
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    #[error("Invalid argument value for {0}: {1}")]
    InvalidArgument(String, String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// A tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Name of the tool to call
    pub name: String,
    /// Arguments for the tool
    pub arguments: ToolArgs,
    /// Optional call ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
}

/// Trait for tools that can be executed
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the name of this tool
    fn name(&self) -> &str;

    /// Get a description of what this tool does
    fn description(&self) -> &str;

    /// Get the parameter schema for this tool
    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    /// Whether this tool requires a license
    fn requires_license(&self) -> bool {
        false
    }

    /// Execute the tool with the given arguments
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError>;
}

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Get all registered tools
    pub fn all(&self) -> impl Iterator<Item = &Arc<dyn Tool>> {
        self.tools.values()
    }

    /// Get tool names
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.tools.keys().map(|s| s.as_str())
    }

    /// Execute a tool call
    pub async fn execute(&self, call: &ToolCall) -> Result<ToolResult, ToolError> {
        let tool = self
            .get(&call.name)
            .ok_or_else(|| ToolError::ExecutionFailed(format!("Unknown tool: {}", call.name)))?;

        tool.execute(call.arguments.clone()).await
    }

    /// Create a registry with default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register file operation tools
        registry.register(Arc::new(file_ops::ReadFileTool));
        registry.register(Arc::new(file_ops::WriteFileTool));
        registry.register(Arc::new(file_ops::ListFilesTool));

        // Register command tool
        registry.register(Arc::new(command::ShellCommandTool));

        // Register system tool
        registry.register(Arc::new(system::SystemInfoTool));

        // Register analysis tools
        registry.register(Arc::new(analysis::ProjectAnalysisTool));
        registry.register(Arc::new(analysis::SyntaxCheckTool));

        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Re-exports
pub use analysis::{ProjectAnalysisTool, SyntaxCheckTool};
pub use command::ShellCommandTool;
pub use file_ops::{ListFilesTool, ReadFileTool, WriteFileTool};
pub use image::ReadImageTool;
pub use loader::{load_snapins, SnapInDefinition, ToolManifest};
pub use system::SystemInfoTool;
