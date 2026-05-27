pub mod agents;
pub mod config;
pub mod context;
pub mod error;
pub mod executor;
pub mod parser;
pub mod shell;

pub use agents::{AgentAdapter, AgentRegistry, ClaudeAdapter, CodexAdapter, GenericAdapter};
pub use config::{AgentConfig, Config, ConfigLoader, DEFAULT_AGENT, DEFAULT_TRIGGER};
pub use context::{Context, ContextCollector, GitStatus};
pub use error::{Error, Result};
pub use executor::AgentExecutor;
pub use parser::{ParsePosition, ParsedCommand, TriggerParser};
pub use shell::{BashHook, PwshHook, ShellHook, ZshHook};
