use crate::context::Context;
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use uuid::Uuid;

/// Maximum history entries to keep per session
const MAX_HISTORY_SIZE: usize = 100;

/// A single command entry in session history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    /// Unique ID for this entry
    pub id: String,
    /// Timestamp of the command
    pub timestamp: DateTime<Utc>,
    /// User request (natural language)
    pub request: String,
    /// Generated command(s)
    pub commands: Vec<String>,
    /// Whether user executed the command
    pub executed: bool,
    /// Working directory at time of command
    pub working_dir: PathBuf,
}

impl CommandEntry {
    pub fn new(request: String, commands: Vec<String>, working_dir: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            request,
            commands,
            executed: false,
            working_dir,
        }
    }
}

/// Session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// Session name (optional)
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub updated_at: DateTime<Utc>,
    /// Command history
    pub history: VecDeque<CommandEntry>,
    /// Current context
    pub context: Option<Context>,
    /// Agent name being used
    pub agent_name: String,
}

impl Session {
    /// Create a new session
    pub fn new(name: Option<String>, agent_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_at: now,
            updated_at: now,
            history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            context: None,
            agent_name,
        }
    }

    /// Add a command to history
    pub fn add_command(&mut self, entry: CommandEntry) {
        // Remove oldest if at capacity
        if self.history.len() >= MAX_HISTORY_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(entry);
        self.updated_at = Utc::now();
    }

    /// Get recent commands
    pub fn recent_commands(&self, limit: usize) -> Vec<&CommandEntry> {
        self.history.iter().rev().take(limit).collect()
    }

    /// Get context for next command (includes recent history)
    pub fn build_context(&self) -> String {
        let mut context = String::new();

        // Add recent commands for context
        let recent = self.recent_commands(5);
        if !recent.is_empty() {
            context.push_str("Recent commands:\n");
            for entry in recent.iter().rev() {
                context.push_str(&format!("- User: {}\n", entry.request));
                if !entry.commands.is_empty() {
                    context.push_str(&format!("  Command: {}\n", entry.commands.join("; ")));
                }
            }
            context.push('\n');
        }

        context
    }

    /// Update context
    pub fn update_context(&mut self, context: Context) {
        self.context = Some(context);
        self.updated_at = Utc::now();
    }
}

/// Session manager
pub struct SessionManager {
    /// Sessions directory
    sessions_dir: PathBuf,
    /// Current session (if any)
    current_session: Option<Session>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Result<Self> {
        let sessions_dir = dirs::home_dir()
            .ok_or_else(|| Error::Parse("Cannot determine home directory".to_string()))?
            .join(".text2cli")
            .join("sessions");

        // Ensure directory exists
        std::fs::create_dir_all(&sessions_dir)?;

        Ok(Self {
            sessions_dir,
            current_session: None,
        })
    }

    /// Get session file path
    fn session_path(&self, session_id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{}.json", session_id))
    }

    /// Create a new session
    pub fn create(&mut self, name: Option<String>, agent_name: String) -> Result<&Session> {
        let session = Session::new(name, agent_name);
        let path = self.session_path(&session.id);

        // Save to disk
        let json = serde_json::to_string_pretty(&session)?;
        std::fs::write(&path, json)?;

        self.current_session = Some(session);
        Ok(self.current_session.as_ref().unwrap())
    }

    /// Load an existing session
    pub fn load(&mut self, session_id: &str) -> Result<&Session> {
        let path = self.session_path(session_id);

        if !path.exists() {
            return Err(Error::Parse(format!("Session '{}' not found", session_id)));
        }

        let json = std::fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&json)?;

        self.current_session = Some(session);
        Ok(self.current_session.as_ref().unwrap())
    }

    /// Get current session
    pub fn current(&self) -> Option<&Session> {
        self.current_session.as_ref()
    }

    /// Get current session mutably
    pub fn current_mut(&mut self) -> Option<&mut Session> {
        self.current_session.as_mut()
    }

    /// Save current session to disk
    pub fn save(&self) -> Result<()> {
        if let Some(session) = &self.current_session {
            let path = self.session_path(&session.id);
            let json = serde_json::to_string_pretty(session)?;
            std::fs::write(&path, json)?;
        }
        Ok(())
    }

    /// List all sessions
    pub fn list(&self) -> Result<Vec<SessionInfo>> {
        let mut sessions = Vec::new();

        for entry in std::fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(json) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&json) {
                        sessions.push(SessionInfo {
                            id: session.id,
                            name: session.name,
                            created_at: session.created_at,
                            updated_at: session.updated_at,
                            command_count: session.history.len(),
                            agent_name: session.agent_name,
                        });
                    }
                }
            }
        }

        // Sort by updated_at descending
        sessions.sort_by_key(|b| std::cmp::Reverse(b.updated_at));

        Ok(sessions)
    }

    /// Delete a session
    pub fn delete(&mut self, session_id: &str) -> Result<()> {
        let path = self.session_path(session_id);

        if !path.exists() {
            return Err(Error::Parse(format!("Session '{}' not found", session_id)));
        }

        std::fs::remove_file(&path)?;

        // Clear current if it was this session
        if let Some(current) = &self.current_session {
            if current.id == session_id {
                self.current_session = None;
            }
        }

        Ok(())
    }

    /// End current session
    pub fn end(&mut self) {
        self.current_session = None;
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SessionManager")
    }
}

/// Brief session info for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub command_count: usize,
    pub agent_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(Some("test".to_string()), "claude-code".to_string());
        assert!(session.name.is_some());
        assert_eq!(session.agent_name, "claude-code");
        assert!(session.history.is_empty());
    }

    #[test]
    fn test_add_command() {
        let mut session = Session::new(None, "claude-code".to_string());
        let entry = CommandEntry::new(
            "list files".to_string(),
            vec!["ls -la".to_string()],
            PathBuf::from("/tmp"),
        );

        session.add_command(entry);
        assert_eq!(session.history.len(), 1);
    }

    #[test]
    fn test_history_limit() {
        let mut session = Session::new(None, "claude-code".to_string());

        for i in 0..150 {
            let entry = CommandEntry::new(
                format!("command {}", i),
                vec![format!("cmd{}", i)],
                PathBuf::from("/tmp"),
            );
            session.add_command(entry);
        }

        assert_eq!(session.history.len(), MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_recent_commands() {
        let mut session = Session::new(None, "claude-code".to_string());

        for i in 0..10 {
            let entry = CommandEntry::new(
                format!("command {}", i),
                vec![format!("cmd{}", i)],
                PathBuf::from("/tmp"),
            );
            session.add_command(entry);
        }

        let recent = session.recent_commands(3);
        assert_eq!(recent.len(), 3);
    }
}
