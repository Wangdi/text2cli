use crate::Result;
use std::io::{self, Write};

/// Command selector for multi-command scenarios
pub struct CommandSelector {
    commands: Vec<String>,
}

impl CommandSelector {
    /// Create a new selector with commands
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }

    /// Display commands and let user select
    pub fn select(&self) -> Result<Option<String>> {
        if self.commands.is_empty() {
            return Ok(None);
        }

        if self.commands.len() == 1 {
            return Ok(Some(self.commands[0].clone()));
        }

        // Display commands
        println!("\nMultiple commands suggested:");
        for (i, cmd) in self.commands.iter().enumerate() {
            println!("  [{}] {}", i + 1, cmd);
        }
        println!("  [a] Execute all");
        println!("  [q] Quit without executing");

        // Get user input
        print!("\nSelect command (1-{} / a / q): ", self.commands.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();

        match input.as_str() {
            "q" | "quit" | "exit" | "n" | "no" => Ok(None),
            "a" | "all" => {
                // Return all commands joined
                Ok(Some(self.commands.join(" && ")))
            }
            _ => {
                // Try to parse as number
                if let Ok(num) = input.parse::<usize>() {
                    if num >= 1 && num <= self.commands.len() {
                        return Ok(Some(self.commands[num - 1].clone()));
                    }
                }
                // Invalid input, return first command
                println!("Invalid selection, using first command.");
                Ok(Some(self.commands[0].clone()))
            }
        }
    }

    /// Non-interactive: return first command
    pub fn first(&self) -> Option<String> {
        self.commands.first().cloned()
    }

    /// Non-interactive: return all commands joined
    pub fn all(&self) -> Option<String> {
        if self.commands.is_empty() {
            None
        } else {
            Some(self.commands.join(" && "))
        }
    }

    /// Preview commands without selection
    pub fn preview(&self) {
        if self.commands.is_empty() {
            println!("No commands available.");
            return;
        }

        println!("Commands:");
        for (i, cmd) in self.commands.iter().enumerate() {
            println!("  {}. {}", i + 1, cmd);
        }
    }
}

/// Interactive prompt for confirmation
pub fn confirm(prompt: &str) -> Result<bool> {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_single_command() {
        let selector = CommandSelector::new(vec!["ls -la".to_string()]);
        assert_eq!(selector.first(), Some("ls -la".to_string()));
    }

    #[test]
    fn test_selector_empty() {
        let selector = CommandSelector::new(vec![]);
        assert_eq!(selector.first(), None);
        assert_eq!(selector.all(), None);
    }

    #[test]
    fn test_selector_all() {
        let selector = CommandSelector::new(vec![
            "echo hello".to_string(),
            "echo world".to_string(),
        ]);
        assert_eq!(selector.all(), Some("echo hello && echo world".to_string()));
    }
}
