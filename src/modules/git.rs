use crate::module_trait::{GitBackend, Module, ModuleContext};
use crate::modules::utils::sanitize_display_text;
use bitflags::bitflags;
use std::path::Path;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct GitStatus: u8 {
        const MODIFIED = 0b001;
        const UNTRACKED = 0b100;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitInfo {
    branch: String,
    status: GitStatus,
}

pub(crate) struct GitModule;

impl Default for GitModule {
    fn default() -> Self {
        Self::new()
    }
}

impl GitModule {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self
    }
}

fn parse_branch_header(header: &str) -> Option<String> {
    let branch = header.strip_prefix("## ")?;
    let branch = branch.strip_prefix("No commits yet on ").unwrap_or(branch);
    let branch = branch.strip_prefix("Initial commit on ").unwrap_or(branch);

    if branch.starts_with("HEAD (") {
        return Some("HEAD".to_owned());
    }

    let branch = branch
        .split_once("...")
        .map_or(branch, |(branch, _tracking)| branch);

    (!branch.is_empty()).then(|| branch.to_owned())
}

fn parse_git_status_output(text: &str) -> Option<GitInfo> {
    let mut lines = text.lines();
    let branch = parse_branch_header(lines.next()?)?;

    let mut status = GitStatus::empty();
    for line in lines {
        let status_code = line.as_bytes().get(..2)?;
        if line.as_bytes().get(2) != Some(&b' ') {
            return None;
        }

        if status_code == b"??" {
            status |= GitStatus::UNTRACKED;
        } else if status_code != b"  " {
            status |= GitStatus::MODIFIED;
        }

        if status.contains(GitStatus::MODIFIED | GitStatus::UNTRACKED) {
            break;
        }
    }

    Some(GitInfo { branch, status })
}

