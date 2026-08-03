use std::io::{self, Read};
use std::process::ExitCode;
use std::time::Instant;

use clap::{Args, Parser, Subcommand};
use serde::Deserialize;

mod module_trait;
mod modules;
mod prompt;
mod style;

use module_trait::GitBackend;

const DIRENV_STATUS_JSON_ENV: &str = "MY_PROMPT_DIRENV_STATUS_JSON";
const FISH_INIT: &str = include_str!("init/my-prompt.fish");

#[derive(Parser)]
#[command(name = "my-prompt")]
#[command(about = "This is my prompt. There are many like it, but this one is mine.")]
#[command(version)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(flatten)]
    prompt: PromptArgs,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Args)]
struct CommonRenderArgs {
    /// Git implementation to use for repository information.
    #[arg(long, value_enum, default_value = "binary")]
    git_backend: GitBackend,

    /// Disable ANSI color output.
    #[arg(long)]
    no_color: bool,
}

#[derive(Args)]
struct PromptRenderArgs {
    #[command(flatten)]
    common: CommonRenderArgs,

    /// Exit code from the previously executed command.
    #[arg(long)]
    code: Option<i32>,

    /// Render Fish's transient prompt.
    #[arg(long, alias = "transient")]
    final_rendering: bool,
}

#[derive(Args)]
struct PromptArgs {
    #[command(flatten)]
    render: PromptRenderArgs,

    /// Print module and timing diagnostics to standard error.
    #[arg(long)]
    debug: bool,
}

#[derive(Args)]
struct BenchArgs {
    #[command(flatten)]
    render: PromptRenderArgs,
}

#[derive(Args)]
struct ClaudeArgs {
    #[command(flatten)]
    common: CommonRenderArgs,

    /// Print module and timing diagnostics to standard error.
    #[arg(long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Benchmark regular or transient prompt rendering.
    Bench(BenchArgs),
    /// Render a Claude Code statusline from JSON on standard input.
    Claude(ClaudeArgs),
    /// Print Fish shell initialization code.
    Init,
}

#[derive(Deserialize)]
struct ClaudeInput {
    model: ClaudeModel,
    context_window: ClaudeContextWindow,
}

#[derive(Deserialize)]
struct ClaudeModel {
    display_name: String,
}

#[derive(Deserialize)]
struct ClaudeContextWindow {
    context_window_size: u64,
    used_percentage: Option<f64>,
    current_usage: Option<ClaudeContextWindowCurrentUsage>,
}

#[allow(clippy::struct_field_names)]
#[derive(Deserialize)]
struct ClaudeContextWindowCurrentUsage {
    input_tokens: u64,
    cache_creation_input_tokens: u64,
    cache_read_input_tokens: u64,
}

fn main() -> ExitCode {
    let process_start = Instant::now();

    let Cli { prompt, command } = Cli::parse();

    match command {
        None => run_prompt(&prompt),
        Some(Command::Bench(args)) => run_bench(&args, process_start),
        Some(Command::Claude(args)) => run_claude(&args),
        Some(Command::Init) => {
            print!("{FISH_INIT}");
            ExitCode::SUCCESS
        }
    }
}

fn run_prompt(args: &PromptArgs) -> ExitCode {
    let modules = prompt_modules(args.render.final_rendering);

    run_format_command(
        modules,
        args.debug,
        args.render.code,
        None,
        &args.render.common,
    )
}

fn run_claude(args: &ClaudeArgs) -> ExitCode {
    let claude_session = parse_claude_stdin();

    run_format_command(
        prompt::CLAUDE_FORMAT,
        args.debug,
        None,
        claude_session,
        &args.common,
    )
}

fn run_bench(args: &BenchArgs, process_start: Instant) -> ExitCode {
    let Some(rayon_threads) = initialize_thread_pool() else {
        return ExitCode::FAILURE;
    };
    let modules = prompt_modules(args.render.final_rendering);
    let context = module_context(&args.render.common, args.render.code, None);
    let output = handle_bench(modules, &context, process_start, rayon_threads);

    print!("{output}");
    ExitCode::SUCCESS
}

