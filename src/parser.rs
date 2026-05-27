use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsePosition {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub content: String,
    pub position: ParsePosition,
    pub raw_input: String,
}

pub struct TriggerParser {
    trigger: String,
}

impl TriggerParser {
    pub fn new(trigger: impl Into<String>) -> Self {
        Self {
            trigger: trigger.into(),
        }
    }

    pub fn parse(&self, input: &str) -> Result<Option<ParsedCommand>> {
        let trimmed = input.trim();

        // Check for empty input
        if trimmed.is_empty() {
            return Ok(None);
        }

        // Try prefix match
        if let Some(content) = self.try_prefix(trimmed) {
            if content.is_empty() {
                return Ok(None);
            }
            return Ok(Some(ParsedCommand {
                content: content.to_string(),
                position: ParsePosition::Prefix,
                raw_input: input.to_string(),
            }));
        }

        // Try suffix match
        if let Some(content) = self.try_suffix(trimmed) {
            if content.is_empty() {
                return Ok(None);
            }
            return Ok(Some(ParsedCommand {
                content: content.to_string(),
                position: ParsePosition::Suffix,
                raw_input: input.to_string(),
            }));
        }

        Ok(None)
    }

    fn try_prefix<'a>(&self, input: &'a str) -> Option<&'a str> {
        if input.starts_with(&self.trigger) {
            let rest = input[self.trigger.len()..].trim();
            Some(rest)
        } else {
            None
        }
    }

    fn try_suffix<'a>(&self, input: &'a str) -> Option<&'a str> {
        if input.ends_with(&self.trigger) {
            let rest = input[..input.len() - self.trigger.len()].trim();
            Some(rest)
        } else {
            None
        }
    }
}
