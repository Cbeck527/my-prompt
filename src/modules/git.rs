use crate::module_trait::{GitBackend, Module, ModuleContext};
use bitflags::bitflags;
use git2::RepositoryOpenFlags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    struct GitStatus: u8 {
        const MODIFIED = 0b001;
        const UNTRACKED = 0b100;
    }
}

struct GitInfo {
    branch: String,
    status: GitStatus,
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

/// Parse the output of `git status --porcelain=v1 --branch`.
/// First line: "## branch" or "## branch...upstream [ahead N, behind M]"
/// Remaining lines: XY status entries.
fn parse_git_status_output(text: &str) -> Option<GitInfo> {
    let mut lines = text.lines();

    let branch = lines
        .next()
        .and_then(|line| line.strip_prefix("## "))
        .map(|rest| {
            // Strip tracking info after "..."
            rest.split("...").next().unwrap_or(rest).to_string()
        })?;

    let mut status = GitStatus::empty();
    for line in lines {
        if line.starts_with("??") {
            status |= GitStatus::UNTRACKED;
        } else if !line.is_empty() {
            // Porcelain v1 format: XY path
            // X = index (staged) status, Y = worktree status
            let chars: Vec<char> = line.chars().take(2).collect();
            if chars.len() >= 2
                && chars[0] != '?'
                && (chars[0] != ' ' || chars[1] != ' ')
            {
                status |= GitStatus::MODIFIED;
            }
        }

        if status.contains(GitStatus::MODIFIED | GitStatus::UNTRACKED) {
            break;
        }
    }

    Some(GitInfo { branch, status })
}

fn get_git_info_binary() -> Option<GitInfo> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain=v1", "--branch", "--untracked-files=normal"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_git_status_output(&String::from_utf8_lossy(&output.stdout))
}

fn get_git_info_gix() -> Option<GitInfo> {
    let repo = gix::discover(".").ok()?;

    // Get branch name
    let branch = if let Ok(head) = repo.head_ref() {
        if let Some(head_ref) = head {
            // We have a branch - get the short name
            head_ref.name().shorten().to_string()
        } else {
            // Detached HEAD - show abbreviated commit hash
            repo.head_id()
                .map_or_else(|_| "HEAD".to_string(), |id| id.shorten_or_id().to_string())
        }
    } else {
        "HEAD".to_string()
    };

    // Get status with optimizations
    let mut status = GitStatus::empty();
    let status_iter = match repo.status(gix::progress::Discard) {
        Ok(platform) => match platform
            .index_worktree_submodules(None) // Skip submodules for speed
            .index_worktree_rewrites(None) // Skip rename detection for speed
            .into_index_worktree_iter(Vec::new())
        {
            Ok(iter) => iter,
            Err(_) => return Some(GitInfo { branch, status }),
        },
        Err(_) => return Some(GitInfo { branch, status }),
    };

    for item in status_iter.filter_map(std::result::Result::ok) {
        match item {
            gix::status::index_worktree::Item::Modification { .. } => {
                status |= GitStatus::MODIFIED;
            }
            gix::status::index_worktree::Item::DirectoryContents {
                entry:
                    gix::dir::Entry {
                        status: gix::dir::entry::Status::Untracked,
                        disk_kind,
                        ..
                    },
                ..
            } if !matches!(disk_kind, Some(gix::dir::entry::Kind::Directory)) => {
                status |= GitStatus::UNTRACKED;
            }
            _ => {}
        }

        if status.contains(GitStatus::MODIFIED | GitStatus::UNTRACKED) {
            break;
        }
    }

    Some(GitInfo { branch, status })
}

fn get_git_info_git2() -> Option<GitInfo> {
    let repo = git2::Repository::open_ext(
        ".",
        RepositoryOpenFlags::CROSS_FS,
        &[] as &[&std::ffi::OsStr],
    )
    .ok()?;

    // Get branch name
    let head = repo.head().ok()?;
    let branch = head.shorthand().unwrap_or("HEAD").to_string();

    // Get status with optimized options
    let mut opts = git2::StatusOptions::new();
    opts.show(git2::StatusShow::IndexAndWorkdir)
        .exclude_submodules(true) // Skip submodules
        .include_untracked(true) // We need untracked files
        .include_unmodified(false); // Skip unchanged files

    let Ok(statuses) = repo.statuses(Some(&mut opts)) else {
        return Some(GitInfo {
            branch,
            status: GitStatus::empty(),
        });
    };

    let mut status = GitStatus::empty();
    for entry in statuses.iter() {
        let s = entry.status();
        if s.is_wt_modified()
            || s.is_wt_typechange()
            || s.is_index_modified()
            || s.is_index_new()
            || s.is_index_deleted()
            || s.is_index_renamed()
            || s.is_index_typechange()
        {
            status |= GitStatus::MODIFIED;
        }
        if s.is_wt_new() {
            status |= GitStatus::UNTRACKED;
        }
        if status.contains(GitStatus::MODIFIED | GitStatus::UNTRACKED) {
            break;
        }
    }

    Some(GitInfo { branch, status })
}