fn run_format_command(
    modules: &[prompt::PromptModule],
    debug: bool,
    exit_code: Option<i32>,
    claude_session: Option<module_trait::ClaudeSession>,
    args: &CommonRenderArgs,
) -> ExitCode {
    let Some(rayon_threads) = initialize_thread_pool() else {
        return ExitCode::FAILURE;
    };
    let context = module_context(args, exit_code, claude_session);
    let output = handle_format(modules, debug, &context, rayon_threads);

    print!("{output}");
    ExitCode::SUCCESS
}

fn initialize_thread_pool() -> Option<usize> {
    match prompt::init_thread_pool() {
        Ok(thread_count) => Some(thread_count),
        Err(error) => {
            eprintln!("Error: failed to initialize Rayon thread pool: {error}");
            None
        }
    }
}

fn module_context(
    args: &CommonRenderArgs,
    exit_code: Option<i32>,
    claude_session: Option<module_trait::ClaudeSession>,
) -> module_trait::ModuleContext {
    let direnv_status_json = std::env::var(DIRENV_STATUS_JSON_ENV)
        .ok()
        .filter(|status_json| !status_json.is_empty());

    module_trait::ModuleContext {
        exit_code,
        no_color: args.no_color || std::env::var("NO_COLOR").is_ok(),
        claude_session,
        git_backend: args.git_backend,
        direnv_status_json,
    }
}

fn prompt_modules(final_rendering: bool) -> &'static [prompt::PromptModule] {
    if final_rendering {
        prompt::TRANSIENT_FORMAT
    } else {
        prompt::PROMPT_FORMAT
    }
}

fn parse_claude_stdin() -> Option<module_trait::ClaudeSession> {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return None;
    }
    parse_claude_json(&input)
}

fn parse_claude_json(input: &str) -> Option<module_trait::ClaudeSession> {
    let parsed: ClaudeInput = serde_json::from_str(input).ok()?;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let percentage = parsed
        .context_window
        .used_percentage
        .map_or(0, |p| p.round().clamp(0.0, 100.0) as u8);

    // Context used = input_tokens + cache_creation + cache_read (no output_tokens).
    // Missing usage is valid before the first API call and represents zero usage.
    // An aggregate that cannot fit in u64 invalidates only this Claude session.
    // ref: https://code.claude.com/docs/en/statusline#context-window-fields
    let context_used = match parsed.context_window.current_usage.as_ref() {
        Some(usage) => usage
            .input_tokens
            .checked_add(usage.cache_creation_input_tokens)?
            .checked_add(usage.cache_read_input_tokens)?,
        None => 0,
    };

    Some(module_trait::ClaudeSession {
        model_name: parsed.model.display_name,
        context_used,
        context_total: parsed.context_window.context_window_size,
        percentage,
    })
}

