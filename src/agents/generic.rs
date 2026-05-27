use super::AgentAdapter;
use crate::{Error, Result};

pub struct GenericAdapter {
    name: String,
    command: String,
}

impl GenericAdapter {
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
        }
    }
}

impl AgentAdapter for GenericAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn parse_output(&self, output: &str) -> Result<Vec<String>> {
        // Simple first-line extraction
        let command = output
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("//"));

        match command {
            Some(cmd) => Ok(vec![cmd.to_string()]),
            None => Err(Error::NoCommandReturned),
        }
    }
}
