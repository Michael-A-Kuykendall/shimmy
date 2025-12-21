// History storage using SQLite with WAL mode - fast embedded database
// Provides persistent message history across sessions with ACID guarantees
// WAL mode eliminates all lock file issues and enables concurrent reads

use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;

// SQL schema for history database
const SCHEMA_SQL: &str = r#"
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS history (
    session_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    message TEXT NOT NULL,
    PRIMARY KEY (session_id, timestamp)
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_session_recent 
    ON history(session_id, timestamp DESC);

CREATE TABLE IF NOT EXISTS session_meta (
    session_id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    last_active TEXT NOT NULL,
    message_count INTEGER NOT NULL
) WITHOUT ROWID;
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: String,          // "user", "assistant", "system", "tool"
    pub content: String,
    pub timestamp: String,     // ISO 8601
    pub tool_calls: Option<Vec<String>>,
    pub tool_results: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub created_at: String,
    pub last_active: String,
    pub message_count: usize,
}

/// SQLite-backed history storage with WAL mode
/// 
/// Features:
/// - Persistent across restarts
/// - Fast writes with WAL mode
/// - Concurrent reads (unlimited readers)
/// - ACID transactions
/// - No lock file issues
/// - Zero lock cleanup code needed
pub struct HistoryStorage {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl HistoryStorage {
    /// Create or open history database
    /// 
    /// Default location: ~/.config/shimmy/console/history.db (Linux/macOS)
    ///                   %APPDATA%\shimmy\console\history.db (Windows)
    /// 
    /// WAL mode is automatically enabled - no lock cleanup needed.
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let db_path = db_path.unwrap_or_else(|| {
            let mut path = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."));
            path.push("shimmy");
            path.push("console");
            std::fs::create_dir_all(&path).ok();
            path.push("history.db");
            path
        });

        // Open SQLite connection with WAL mode
        let conn = Connection::open(&db_path)
            .context("Failed to open history database")?;
        
        // Initialize WAL mode and schema
        conn.execute_batch(SCHEMA_SQL)
            .context("Failed to initialize database schema")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// Store a message in history
    pub fn store_message(&self, session_id: &str, message: &HistoryMessage) -> Result<()> {
        let conn = self.conn.lock();

        // Generate timestamp key (microseconds for uniqueness)
        let timestamp_micros = chrono::Utc::now().timestamp_micros() as u64;
        
        // Serialize message
        let message_json = serde_json::to_string(message)?;

        // Store message
        conn.execute(
            "INSERT INTO history (session_id, timestamp, message) VALUES (?1, ?2, ?3)",
            params![session_id, timestamp_micros, message_json],
        )?;

        // Update session metadata
        self.update_session_meta_internal(&conn, session_id)?;

        Ok(())
    }

    /// Retrieve last N messages for a session
    pub fn get_history(&self, session_id: &str, limit: usize) -> Result<Vec<HistoryMessage>> {
        let conn = self.conn.lock();
        
        let mut stmt = conn.prepare(
            "SELECT message FROM history WHERE session_id = ?1 ORDER BY timestamp DESC LIMIT ?2"
        )?;
        
        let messages = stmt.query_map(params![session_id, limit as i64], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?
        .filter_map(|result| {
            result.ok().and_then(|json| serde_json::from_str(&json).ok())
        })
        .collect();

        Ok(messages)
    }

    /// Get total message count for a session
    pub fn get_message_count(&self, session_id: &str) -> Result<usize> {
        let conn = self.conn.lock();
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        
        Ok(count as usize)
    }

    /// List all sessions with metadata
    pub fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
        let conn = self.conn.lock();
        
        let mut stmt = conn.prepare("SELECT session_id, created_at, last_active, message_count FROM session_meta")?;
        
        let sessions = stmt.query_map([], |row| {
            Ok(SessionMetadata {
                session_id: row.get(0)?,
                created_at: row.get(1)?,
                last_active: row.get(2)?,
                message_count: row.get(3)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();

        Ok(sessions)
    }

    /// Clear history for a specific session
    pub fn clear_session(&self, session_id: &str) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute("DELETE FROM history WHERE session_id = ?1", params![session_id])?;
        conn.execute("DELETE FROM session_meta WHERE session_id = ?1", params![session_id])?;

        Ok(())
    }

    /// Trim old messages (keep only last N messages per session)
    pub fn trim_session(&self, session_id: &str, keep_count: usize) -> Result<usize> {
        let conn = self.conn.lock();

        // Get total count
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        if total as usize <= keep_count {
            return Ok(0);
        }

        let delete_count = total as usize - keep_count;

        // Delete oldest messages (keep only the most recent keep_count)
        conn.execute(
            "DELETE FROM history WHERE session_id = ?1 AND timestamp NOT IN (
                SELECT timestamp FROM history WHERE session_id = ?1 
                ORDER BY timestamp DESC LIMIT ?2
            )",
            params![session_id, keep_count as i64],
        )?;

        // Update session metadata
        self.update_session_meta_internal(&conn, session_id)?;

        Ok(delete_count)
    }

    /// Get database file path
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Get database size in bytes (including WAL files)
    pub fn db_size(&self) -> Result<u64> {
        let mut size = std::fs::metadata(&self.db_path)?.len();
        
        // Add WAL file size if it exists
        let wal_path = self.db_path.with_extension("db-wal");
        if let Ok(wal_meta) = std::fs::metadata(&wal_path) {
            size += wal_meta.len();
        }
        
        // Add SHM file size if it exists
        let shm_path = self.db_path.with_extension("db-shm");
        if let Ok(shm_meta) = std::fs::metadata(&shm_path) {
            size += shm_meta.len();
        }
        
        Ok(size)
    }

    // Internal helper to update session metadata
    fn update_session_meta_internal(&self, conn: &Connection, session_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        
        // Get current count
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        // Try to load existing metadata
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM session_meta WHERE session_id = ?1)",
            params![session_id],
            |row| row.get(0),
        )?;

        if exists {
            conn.execute(
                "UPDATE session_meta SET last_active = ?1, message_count = ?2 WHERE session_id = ?3",
                params![now, count, session_id],
            )?;
        } else {
            conn.execute(
                "INSERT INTO session_meta (session_id, created_at, last_active, message_count) VALUES (?1, ?2, ?3, ?4)",
                params![session_id, now.clone(), now, count],
            )?;
        }

        Ok(())
    }
}

// Thread-safe implementation
unsafe impl Send for HistoryStorage {}
unsafe impl Sync for HistoryStorage {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_history_storage_crud() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let storage = HistoryStorage::new(Some(db_path)).unwrap();

        // Store message
        let msg = HistoryMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            tool_calls: None,
            tool_results: None,
        };

        storage.store_message("session1", &msg).unwrap();

        // Retrieve
        let history = storage.get_history("session1", 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "Hello");

        // Count
        let count = storage.get_message_count("session1").unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_trim_history() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let storage = HistoryStorage::new(Some(db_path)).unwrap();

        // Store 5 messages
        for i in 0..5 {
            let msg = HistoryMessage {
                role: "user".to_string(),
                content: format!("Message {}", i),
                timestamp: chrono::Utc::now().to_rfc3339(),
                tool_calls: None,
                tool_results: None,
            };
            storage.store_message("session1", &msg).unwrap();
            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_micros(100));
        }

        // Trim to keep only 2
        let deleted = storage.trim_session("session1", 2).unwrap();
        assert_eq!(deleted, 3);

        // Verify only 2 remain
        let history = storage.get_history("session1", 10).unwrap();
        assert_eq!(history.len(), 2);
    }
}
