use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use my_prompt::prompt::MAX_RAYON_THREADS;
use my_prompt::{ModuleContext, PROMPT_FORMAT, render_prompt};

const CLAUDE_INPUT: &str = r#"{
  "model": { "display_name": "Opus" },
  "context_window": {
    "context_window_size": 200000,
    "used_percentage": 6.0,
    "current_usage": {
      "input_tokens": 8500,
      "cache_creation_input_tokens": 5000,
      "cache_read_input_tokens": 2000
    }
  }
}"#;

const OVERFLOWING_CLAUDE_INPUT: &str = r#"{
  "model": { "display_name": "Opus" },
  "context_window": {
    "context_window_size": 200000,
    "current_usage": {
      "input_tokens": 18446744073709551615,
      "cache_creation_input_tokens": 1,
      "cache_read_input_tokens": 0
    }
  }
}"#;

const HOSTILE_CLAUDE_INPUT: &str = r#"{
  "model": { "display_name": "safe\u0000\t\n\r\u001b\u007f\u0085 café" },
  "context_window": {
    "context_window_size": 200000,
    "used_percentage": 6.0,
    "current_usage": {
      "input_tokens": 8500,
      "cache_creation_input_tokens": 5000,
      "cache_read_input_tokens": 2000
    }
  }
}"#;

fn binary_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_my-prompt"))
}

fn output_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn error_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn run_claude_mode(input: &str, current_dir: &Path) -> Output {
    let mut child = binary_command()
        .args(["--claude", "--no-color"])
        .current_dir(current_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start Claude mode");

    child
        .stdin
        .take()
        .expect("Claude stdin")
        .write_all(input.as_bytes())
        .expect("write Claude input");

    child.wait_with_output().expect("collect Claude output")
}

fn run_benchmark_with_threads(thread_count: &str) -> Output {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    binary_command()
        .args(["--bench", "--final-rendering", "--no-color"])
        .current_dir(temp_dir.path())
        .env("RAYON_NUM_THREADS", thread_count)
        .output()
        .expect("run benchmark")
}

fn assert_stdout_has_no_control_characters(output: &Output) {
    let stdout = std::str::from_utf8(&output.stdout).expect("stdout should be valid UTF-8");
    let control = stdout.chars().find(|character| character.is_control());

    assert!(
        control.is_none(),
        "stdout contains control character {control:?}: {stdout:?}"
    );
}

fn path_with_binary_directory() -> std::ffi::OsString {
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_my-prompt"));
    let binary_directory = binary.parent().expect("binary directory");
    let inherited_paths =
        env::var_os("PATH").map_or_else(Vec::new, |path| env::split_paths(&path).collect());
    let paths = std::iter::once(binary_directory.to_path_buf()).chain(inherited_paths);

    env::join_paths(paths).expect("join PATH entries")
}

struct CliGitRepository {
    _workspace: tempfile::TempDir,
    root: PathBuf,
}

fn run_git_setup(current_dir: &Path, global_config: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(current_dir)
        .env("GIT_CONFIG_GLOBAL", global_config)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .expect("run Git setup command");

    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        error_text(&output)
    );
}

impl CliGitRepository {
    fn clean() -> Self {
        let workspace = tempfile::tempdir().expect("create CLI repository workspace");
        let root = workspace.path().join("repository");
        let global_config = workspace.path().join("global.gitconfig");

        fs::create_dir(&root).expect("create CLI repository directory");
        fs::write(
            &global_config,
            "[user]\nname = My Prompt Tests\nemail = my-prompt@example.invalid\n[commit]\ngpgSign = false\n",
        )
        .expect("write isolated Git config");

        run_git_setup(
            &root,
            &global_config,
            &["init", "--quiet", "--initial-branch=main"],
        );
        fs::write(root.join("tracked.txt"), "tracked\n")
            .expect("write tracked CLI repository file");
        run_git_setup(&root, &global_config, &["add", "tracked.txt"]);
        run_git_setup(
            &root,
            &global_config,
            &["commit", "--quiet", "--message", "initial commit"],
        );

        Self {
            _workspace: workspace,
            root,
        }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

#[test]
fn readme_documents_claude_code_command_configuration() {
    let readme = include_str!("../README.md");

    assert!(readme.contains("\"statusLine\""));
    assert!(readme.contains("\"type\": \"command\""));
    assert!(readme.contains("\"command\": \"/absolute/path/to/my-prompt --claude\""));
    assert!(readme.contains("JSON on standard input"));
}

#[test]
fn readme_documents_fish_setup_with_path_requirement() {
    let readme = include_str!("../README.md");

    assert!(readme.contains("## Fish shell prompt"));
    assert!(readme.contains("source /absolute/path/to/my-prompt/etc/my-prompt.fish"));
    assert!(readme.contains("available on your `PATH`"));
}

#[test]
fn help_exits_successfully_and_displays_usage() {
    let output = binary_command().arg("--help").output().expect("run --help");

    assert!(output.status.success(), "{}", error_text(&output));
    assert!(output_text(&output).contains("Usage: my-prompt"));
}

#[test]
fn rayon_thread_count_is_configured_in_a_child_process() {
    let one_thread = run_benchmark_with_threads("1");
    assert!(one_thread.status.success(), "{}", error_text(&one_thread));
    assert!(output_text(&one_thread).contains("Rayon threads: 1"));

    let capped_threads = run_benchmark_with_threads("16");
    assert!(
        capped_threads.status.success(),
        "{}",
        error_text(&capped_threads)
    );
    let host_limit = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let expected_threads = host_limit.min(MAX_RAYON_THREADS);
    assert!(
        output_text(&capped_threads).contains(&format!("Rayon threads: {expected_threads}")),
        "{}",
        output_text(&capped_threads)
    );
}

#[test]
fn claude_mode_reads_stdin_and_renders_without_ansi_codes() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let output = run_claude_mode(CLAUDE_INPUT, temp_dir.path());
    let stdout = output_text(&output);

    assert!(output.status.success(), "{}", error_text(&output));
    assert!(stdout.contains("[Opus 15k/200k (6%)]"), "{stdout}");
    assert!(!stdout.contains("\u{1b}["), "{stdout:?}");
}

#[test]
fn claude_mode_treats_overflowing_context_usage_as_invalid_input() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let output = run_claude_mode(OVERFLOWING_CLAUDE_INPUT, temp_dir.path());
    let stdout = output_text(&output);

