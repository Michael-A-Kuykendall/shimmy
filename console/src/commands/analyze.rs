//! Analyze command

use clap::Args;

/// Arguments for the analyze command
#[derive(Debug, Args)]
pub struct AnalyzeCommand {
    /// Path to analyze
    #[arg(default_value = ".")]
    pub path: String,

    /// Include dependency analysis
    #[arg(short, long)]
    pub dependencies: bool,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

impl AnalyzeCommand {
    /// Execute the analyze command
    pub async fn run(&self) -> anyhow::Result<()> {
        use crate::tools::{ProjectAnalysisTool, Tool, ToolArgs};

        let tool = ProjectAnalysisTool;

        let mut args = ToolArgs::new();
        args.args.insert(
            "path".to_string(),
            serde_json::Value::String(self.path.clone()),
        );
        args.args.insert(
            "include_dependencies".to_string(),
            serde_json::Value::Bool(self.dependencies),
        );

        let result = tool.execute(args).await?;

        if self.format == "json" {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{}", result.output);
        }

        Ok(())
    }
}