fn handle_format(
    modules: &[prompt::PromptModule],
    debug: bool,
    context: &module_trait::ModuleContext,
    rayon_threads: usize,
) -> String {
    if debug {
        let start = Instant::now();
        let output = prompt::render_prompt(modules, context);
        let elapsed = start.elapsed();

        eprintln!("Modules: {modules:?}");
        eprintln!("Rayon threads: {rayon_threads}");
        eprintln!("Execution time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);

        output
    } else {
        prompt::render_prompt(modules, context)
    }
}

fn handle_bench(
    modules: &[prompt::PromptModule],
    context: &module_trait::ModuleContext,
    process_start: Instant,
    rayon_threads: usize,
) -> String {
    let first_render_start = Instant::now();
    let _ = prompt::render_prompt(modules, context);
    let cold_start = process_start.elapsed();
    let first_render = first_render_start.elapsed();

    let mut times = Vec::with_capacity(100);

    for _ in 0..100 {
        let start = Instant::now();
        let _ = prompt::render_prompt(modules, context);
        times.push(start.elapsed());
    }

    times.sort();
    let min = times[0];
    let max = times[99];
    let avg: std::time::Duration = times.iter().sum::<std::time::Duration>() / 100;
    let p99 = times[98];
    let has_direnv_file = modules::utils::find_upward(".envrc").is_some();
    let direnv_lookup =
        direnv_benchmark_label(has_direnv_file, context.direnv_status_json.is_some());
    let git_backend = context.git_backend;

    format!(
        "Using backend: {git_backend:?}\nRayon threads: {rayon_threads}\nDirenv lookup: {direnv_lookup}\nCold start: {:.2}ms (process start to first render; first render {:.2}ms)\n100 warm runs: min={:.2}ms avg={:.2}ms max={:.2}ms p99={:.2}ms\n",
        cold_start.as_secs_f64() * 1000.0,
        first_render.as_secs_f64() * 1000.0,
        min.as_secs_f64() * 1000.0,
        avg.as_secs_f64() * 1000.0,
        max.as_secs_f64() * 1000.0,
        p99.as_secs_f64() * 1000.0
    )
}

fn direnv_benchmark_label(has_direnv_file: bool, has_cached_status: bool) -> &'static str {
    if !has_direnv_file {
        "no .envrc"
    } else if has_cached_status {
        "shell cache"
    } else {
        "external status"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direnv_benchmark_label_reports_no_envrc() {
        assert_eq!(direnv_benchmark_label(false, false), "no .envrc");
    }

    #[test]
    fn direnv_benchmark_label_reports_shell_cache() {
        assert_eq!(direnv_benchmark_label(true, true), "shell cache");
    }

    #[test]
    fn direnv_benchmark_label_reports_external_status() {
        assert_eq!(direnv_benchmark_label(true, false), "external status");
    }

    fn full_json() -> String {
        serde_json::json!({
            "cwd": "/some/dir",
            "session_id": "abc123",
            "model": {
                "id": "claude-opus-4-6",
                "display_name": "Opus"
            },
            "context_window": {
                "total_input_tokens": 15234,
                "total_output_tokens": 4521,
                "context_window_size": 200_000,
                "used_percentage": 8.0,
                "remaining_percentage": 92.0,
                "current_usage": {
                    "input_tokens": 8500,
                    "output_tokens": 1200,
                    "cache_creation_input_tokens": 5000,
                    "cache_read_input_tokens": 2000
                }
            },
            "cost": {
                "total_cost_usd": 0.01234,
                "total_duration_ms": 45000,
                "total_api_duration_ms": 2300
            },
            "version": "1.0.80"
        })
        .to_string()
    }

    fn minimal_json() -> String {
        serde_json::json!({
            "model": {
                "display_name": "Sonnet"
            },
            "context_window": {
                "context_window_size": 200_000,
                "used_percentage": 0.3,
                "current_usage": {
                    "input_tokens": 300,
                    "cache_creation_input_tokens": 100,
                    "cache_read_input_tokens": 100
                }
            }
        })
        .to_string()
    }

    #[test]
    fn test_parse_full_json() {
        let session = parse_claude_json(&full_json()).unwrap();
        assert_eq!(session.model_name, "Opus");
        assert_eq!(session.context_used, 15500);
        assert_eq!(session.context_total, 200_000);
        assert_eq!(session.percentage, 8);
    }

    #[test]
    fn test_parse_minimal_json() {
        let session = parse_claude_json(&minimal_json()).unwrap();
        assert_eq!(session.model_name, "Sonnet");
        assert_eq!(session.context_used, 500); // 300 + 100 + 100
        assert_eq!(session.context_total, 200_000);
        assert_eq!(session.percentage, 0);
    }

    #[test]
    fn test_parse_null_percentage() {
        let json = serde_json::json!({
            "model": { "display_name": "Opus" },
            "context_window": {
                "context_window_size": 200_000,
                "used_percentage": null,
                "current_usage": null
            }
        })
        .to_string();

        let session = parse_claude_json(&json).unwrap();
        assert_eq!(session.percentage, 0);
        assert_eq!(session.context_used, 0);
    }

    #[test]
    fn test_parse_missing_percentage_and_usage() {
        // Both absent entirely (early session) -- defaults to 0
        let json = serde_json::json!({
            "model": { "display_name": "Opus" },
            "context_window": {
                "context_window_size": 200_000
            }
        })
        .to_string();

        let session = parse_claude_json(&json).unwrap();
        assert_eq!(session.percentage, 0);
        assert_eq!(session.context_used, 0);
    }

    #[test]
    fn test_parse_null_current_usage() {
        // current_usage is null before the first API call
        let json = serde_json::json!({
            "model": { "display_name": "Opus" },
            "context_window": {
                "context_window_size": 200_000,
                "used_percentage": 0.0,
                "current_usage": null
            }
        })
        .to_string();

        let session = parse_claude_json(&json).unwrap();
        assert_eq!(session.context_used, 0);
    }

    #[test]
    fn test_parse_percentage_rounding() {
        let make_json = |pct: f64| {
            serde_json::json!({
                "model": { "display_name": "Opus" },
                "context_window": {
                    "context_window_size": 200_000,
                    "used_percentage": pct,
                    "current_usage": {
                        "input_tokens": 1000,
                        "cache_creation_input_tokens": 0,
                        "cache_read_input_tokens": 0
                    }
                }
            })
            .to_string()
        };

        assert_eq!(parse_claude_json(&make_json(8.4)).unwrap().percentage, 8);
        assert_eq!(parse_claude_json(&make_json(8.5)).unwrap().percentage, 9);
        assert_eq!(parse_claude_json(&make_json(99.7)).unwrap().percentage, 100);
        assert_eq!(parse_claude_json(&make_json(0.0)).unwrap().percentage, 0);
    }

    #[test]
    fn test_parse_unknown_fields_ignored() {
        // Forward-compat: unknown fields should be silently ignored
        let json = serde_json::json!({
            "model": {
                "id": "claude-opus-4-6",
                "display_name": "Opus",
                "some_new_field": true
            },
            "context_window": {
                "context_window_size": 200_000,
                "used_percentage": 3.0,
                "current_usage": {
                    "input_tokens": 3000,
                    "cache_creation_input_tokens": 1000,
                    "cache_read_input_tokens": 1000
                },
                "some_future_field": "hello"
            },
            "vim": { "mode": "NORMAL" },
            "agent": { "name": "test-agent" },
            "worktree": { "name": "feat", "path": "/tmp/wt" },
            "exceeds_200k_tokens": false
        })
        .to_string();

        let session = parse_claude_json(&json).unwrap();
        assert_eq!(session.model_name, "Opus");
        assert_eq!(session.context_used, 5000); // 3000 + 1000 + 1000
    }

    #[test]
    fn test_parse_empty_string() {
        assert!(parse_claude_json("").is_none());
    }

    #[test]
    fn test_parse_invalid_json() {
        assert!(parse_claude_json("{not json}").is_none());
    }

    #[test]
    fn test_parse_missing_required_fields() {
        // Missing model entirely
        let json = serde_json::json!({
            "context_window": {
                "context_window_size": 200_000,
                "used_percentage": 1.0
            }
        })
        .to_string();
        assert!(parse_claude_json(&json).is_none());

        // Missing context_window entirely
        let json = serde_json::json!({
            "model": { "display_name": "Opus" }
        })
        .to_string();
        assert!(parse_claude_json(&json).is_none());
    }

    #[test]
    fn test_parse_large_context_window() {
        // 1M context window
        let json = serde_json::json!({
            "model": { "display_name": "Opus" },
            "context_window": {
                "context_window_size": 1_000_000,
                "used_percentage": 50.0,
                "current_usage": {
                    "input_tokens": 400_000,
                    "cache_creation_input_tokens": 50_000,
                    "cache_read_input_tokens": 50_000
                }
            }
        })
        .to_string();

        let session = parse_claude_json(&json).unwrap();
        assert_eq!(session.context_total, 1_000_000);
        assert_eq!(session.context_used, 500_000);
        assert_eq!(session.percentage, 50);
    }

    #[test]
    fn test_parse_context_used_sums_input_fields() {
        let json = serde_json::json!({
            "model": { "display_name": "Opus" },
            "context_window": {
                "context_window_size": 200_000,
                "used_percentage": 10.0,
                "current_usage": {
                    "input_tokens": 10000,
                    "output_tokens": 5000,
                    "cache_creation_input_tokens": 7000,
                    "cache_read_input_tokens": 3000
                }
            }
        })
        .to_string();

        let session = parse_claude_json(&json).unwrap();
        // output_tokens should NOT be included
        assert_eq!(session.context_used, 20000); // 10000 + 7000 + 3000
    }

    #[test]
    fn test_parse_context_used_overflow_returns_none() {
        let json = serde_json::json!({
            "model": { "display_name": "Opus" },
            "context_window": {
                "context_window_size": 200_000,
                "current_usage": {
                    "input_tokens": u64::MAX,
                    "cache_creation_input_tokens": 1,
                    "cache_read_input_tokens": 0
                }
            }
        })
        .to_string();

        assert!(parse_claude_json(&json).is_none());
    }
}