    assert!(output.status.success(), "{}", error_text(&output));
    assert!(output.stderr.is_empty(), "{}", error_text(&output));
    assert!(!stdout.contains("[Opus "), "{stdout}");
}

#[test]
fn claude_mode_escapes_control_characters_from_model_name() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let output = run_claude_mode(HOSTILE_CLAUDE_INPUT, temp_dir.path());
    let stdout = output_text(&output);

    assert!(output.status.success(), "{}", error_text(&output));
    assert!(
        stdout.contains(r"[safe\0\t\n\r\u{1b}\u{7f}\u{85} café 15k/200k (6%)]"),
        "{stdout:?}"
    );
    assert_stdout_has_no_control_characters(&output);
}

#[cfg(unix)]
#[test]
fn path_module_escapes_control_characters_from_current_directory() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let hostile_dir = temp_dir.path().join("safe\t\n\r\u{1b}\u{7f}\u{85} café");
    fs::create_dir(&hostile_dir).expect("create hostile directory");

    let output = binary_command()
        .arg("--no-color")
        .current_dir(&hostile_dir)
        .output()
        .expect("render prompt from hostile directory");
    let stdout = output_text(&output);

    assert!(output.status.success(), "{}", error_text(&output));
    assert!(
        stdout.contains(r"safe\t\n\r\u{1b}\u{7f}\u{85} café"),
        "{stdout:?}"
    );
    assert_stdout_has_no_control_characters(&output);
}

#[test]
fn code_option_renders_the_previous_exit_code() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let output = binary_command()
        .args(["--code", "42", "--no-color"])
        .current_dir(temp_dir.path())
        .output()
        .expect("run with exit code");

    assert!(output.status.success(), "{}", error_text(&output));
    assert!(output_text(&output).contains("[exit: 42]"));
}

#[test]
fn cli_prompt_output_matches_public_library_rendering() {
    let context = ModuleContext {
        exit_code: Some(42),
        no_color: true,
        ..ModuleContext::default()
    };
    let expected = render_prompt(PROMPT_FORMAT, &context).expect("render prompt through library");
    let output = binary_command()
        .args(["--code", "42", "--no-color"])
        .output()
        .expect("render prompt through CLI");

    assert!(output.status.success(), "{}", error_text(&output));
    assert_eq!(output_text(&output), expected);
}

#[test]
fn invalid_arguments_exit_with_clap_usage_error() {
    let output = binary_command()
        .arg("--not-a-real-option")
        .output()
        .expect("run invalid command");

    assert_eq!(output.status.code(), Some(2));
    assert!(error_text(&output).contains("unexpected argument"));
}

#[test]
fn git2_backend_is_rejected() {
    let output = binary_command()
        .args(["--git-backend", "git2"])
        .output()
        .expect("run removed Git backend");

    assert_eq!(output.status.code(), Some(2));
    assert!(error_text(&output).contains("invalid value 'git2'"));
}

#[test]
fn binary_backend_omits_git_segment_when_git_is_missing() {
    let repository = CliGitRepository::clean();
    let output = binary_command()
        .args(["--no-color", "--git-backend", "binary"])
        .current_dir(repository.path())
        .env("PATH", "")
        .output()
        .expect("render with missing Git binary");
    let stdout = output_text(&output);

    assert!(output.status.success(), "{}", error_text(&output));
    assert!(!stdout.contains("[main"), "{stdout}");
}

#[test]
fn gix_backend_renders_when_child_process_path_is_empty() {
    let repository = CliGitRepository::clean();
    let output = binary_command()
        .args(["--no-color", "--git-backend", "gix"])
        .current_dir(repository.path())
        .env("PATH", "")
        .output()
        .expect("render with gix and an empty PATH");
    let stdout = output_text(&output);

    assert!(output.status.success(), "{}", error_text(&output));
    assert!(stdout.contains("[main]"), "{stdout}");
}

#[test]
fn fish_helper_is_syntax_valid_and_runs_in_a_clean_fish_process() {
    let helper = Path::new(env!("CARGO_MANIFEST_DIR")).join("etc/my-prompt.fish");
    let syntax_output = Command::new("fish")
        .arg("-n")
        .arg(&helper)
        .output()
        .expect("check Fish syntax");

    assert!(
        syntax_output.status.success(),
        "{}",
        error_text(&syntax_output)
    );

    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let smoke_output = Command::new("fish")
        .args([
            "--no-config",
            "--private",
            "-c",
            "source $MY_PROMPT_FISH_HELPER; fish_prompt",
        ])
        .current_dir(temp_dir.path())
        .env("MY_PROMPT_FISH_HELPER", &helper)
        .env("PATH", path_with_binary_directory())
        .output()
        .expect("run Fish prompt helper");

    assert!(
        smoke_output.status.success(),
        "{}",
        error_text(&smoke_output)
    );
    assert!(!output_text(&smoke_output).is_empty());
}
