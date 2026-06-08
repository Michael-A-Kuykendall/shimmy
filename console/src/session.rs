use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use anyhow::Result;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub created: String,
    pub updated: String,
    pub messages: Vec<serde_json::Value>,  // OpenAI message objects
}

impl Session {
    pub fn new() -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            created: now.clone(),
            updated: now,
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, message: serde_json::Value) {
        self.updated = Utc::now().to_rfc3339();
        self.messages.push(message);
    }

    pub fn save(&self, session_dir: &Path) -> Result<PathBuf> {
        std::fs::create_dir_all(session_dir)?;
        let path = session_dir.join(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(path)
    }

    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn load_by_id(session_dir: &Path, id: &str) -> Result<Self> {
        let path = session_dir.join(format!("{}.json", id));
        Self::load(&path)
    }

    pub fn list(session_dir: &Path) -> Vec<(String, String)> {
        // Returns vec of (id, updated) sorted by most recent first
        if !session_dir.exists() { return Vec::new(); }
        let mut sessions: Vec<(String, String)> = std::fs::read_dir(session_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
            .filter_map(|e| {
                let text = std::fs::read_to_string(e.path()).ok()?;
                let s: Session = serde_json::from_str(&text).ok()?;
                Some((s.id, s.updated))
            })
            .collect();
        sessions.sort_by(|a, b| b.1.cmp(&a.1));
        sessions
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
