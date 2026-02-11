//! Edit command

use clap::Args;

/// Arguments for the edit command
#[derive(Debug, Args)]
pub struct EditCommand {
    /// File to edit
    pub file: String,

    /// Line number to start at
    #[arg(short, long)]
    pub line: Option<usize>,

    /// End line number (for range edits)
    #[arg(short, long)]
    pub end_line: Option<usize>,

    /// Content to replace with
    #[arg(short, long)]
    pub content: Option<String>,

    /// Use AI to generate edit
    #[arg(long)]
    pub ai: bool,

    /// AI prompt for edit
    #[arg(short, long)]
    pub prompt: Option<String>,
}

impl EditCommand {
    /// Execute the edit command
    pub async fn run(&self) -> anyhow::Result<()> {
        use crate::tools::{ReadFileTool, Tool, ToolArgs, WriteFileTool};

        // Read current content
        let read_tool = ReadFileTool;
        let mut args = ToolArgs::new();
        args.args.insert(
            "path".to_string(),
            serde_json::Value::String(self.file.clone()),
        );

        if let Some(line) = self.line {
            args.args.insert(
                "start_line".to_string(),
                serde_json::Value::Number(line.into()),
            );
        }
        if let Some(end) = self.end_line {
            args.args
                .insert("end_line".to_string(), serde_json::Value::Number(end.into()));
        }

        let result = read_tool.execute(args).await?;
        println!("Current content:\n{}", result.output);

        // If content provided, write it
        if let Some(ref content) = self.content {
            let write_tool = WriteFileTool;
            let mut args = ToolArgs::new();
            args.args.insert(
                "path".to_string(),
                serde_json::Value::String(self.file.clone()),
            );
            args.args.insert(
                "content".to_string(),
                serde_json::Value::String(content.clone()),
            );

            let result = write_tool.execute(args).await?;
            println!("{}", result.output);
        } else if self.ai {
            println!("AI-assisted editing not yet implemented");
            if let Some(ref prompt) = self.prompt {
                println!("Prompt: {}", prompt);
            }
        }

        Ok(())
    }
}
