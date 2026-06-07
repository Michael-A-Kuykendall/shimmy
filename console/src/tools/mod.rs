use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub mod analysis;
pub mod command;
pub mod file_ops;
pub mod git;
pub mod system;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
    #[error("Invalid argument '{0}': {1}")]
    InvalidArgument(String, String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArgs {
    #[serde(flatten)]
    pub args: HashMap<String, Value>,
}

impl ToolArgs {
    pub fn new(args: HashMap<String, Value>) -> Self {
        Self { args }
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.args.get(key)?.as_str()
    }

    pub fn require_str(&self, key: &str) -> Result<&str, ToolError> {
        self.get_str(key)
            .ok_or_else(|| ToolError::MissingArgument(key.to_string()))
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.args
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.args.get(key)?.as_i64()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub data: Option<Value>,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            data: None,
        }
    }

    pub fn success_with_data(output: impl Into<String>, data: Value) -> Self {
        Self {
            success: true,
            output: output.into(),
            data: Some(data),
        }
    }

    pub fn failure(output: impl Into<String>) -> Self {
        Self {
            success: false,
            output: output.into(),
            data: None,
        }
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult, ToolError>;
}

#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<Mutex<HashMap<String, Arc<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, tool: Arc<dyn Tool>) {
        let mut map = self.tools.lock().unwrap();
        map.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.lock().unwrap().get(name).cloned()
    }

    pub fn list(&self) -> Vec<String> {
        self.tools.lock().unwrap().keys().cloned().collect()
    }

    /// Return OpenAI-compatible tool definitions for the chat API
    pub fn to_openai_tools(&self) -> Vec<Value> {
        self.tools
            .lock()
            .unwrap()
            .values()
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameters()
                    }
                })
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the default registry with all 14 tools registered
pub fn build_default_registry() -> ToolRegistry {
    let registry = ToolRegistry::new();
    registry.register(Arc::new(file_ops::ReadFileTool));
    registry.register(Arc::new(file_ops::WriteFileTool));
    registry.register(Arc::new(file_ops::ListFilesTool));
    registry.register(Arc::new(git::GitStatusTool));
    registry.register(Arc::new(git::GitDiffTool));
    registry.register(Arc::new(git::GitCommitTool));
    registry.register(Arc::new(git::GitLogTool));
    registry.register(Arc::new(analysis::AnalyzeProjectTool));
    registry.register(Arc::new(analysis::SyntaxCheckTool));
    registry.register(Arc::new(command::ShellCommandTool));
    registry.register(Arc::new(system::SystemInfoTool));
    registry
}
