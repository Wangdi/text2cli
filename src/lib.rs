pub mod agents;
pub mod config;
pub mod context;
pub mod error;
pub mod parser;

pub use agents::{AgentAdapter, AgentRegistry, ClaudeAdapter, CodexAdapter, GenericAdapter};
pub use config::{AgentConfig, Config, ConfigLoader, DEFAULT_AGENT, DEFAULT_TRIGGER};
pub use context::{Context, ContextCollector, GitStatus};
pub use error::{Error, Result};
pub use parser::{ParsePosition, ParsedCommand, TriggerParser};
