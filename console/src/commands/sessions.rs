use anyhow::Result;
use crate::session::Session;
use std::path::Path;

pub async fn execute_sessions(_session_dir: &Path) -> Result<()> {
    // This is called as a placeholder; see list_sessions, show_session below
    println!("Use: shimmy sessions list | show <id> | delete <id>");
    Ok(())
}

pub async fn list_sessions(session_dir: &Path) -> Result<()> {
    let sessions = Session::list(session_dir);
    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }
    println!("{} session(s):", sessions.len());
    for (id, updated) in sessions {
        println!("  {} ({})", id, updated);
    }
    Ok(())
}

pub async fn show_session(session_dir: &Path, id: &str) -> Result<()> {
    let session = Session::load_by_id(session_dir, id)?;
    println!("Session: {} (created: {})", session.id, session.created);
    println!("{} messages:", session.messages.len());
    for msg in &session.messages {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("?");
        let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("(tool result)");
        let preview = if content.len() > 120 { &content[..120] } else { content };
        println!("  [{}] {}", role, preview);
    }
    Ok(())
}

pub async fn delete_session(session_dir: &Path, id: &str) -> Result<()> {
    let path = session_dir.join(format!("{}.json", id));
    if !path.exists() {
        anyhow::bail!("Session not found: {}", id);
    }
    std::fs::remove_file(&path)?;
    println!("Deleted session {}", id);
    Ok(())
}