impl Module for GitModule {
    fn render(&self, context: &ModuleContext) -> crate::error::Result<Option<String>> {
        use crate::style::{AnsiStyle, Color};

        // Get git info using configured backend
        let Some(info) = (match context.git_backend {
            GitBackend::Binary => get_git_info_binary(),
            GitBackend::Gix => get_git_info_gix(),
            GitBackend::Git2 => get_git_info_git2(),
        }) else {
            return Ok(None);
        };

        let has_changes = info.status.contains(GitStatus::MODIFIED);
        let has_untracked = info.status.contains(GitStatus::UNTRACKED);

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
                Ok(Some(format!("[{}] ", info.branch)))
            } else {
                Ok(Some(format!("[{}{}] ", info.branch, indicators)))
            }
        } else {
            let blue = AnsiStyle::new(Color::Blue, false);
            let red = AnsiStyle::new(Color::Red, false);

            if indicators.is_empty() {
                Ok(Some(format!(
                    "{}[{}]{} ",
                    blue.start_codes(),
                    info.branch,
                    AnsiStyle::RESET
                )))
            } else {
                Ok(Some(format!(
                    "{}[{}{}{}{}]{} ",
                    blue.start_codes(),
                    info.branch,
                    red.start_codes(),
                    indicators,
                    blue.start_codes(),
                    AnsiStyle::RESET
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_git_outside_repo() {
        // Create temp dir that's not a git repo
        let temp = env::temp_dir();
        let original = env::current_dir().unwrap();

        env::set_current_dir(&temp).unwrap();

        let module = GitModule::new();
        let context = ModuleContext::default();

        let result = module.render(&context).unwrap();

        // Restore original directory
        env::set_current_dir(original).unwrap();

        // Should return None outside a git repo
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_clean_repo() {
        let output = "## main\n";
        let info = parse_git_status_output(output).unwrap();
        assert_eq!(info.branch, "main");
        assert!(info.status.is_empty());
    }

    #[test]
    fn test_parse_branch_with_tracking() {
        let output = "## trunk...origin/trunk [ahead 3]\n";
        let info = parse_git_status_output(output).unwrap();
        assert_eq!(info.branch, "trunk");
        assert!(info.status.is_empty());
    }

    #[test]
    fn test_parse_unstaged_modification() {
        let output = "## main\n M src/main.rs\n";
        let info = parse_git_status_output(output).unwrap();
        assert_eq!(info.branch, "main");
        assert!(info.status.contains(GitStatus::MODIFIED));
        assert!(!info.status.contains(GitStatus::UNTRACKED));
    }

    #[test]
    fn test_parse_staged_modification() {
        let output = "## main\nM  src/main.rs\n";
        let info = parse_git_status_output(output).unwrap();
        assert_eq!(info.branch, "main");
        assert!(info.status.contains(GitStatus::MODIFIED));
        assert!(!info.status.contains(GitStatus::UNTRACKED));
    }

    #[test]
    fn test_parse_staged_and_unstaged() {
        let output = "## main\nMM src/main.rs\n";
        let info = parse_git_status_output(output).unwrap();
        assert!(info.status.contains(GitStatus::MODIFIED));
    }

    #[test]
    fn test_parse_staged_new_file() {
        let output = "## main\nA  src/new.rs\n";
        let info = parse_git_status_output(output).unwrap();
        assert!(info.status.contains(GitStatus::MODIFIED));
    }

    #[test]
    fn test_parse_staged_delete() {
        let output = "## main\nD  src/old.rs\n";
        let info = parse_git_status_output(output).unwrap();
        assert!(info.status.contains(GitStatus::MODIFIED));
    }

    #[test]
    fn test_parse_untracked_files() {
        let output = "## main\n?? newfile.txt\n";
        let info = parse_git_status_output(output).unwrap();
        assert!(!info.status.contains(GitStatus::MODIFIED));
        assert!(info.status.contains(GitStatus::UNTRACKED));
    }

    #[test]
    fn test_parse_modified_and_untracked() {
        let output = "## feature\n M src/lib.rs\n?? TODO.md\n";
        let info = parse_git_status_output(output).unwrap();
        assert_eq!(info.branch, "feature");
        assert!(info.status.contains(GitStatus::MODIFIED));
        assert!(info.status.contains(GitStatus::UNTRACKED));
    }

    #[test]
    fn test_parse_no_branch_header() {
        let output = "not a valid header\n";
        assert!(parse_git_status_output(output).is_none());
    }

    #[test]
    fn test_parse_empty_output() {
        assert!(parse_git_status_output("").is_none());
    }
}
