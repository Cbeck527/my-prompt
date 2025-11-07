use crate::cache::{GIT_CACHE, GitInfo};
use crate::error::{PromptError, Result};
use crate::module_trait::{Module, ModuleContext};
use crate::modules::utils;
use bitflags::bitflags;
use std::path::PathBuf;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct GitStatus: u8 {
        const MODIFIED = 0b001;
        const UNTRACKED = 0b100;
    }
}

pub struct GitModule;

impl Default for GitModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GitModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[cold]
fn get_git_status_slow(repo_root: &PathBuf) -> GitStatus {
    let mut status = GitStatus::empty();

    // Only run git status if not cached
    if let Ok(output) = std::process::Command::new("git")
        .arg("status")
        .arg("--porcelain=v1")
        .arg("--untracked-files=normal")
        .current_dir(repo_root)
        .output()
        && output.status.success()
    {
        let status_text = String::from_utf8_lossy(&output.stdout);

        for line in status_text.lines() {
            if line.starts_with("??") {
                status |= GitStatus::UNTRACKED;
            } else if !line.is_empty() {
                let chars: Vec<char> = line.chars().take(2).collect();
                if chars.len() >= 2 && chars[1] != ' ' && chars[1] != '?' {
                    status |= GitStatus::MODIFIED;
                }
            }
        }
    }
    status
}

impl Module for GitModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        use crate::style::{AnsiStyle, Color};

        // Check if we're in a git repository
        let Some(git_dir) = utils::find_upward(".git") else {
            return Ok(None);
        };

        let Some(repo_root) = git_dir.parent() else {
            return Ok(None);
        };

        // Check cache first
        let (branch_name, has_changes, has_untracked) =
            if let Some(cached) = GIT_CACHE.get(repo_root) {
                (cached.branch, cached.has_changes, cached.has_untracked)
            } else {
                // Open repo to get branch and status
                let Ok(repo) = gix::open(repo_root) else {
                    return Ok(None);
                };

                // Get branch name efficiently
                let branch = if let Ok(Some(head_ref)) = repo.head_ref() {
                    String::from_utf8(head_ref.name().shorten().to_vec())
                        .unwrap_or_else(|_| "HEAD".to_string())
                } else if let Ok(Some(head_name)) = repo.head_name() {
                    String::from_utf8(head_name.shorten().to_vec())
                        .unwrap_or_else(|_| "HEAD".to_string())
                } else if let Ok(head) = repo.head() {
                    head.id()
                        .map_or_else(|| "HEAD".to_string(), |id| id.shorten_or_id().to_string())
                } else {
                    "HEAD".to_string()
                };

                // Get status
                let status = get_git_status_slow(&repo_root.to_path_buf());

                // Cache the result
                let info = GitInfo {
                    branch: branch.clone(),
                    has_changes: status.contains(GitStatus::MODIFIED),
                    has_untracked: status.contains(GitStatus::UNTRACKED),
                };
                GIT_CACHE.insert(repo_root.to_path_buf(), info.clone());

                (info.branch, info.has_changes, info.has_untracked)
            };

        // Build status indicators
        let mut indicators = String::new();
        if has_changes {
            indicators.push('+');
        }
        if has_untracked {
            indicators.push('?');
        }

        if context.no_color {
            if indicators.is_empty() {
                Ok(Some(format!("[{}] ", branch_name)))
            } else {
                Ok(Some(format!("[{}{}] ", branch_name, indicators)))
            }
        } else {
            let blue = AnsiStyle::new(Color::Blue, false);
            let red = AnsiStyle::new(Color::Red, false);

            if indicators.is_empty() {
                Ok(Some(format!(
                    "{}[{}]{} ",
                    blue.start_codes(),
                    branch_name,
                    AnsiStyle::RESET
                )))
            } else {
                Ok(Some(format!(
                    "{}[{}{}{}{}]{} ",
                    blue.start_codes(),
                    branch_name,
                    red.start_codes(),
                    indicators,
                    blue.start_codes(),
                    AnsiStyle::RESET
                )))
            }
        }
    }
}
