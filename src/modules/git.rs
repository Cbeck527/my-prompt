use crate::cache::{GIT_CACHE, GitInfo};
use crate::error::{PromptError, Result};
use crate::module_trait::{Module, ModuleContext};
use crate::modules::utils;
use crate::style::{AnsiStyle, ModuleStyle};
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

fn parse_git_format(format: &str) -> (&str, Option<String>) {
    if let Some((base_fmt, rest)) = format.split_once(',') {
        if let Some(status_style) = rest.strip_prefix("status=") {
            return (base_fmt, Some(status_style.to_string()));
        }
    }
    (format, None)
}

fn validate_git_format(format: &str) -> Result<&str> {
    match format {
        "" | "full" | "f" => Ok("full"),
        "short" | "s" => Ok("short"),
        _ => Err(PromptError::InvalidFormat {
            module: "git".to_string(),
            format: format.to_string(),
            valid_formats: "full, f, short, s".to_string(),
        }),
    }
}

impl Module for GitModule {
    fn render(&self, format: &str, _context: &ModuleContext) -> Result<Option<String>> {
        // Parse format to extract base format and optional status color
        let (base_format, status_color) = parse_git_format(format);

        // Validate format
        let normalized_format = validate_git_format(base_format)?;

        // Fast path: find git directory
        let Some(git_dir) = utils::find_upward(".git") else {
            return Ok(None);
        };
        let repo_root = match git_dir.parent() {
            Some(p) => p.to_path_buf(),
            None => return Ok(None),
        };

        // Check cache first
        if let Some(cached) = GIT_CACHE.get(&repo_root) {
            return Ok(match normalized_format {
                "full" => {
                    let mut result = cached.branch.clone();
                    if cached.has_changes || cached.has_untracked {
                        // Build status indicators
                        let mut indicators = String::new();
                        if cached.has_changes {
                            indicators.push('+');
                        }
                        if cached.has_untracked {
                            indicators.push('?');
                        }

                        // Apply color if specified
                        if let Some(ref status_color) = status_color {
                            let style = AnsiStyle::parse(status_color).map_err(|e| {
                                PromptError::StyleError {
                                    module: "git".to_string(),
                                    error: e,
                                }
                            })?;
                            // Write red start codes, indicators, then reset
                            result.push_str(&style.apply(&indicators));
                            // Restore outer blue color for suffix
                            result.push_str("\x1b[34m");
                        } else {
                            result.push_str(&indicators);
                        }
                    }
                    Some(result)
                }
                "short" => Some(cached.branch),
                _ => unreachable!("validate_git_format should have caught this"),
            });
        }

        // Open repo with minimal operations
        let Ok(repo) = gix::open(&repo_root) else {
            return Ok(None);
        };

        // Get branch name efficiently
        let branch_name = if let Ok(Some(head_ref)) = repo.head_ref() {
            String::from_utf8(head_ref.name().shorten().to_vec())
                .unwrap_or_else(|_| "HEAD".to_string())
        } else if let Ok(Some(head_name)) = repo.head_name() {
            String::from_utf8(head_name.shorten().to_vec()).unwrap_or_else(|_| "HEAD".to_string())
        } else if let Ok(head) = repo.head() {
            head.id()
                .map_or_else(|| "HEAD".to_string(), |id| id.shorten_or_id().to_string())
        } else {
            "HEAD".to_string()
        };

        // Get status info based on format
        let status = match normalized_format {
            "full" => get_git_status_slow(&repo_root),
            "short" => GitStatus::empty(),
            _ => unreachable!("validate_git_format should have caught this"),
        };

        // Cache the result
        let info = GitInfo {
            branch: branch_name.clone(),
            has_changes: status.contains(GitStatus::MODIFIED),
            has_untracked: status.contains(GitStatus::UNTRACKED),
        };
        GIT_CACHE.insert(repo_root, info);

        // Build result
        Ok(match normalized_format {
            "full" => {
                let mut result = branch_name;
                if !status.is_empty() {
                    // Build status indicators
                    let mut indicators = String::new();
                    if status.contains(GitStatus::MODIFIED) {
                        indicators.push('+');
                    }
                    if status.contains(GitStatus::UNTRACKED) {
                        indicators.push('?');
                    }

                    // Apply color if specified
                    if let Some(ref status_color) = status_color {
                        let style = AnsiStyle::parse(status_color).map_err(|e| {
                            PromptError::StyleError {
                                module: "git".to_string(),
                                error: e,
                            }
                        })?;
                        // Write red start codes, indicators, then reset
                        result.push_str(&style.apply(&indicators));
                        // HACK: restore outer blue color for suffix
                        result.push_str("\x1b[34m");
                    } else {
                        result.push_str(&indicators);
                    }
                }
                Some(result)
            }
            "short" => Some(branch_name),
            _ => unreachable!("validate_git_format should have caught this"),
        })
    }
}
