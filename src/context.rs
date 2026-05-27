use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub untracked: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub working_dir: PathBuf,
    pub git_branch: Option<String>,
    pub git_status: Option<GitStatus>,
    pub recent_files: Vec<PathBuf>,
    pub shell_env: HashMap<String, String>,
}

pub struct ContextCollector;

impl ContextCollector {
    pub fn collect() -> Result<Context> {
        let working_dir = std::env::current_dir()
            .map_err(|e| Error::Parse(format!("Cannot get current directory: {}", e)))?;

        let git_branch = Self::get_git_branch(&working_dir);
        let git_status = Self::get_git_status(&working_dir)?;
        let recent_files = Self::get_recent_files(&working_dir);
        let shell_env = Self::get_shell_env();

        Ok(Context {
            working_dir,
            git_branch,
            git_status,
            recent_files,
            shell_env,
        })
    }

    fn get_git_branch(dir: &PathBuf) -> Option<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(dir)
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() {
                return Some(branch);
            }
        }
        None
    }

    fn get_git_status(dir: &PathBuf) -> Result<Option<GitStatus>> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(dir)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let status_str = String::from_utf8_lossy(&output.stdout);
                let mut status = GitStatus::default();

                for line in status_str.lines() {
                    let prefix: String = line.chars().take(2).collect();
                    match prefix.trim() {
                        "M" | "MM" | " M" => status.modified += 1,
                        "A" | "AM" | " A" => status.added += 1,
                        "D" | "AD" | " D" => status.deleted += 1,
                        "??" => status.untracked += 1,
                        _ => {}
                    }
                }

                Ok(Some(status))
            }
            _ => Ok(None),
        }
    }

    fn get_recent_files(_dir: &PathBuf) -> Vec<PathBuf> {
        // TODO: Implement file watching or history tracking
        Vec::new()
    }

    fn get_shell_env() -> HashMap<String, String> {
        std::env::vars()
            .filter(|(k, _)| k.starts_with("SHELL") || k.starts_with("TERM"))
            .collect()
    }
}