fn get_git_info_binary(current_dir: &Path) -> Option<GitInfo> {
    let output = std::process::Command::new("git")
        .args([
            "status",
            "--porcelain=v1",
            "--branch",
            "--untracked-files=normal",
            "--ignore-submodules=dirty",
            "--no-renames",
        ])
        .current_dir(current_dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_git_status_output(&String::from_utf8_lossy(&output.stdout))
}

fn get_git_info_gix(current_dir: &Path) -> Option<GitInfo> {
    let repo = gix::discover(current_dir).ok()?;
    let head = repo.head().ok()?;
    let branch = head
        .referent_name()
        .map_or_else(|| "HEAD".to_owned(), |name| name.shorten().to_string());

    // gix-index 0.53 assumes the mapped index can contain an object ID checksum.
    let index_is_long_enough = match repo.index_path().metadata() {
        Ok(metadata) => {
            let checksum_length = u64::try_from(repo.object_hash().len_in_bytes()).ok()?;
            metadata.len() >= checksum_length
        }
        Err(error) => error.kind() == std::io::ErrorKind::NotFound,
    };
    if !index_is_long_enough {
        return None;
    }

    let dirwalk_options = repo.dirwalk_options().ok()?;

    let mut status = GitStatus::empty();
    let status_iter = repo
        .status(gix::progress::Discard)
        .ok()?
        .index_worktree_options_mut(|options| {
            options.dirwalk_options = Some(dirwalk_options);
        })
        .untracked_files(gix::status::UntrackedFiles::Collapsed)
        .index_worktree_submodules(gix::status::Submodule::Given {
            ignore: gix::submodule::config::Ignore::Dirty,
            check_dirty: true,
        })
        .index_worktree_rewrites(None)
        .tree_index_track_renames(gix::status::tree_index::TrackRenames::Disabled)
        .into_iter(Vec::new())
        .ok()?;

    for item in status_iter {
        let item = item.ok()?;

        match item {
            gix::status::Item::TreeIndex(_) => {
                status |= GitStatus::MODIFIED;
            }
            gix::status::Item::IndexWorktree(item) => match item.summary() {
                Some(gix::status::index_worktree::iter::Summary::Added) => {
                    status |= GitStatus::UNTRACKED;
                }
                Some(_) => {
                    status |= GitStatus::MODIFIED;
                }
                None => {}
            },
        }

        if status.contains(GitStatus::MODIFIED | GitStatus::UNTRACKED) {
            break;
        }
    }

    Some(GitInfo { branch, status })
}

fn get_git_info(backend: GitBackend, current_dir: &Path) -> Option<GitInfo> {
    match backend {
        GitBackend::Binary => get_git_info_binary(current_dir),
        GitBackend::Gix => get_git_info_gix(current_dir),
    }
}

impl Module for GitModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        use crate::style::{AnsiStyle, Color};

        let Ok(current_dir) = std::env::current_dir() else {
            return None;
        };

        let info = get_git_info(context.git_backend, &current_dir)?;

        let branch = sanitize_display_text(&info.branch);
        let has_changes = info.status.contains(GitStatus::MODIFIED);
        let has_untracked = info.status.contains(GitStatus::UNTRACKED);

        let mut indicators = String::new();
        if has_changes {
            indicators.push('+');
        }
        if has_untracked {
            indicators.push('?');
        }

        if context.no_color {
            if indicators.is_empty() {
                Some(format!("[{branch}] "))
            } else {
                Some(format!("[{branch}{indicators}] "))
            }
        } else {
            let blue = AnsiStyle::new(Color::Blue, false);
            let red = AnsiStyle::new(Color::Red, false);

            if indicators.is_empty() {
                Some(format!(
                    "{}[{}]{} ",
                    blue.start_codes(),
                    branch,
                    AnsiStyle::RESET
                ))
            } else {
                Some(format!(
                    "{}[{}{}{}{}]{} ",
                    blue.start_codes(),
                    branch,
                    red.start_codes(),
                    indicators,
                    blue.start_codes(),
                    AnsiStyle::RESET
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};

    use tempfile::TempDir;

    use super::*;

    const TEST_GIT_CONFIG: &str = r#"
[user]
    name = My Prompt Tests
    email = my-prompt@example.invalid
[commit]
    gpgSign = false
[core]
    autocrlf = false
[init]
    defaultBranch = main
[protocol "file"]
    allow = always
"#;

    struct TestRepository {
        _workspace: TempDir,
        root: PathBuf,
        global_config: PathBuf,
    }

    impl TestRepository {
        fn init() -> Self {
            let workspace = tempfile::tempdir().expect("create repository workspace");
            let root = workspace.path().join("repository");
            let global_config = workspace.path().join("global.gitconfig");

            fs::create_dir(&root).expect("create repository directory");
            fs::write(&global_config, TEST_GIT_CONFIG).expect("write isolated Git config");

            let repository = Self {
                _workspace: workspace,
                root,
                global_config,
            };
            repository.git(&["init", "--quiet", "--initial-branch=main"]);
            repository
        }

        fn with_initial_commit() -> Self {
            let repository = Self::init();
            repository.write("tracked.txt", "initial contents\n");
            repository.commit_all("initial commit");
            repository
        }

        fn path(&self) -> &Path {
            &self.root
        }

        fn write(&self, relative_path: &str, contents: &str) {
            let path = self.path().join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create file parent directory");
            }
            fs::write(path, contents).expect("write repository file");
        }

        fn remove(&self, relative_path: &str) {
            fs::remove_file(self.path().join(relative_path)).expect("remove repository file");
        }

        fn commit_all(&self, message: &str) {
            self.git(&["add", "--all"]);
            self.git(&["commit", "--quiet", "--message", message]);
        }

        fn git(&self, args: &[&str]) {
            let output = self.git_output_at(self.path(), args);
            assert!(
                output.status.success(),
                "git {args:?} failed\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        fn git_at(&self, current_dir: &Path, args: &[&str]) {
            let output = self.git_output_at(current_dir, args);
            assert!(
                output.status.success(),
                "git {args:?} failed\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        fn git_output_at(&self, current_dir: &Path, args: &[&str]) -> Output {
            Command::new("git")
                .args(args)
                .current_dir(current_dir)
                .env("GIT_CONFIG_GLOBAL", &self.global_config)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_TERMINAL_PROMPT", "0")
                .env_remove("GIT_DIR")
                .env_remove("GIT_WORK_TREE")
                .output()
                .expect("run Git command")
        }
    }

    fn git_info(branch: &str, status: GitStatus) -> GitInfo {
        GitInfo {
            branch: branch.to_owned(),
            status,
        }
    }

    fn assert_backends(current_dir: &Path, expected: GitInfo) {
        assert_eq!(
            get_git_info(GitBackend::Binary, current_dir),
            Some(expected.clone()),
            "unexpected result from Binary backend"
        );
        assert_eq!(
            get_git_info(GitBackend::Gix, current_dir),
            Some(expected),
            "unexpected result from Gix backend"
        );
    }

    fn assert_backends_omit(current_dir: &Path) {
        assert_eq!(get_git_info(GitBackend::Binary, current_dir), None);
        assert_eq!(get_git_info(GitBackend::Gix, current_dir), None);
    }

    fn repository_with_submodule() -> (TestRepository, TestRepository) {
        let child = TestRepository::with_initial_commit();
        let parent = TestRepository::with_initial_commit();
        let child_path = child.path().to_str().expect("UTF-8 child path");

        parent.git(&["submodule", "add", "--quiet", child_path, "submodule"]);
        parent.commit_all("add submodule");

        (parent, child)
    }

    #[test]
    fn backends_omit_git_info_outside_repository() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");

        assert_backends_omit(temp_dir.path());
    }

    #[test]
    fn backends_report_clean_named_branch() {
        let repository = TestRepository::with_initial_commit();

        assert_backends(repository.path(), git_info("main", GitStatus::empty()));
    }

    #[test]
    fn backends_report_short_named_branch() {
        let repository = TestRepository::with_initial_commit();
        repository.git(&["switch", "--quiet", "--create", "feature/topic"]);

        assert_backends(
            repository.path(),
            git_info("feature/topic", GitStatus::empty()),
        );
    }

    #[test]
    fn backends_report_unborn_branch() {
        let repository = TestRepository::init();

        assert_backends(repository.path(), git_info("main", GitStatus::empty()));
    }

    #[test]
    fn backends_report_detached_head() {
        let repository = TestRepository::with_initial_commit();
        repository.git(&["switch", "--quiet", "--detach", "HEAD"]);

        assert_backends(repository.path(), git_info("HEAD", GitStatus::empty()));
    }

    #[test]
    fn backends_discover_repository_from_nested_directory() {
        let repository = TestRepository::with_initial_commit();
        let nested_directory = repository.path().join("nested/directory");
        repository.write("nested/directory/tracked.txt", "tracked\n");
        repository.commit_all("add nested directory");

        assert_backends(&nested_directory, git_info("main", GitStatus::empty()));
    }

    #[test]
    fn backends_discover_linked_worktree() {
        let repository = TestRepository::with_initial_commit();
        let linked_workspace = tempfile::tempdir().expect("create linked-worktree workspace");
        let linked_worktree = linked_workspace.path().join("linked");
        let linked_path = linked_worktree
            .to_str()
            .expect("UTF-8 linked-worktree path");

        repository.git(&[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            linked_path,
            "HEAD",
        ]);

        assert_backends(&linked_worktree, git_info("HEAD", GitStatus::empty()));
    }

    #[test]
    fn backends_report_staged_addition() {
        let repository = TestRepository::with_initial_commit();
        repository.write("added.txt", "added\n");
        repository.git(&["add", "added.txt"]);

        assert_backends(repository.path(), git_info("main", GitStatus::MODIFIED));
    }

    #[test]
    fn backends_report_unstaged_modification() {
        let repository = TestRepository::with_initial_commit();
        repository.write("tracked.txt", "different unstaged contents\n");

        assert_backends(repository.path(), git_info("main", GitStatus::MODIFIED));
    }

    #[test]
    fn backends_report_staged_modification() {
        let repository = TestRepository::with_initial_commit();
        repository.write("tracked.txt", "different staged contents\n");
        repository.git(&["add", "tracked.txt"]);

        assert_backends(repository.path(), git_info("main", GitStatus::MODIFIED));
    }

    #[test]
    fn backends_report_unstaged_deletion() {
        let repository = TestRepository::with_initial_commit();
        repository.remove("tracked.txt");

        assert_backends(repository.path(), git_info("main", GitStatus::MODIFIED));
    }

    #[test]
    fn backends_report_staged_deletion() {
        let repository = TestRepository::with_initial_commit();
        repository.git(&["rm", "--quiet", "tracked.txt"]);

        assert_backends(repository.path(), git_info("main", GitStatus::MODIFIED));
    }

    #[test]
    fn backends_report_staged_rename() {
        let repository = TestRepository::with_initial_commit();
        repository.git(&["mv", "tracked.txt", "renamed.txt"]);

        assert_backends(repository.path(), git_info("main", GitStatus::MODIFIED));
    }

    #[test]
    fn backends_report_unstaged_rename_as_modified_and_untracked() {
        let repository = TestRepository::with_initial_commit();
        fs::rename(
            repository.path().join("tracked.txt"),
            repository.path().join("renamed.txt"),
        )
        .expect("rename tracked file");

        assert_backends(
            repository.path(),
            git_info("main", GitStatus::MODIFIED | GitStatus::UNTRACKED),
        );
    }

    #[test]
    fn backends_report_conflict() {
        let repository = TestRepository::with_initial_commit();
        repository.git(&["switch", "--quiet", "--create", "other"]);
        repository.write("tracked.txt", "other branch contents\n");
        repository.commit_all("change on other branch");
        repository.git(&["switch", "--quiet", "main"]);
        repository.write("tracked.txt", "main branch contents\n");
        repository.commit_all("change on main branch");

        let merge = repository.git_output_at(repository.path(), &["merge", "other"]);
        assert!(!merge.status.success(), "merge should produce a conflict");

        assert_backends(repository.path(), git_info("main", GitStatus::MODIFIED));
    }

    #[test]
    fn backends_report_unstaged_addition_as_untracked() {
        let repository = TestRepository::with_initial_commit();
        repository.write("untracked.txt", "untracked\n");

        assert_backends(repository.path(), git_info("main", GitStatus::UNTRACKED));
    }

    #[test]
    fn backends_report_untracked_directory() {
        let repository = TestRepository::with_initial_commit();
        repository.write("untracked-directory/file.txt", "untracked\n");

        assert_backends(repository.path(), git_info("main", GitStatus::UNTRACKED));
    }

    #[test]
    fn backends_ignore_ignored_only_files() {
        let repository = TestRepository::with_initial_commit();
        repository.write(".gitignore", "ignored-directory/\n");
        repository.commit_all("add ignore rule");
        repository.write("ignored-directory/file.txt", "ignored\n");

        assert_backends(repository.path(), git_info("main", GitStatus::empty()));
    }

    #[test]
    fn backends_report_modified_and_untracked() {
        let repository = TestRepository::with_initial_commit();
        repository.write("tracked.txt", "different contents\n");
        repository.write("untracked.txt", "untracked\n");

        assert_backends(
            repository.path(),
            git_info("main", GitStatus::MODIFIED | GitStatus::UNTRACKED),
        );
    }

    #[test]
    fn backends_override_status_show_untracked_files() {
        let repository = TestRepository::with_initial_commit();
        repository.git(&["config", "status.showUntrackedFiles", "no"]);
        repository.write("untracked.txt", "untracked\n");

        assert_backends(repository.path(), git_info("main", GitStatus::UNTRACKED));
    }

    #[test]
    fn backends_ignore_dirty_submodule_contents() {
        let (parent, _child) = repository_with_submodule();
        parent.write("submodule/tracked.txt", "dirty submodule contents\n");
        parent.write("submodule/untracked.txt", "untracked inside submodule\n");

        assert_backends(parent.path(), git_info("main", GitStatus::empty()));
    }

    #[test]
    fn backends_report_submodule_commit_pointer_drift() {
        let (parent, _child) = repository_with_submodule();
        let submodule = parent.path().join("submodule");
        parent.write("submodule/tracked.txt", "new submodule commit\n");
        parent.git_at(&submodule, &["add", "tracked.txt"]);
        parent.git_at(
            &submodule,
            &["commit", "--quiet", "--message", "advance submodule"],
        );

        assert_backends(parent.path(), git_info("main", GitStatus::MODIFIED));
    }

    #[test]
    fn backends_report_staged_gitlink() {
        let (parent, _child) = repository_with_submodule();
        let submodule = parent.path().join("submodule");
        parent.write("submodule/tracked.txt", "new submodule commit\n");
        parent.git_at(&submodule, &["add", "tracked.txt"]);
        parent.git_at(
            &submodule,
            &["commit", "--quiet", "--message", "advance submodule"],
        );
        parent.git(&["add", "submodule"]);

        assert_backends(parent.path(), git_info("main", GitStatus::MODIFIED));
    }

    #[test]
    fn backends_omit_git_info_for_corrupt_index() {
        let repository = TestRepository::with_initial_commit();
        fs::write(repository.path().join(".git/index"), b"not a Git index")
            .expect("corrupt repository index");

        assert_backends_omit(repository.path());
    }

    #[test]
    fn parser_reports_clean_repository() {
        let info = parse_git_status_output("## main\n").expect("parse clean status");

        assert_eq!(
            info,
            GitInfo {
                branch: "main".to_owned(),
                status: GitStatus::empty(),
            }
        );
    }

    #[test]
    fn parser_removes_tracking_suffix() {
        let info = parse_git_status_output("## trunk...origin/trunk [ahead 3, behind 1]\n")
            .expect("parse tracking status");

        assert_eq!(info.branch, "trunk");
    }

    #[test]
    fn parser_normalizes_unborn_branch() {
        let info =
            parse_git_status_output("## No commits yet on topic\n").expect("parse unborn status");

        assert_eq!(info.branch, "topic");
    }

    #[test]
    fn parser_normalizes_detached_head() {
        let info = parse_git_status_output("## HEAD (no branch)\n").expect("parse detached status");

        assert_eq!(info.branch, "HEAD");
    }

    #[test]
    fn parser_treats_tracked_status_records_as_modified() {
        for status_record in [
            " M modified.txt",
            "M  staged.txt",
            "A  added.txt",
            " D deleted.txt",
            "R  renamed.txt",
            "UU conflicted.txt",
        ] {
            let output = format!("## main\n{status_record}\n");
            let info = parse_git_status_output(&output).expect("parse tracked status record");

            assert_eq!(info.status, GitStatus::MODIFIED, "record: {status_record}");
        }
    }

    #[test]
    fn parser_treats_untracked_status_records_as_untracked() {
        let info = parse_git_status_output("## main\n?? untracked.txt\n")
            .expect("parse untracked status record");

        assert_eq!(info.status, GitStatus::UNTRACKED);
    }

    #[test]
    fn parser_reports_modified_and_untracked_status() {
        let info = parse_git_status_output("## feature\n M src/main.rs\n?? TODO.md\n")
            .expect("parse combined status");

        assert_eq!(
            info,
            GitInfo {
                branch: "feature".to_owned(),
                status: GitStatus::MODIFIED | GitStatus::UNTRACKED,
            }
        );
    }

    #[test]
    fn parser_rejects_malformed_output() {
        for output in ["", "not a branch header\n", "## \n", "## main\nM\n"] {
            assert!(
                parse_git_status_output(output).is_none(),
                "unexpectedly parsed {output:?}"
            );
        }
    }
}
