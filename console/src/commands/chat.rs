//! Chat command

use clap::Args;

/// Arguments for the chat command
#[derive(Debug, Args)]
pub struct ChatCommand {
    /// Model to use for chat
    #[arg(short, long)]
    pub model: Option<String>,

    /// Initial message to send
    pub message: Option<String>,

    /// Session ID to continue
    #[arg(short, long)]
    pub session: Option<String>,

    /// Enable streaming responses
    #[arg(long, default_value = "true")]
    pub stream: bool,
}

impl ChatCommand {
    /// Execute the chat command
    pub async fn run(&self) -> anyhow::Result<()> {
        println!("Chat command not yet implemented");
        if let Some(ref msg) = self.message {
            println!("Message: {}", msg);
        }
        if let Some(ref model) = self.model {
            println!("Model: {}", model);
        }
        Ok(())
    }
}
