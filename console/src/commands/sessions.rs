// DEPRECATED: This module was for shimmy-session-store integration (archived plan)
// Real plan (BRANCH_001_COMPLETE_AUDIT.md) does NOT use session/context crates
// Console is stateless tool executor - no persistent sessions needed
//
// Archived plans in: .internal/archive/shimmy-crates-plans/
// - CONSOLE_COMPLETION_PLAN_WRONG.md
// - WIRING_AND_INTEGRATION_TASKS.md
// - COMPREHENSIVE_AUDIT_REPORT.md

use anyhow::Result;

#[derive(clap::Subcommand, Debug)]
pub enum SessionCommand {
    /// List all sessions
    List {
        #[arg(long)]
        json: bool,
    },
    /// Create a new session
    Create {
        name: String,
        #[arg(long)]
        overwrite: bool,
    },
    /// Delete a session
    Delete {
        name: String,
        #[arg(long)]
        yes: bool,
    },
    /// Show session context information
    Context {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Trim session data
    Trim {
        name: String,
        #[arg(long, value_enum)]
        policy: Option<TrimPolicy>,
        #[arg(long)]
        target_tokens: Option<usize>,
    },
    /// Search session history
    Search {
        name: String,
        query: String,
        #[arg(long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum TrimPolicy {
    Keep,
    Summarize,
    Compact,
}

pub async fn execute_session_command(_session_cmd: SessionCommand) -> Result<()> {
    anyhow::bail!(
        "Session commands not implemented.\n\
        Console is stateless per BRANCH_001_COMPLETE_AUDIT.md spec.\n\
        See .internal/archive/shimmy-crates-plans/ for archived session/context crate plans."
    )
}
