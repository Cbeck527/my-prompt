use std::ffi::OsStr;
use std::hint::black_box;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use clap::{Args, Parser, Subcommand};

mod claude;
mod module_trait;
mod modules;
mod prompt;
mod style;

use module_trait::{EnvironmentState, GitBackend, ModuleContext};

const DIRENV_STATUS_JSON_ENV: &str = "MY_PROMPT_DIRENV_STATUS_JSON";
const FISH_INIT: &str = include_str!("init/my-prompt.fish");
const WARM_BENCHMARK_RUNS: u16 = 100;

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

fn main() -> ExitCode {
    let process_start = Instant::now();
    let Cli { prompt, command } = Cli::parse();

    match command {
        None => run_prompt(&prompt),
        Some(Command::Bench(args)) => run_bench(&args, process_start),
        Some(Command::Claude(args)) => run_claude(&args),
        Some(Command::Init) => write_stdout(FISH_INIT),
    }
}

fn run_prompt(args: &PromptArgs) -> ExitCode {
    run_format_command(
        prompt_modules(args.render.final_rendering),
        args.debug,
        args.render.code,
        None,
        &args.render.common,
    )
}

fn run_claude(args: &ClaudeArgs) -> ExitCode {
    run_format_command(
        prompt::CLAUDE_FORMAT,
        args.debug,
        None,
        parse_claude_stdin(),
        &args.common,
    )
}

fn run_bench(args: &BenchArgs, process_start: Instant) -> ExitCode {
    let modules = prompt_modules(args.render.final_rendering);
    let context = module_context(&args.render.common, args.render.code, None);
    let Some(rayon_threads) = initialize_thread_pool() else {
        return ExitCode::FAILURE;
    };
    let output = handle_bench(modules, &context, process_start, rayon_threads);

    write_stdout(&output)
}

fn run_format_command(
    modules: &[prompt::PromptModule],
    debug: bool,
    exit_code: Option<i32>,
    claude_session: Option<claude::ClaudeSession>,
    args: &CommonRenderArgs,
) -> ExitCode {
    let context = module_context(args, exit_code, claude_session);
    let Some(rayon_threads) = initialize_thread_pool() else {
        return ExitCode::FAILURE;
    };
    let output = handle_format(modules, debug, &context, rayon_threads);

    write_stdout(&output)
}

fn initialize_thread_pool() -> Option<usize> {
    match prompt::init_thread_pool() {
        Ok(thread_count) => Some(thread_count),
        Err(error) => {
            write_stderr(&format!(
                "Error: failed to initialize Rayon thread pool: {error}\n"
            ));
            None
        }
    }
}

fn module_context(
    args: &CommonRenderArgs,
    exit_code: Option<i32>,
    claude_session: Option<claude::ClaudeSession>,
) -> ModuleContext {
    let no_color = std::env::var_os("NO_COLOR");
    let direnv_status_json = std::env::var(DIRENV_STATUS_JSON_ENV)
        .ok()
        .filter(|status_json| !status_json.is_empty());

    ModuleContext {
        exit_code,
        no_color: no_color_requested(args.no_color, no_color.as_deref()),
        claude_session,
        git_backend: args.git_backend,
        direnv_status_json,
        environments: EnvironmentState {
            nix_shell: std::env::var_os("IN_NIX_SHELL").is_some(),
            virtual_env: std::env::var_os("VIRTUAL_ENV").is_some(),
        },
    }
}

fn no_color_requested(flag: bool, environment_value: Option<&OsStr>) -> bool {
    flag || environment_value.is_some_and(|value| !value.is_empty())
}

fn prompt_modules(final_rendering: bool) -> &'static [prompt::PromptModule] {
    if final_rendering {
        prompt::TRANSIENT_FORMAT
    } else {
        prompt::PROMPT_FORMAT
    }
}

fn parse_claude_stdin() -> Option<claude::ClaudeSession> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).ok()?;
    claude::parse_json(&input)
}

fn handle_format(
    modules: &[prompt::PromptModule],
    debug: bool,
    context: &ModuleContext,
    rayon_threads: usize,
) -> String {
    if !debug {
        return prompt::render_prompt(modules, context);
    }

    let start = Instant::now();
    let output = prompt::render_prompt(modules, context);
    let elapsed = start.elapsed();
    write_stderr(&format!(
        "Modules: {modules:?}\nRayon threads: {rayon_threads}\nExecution time: {:.2}ms\n",
        elapsed.as_secs_f64() * 1000.0
    ));

    output
}

fn handle_bench(
    modules: &[prompt::PromptModule],
    context: &ModuleContext,
    process_start: Instant,
    rayon_threads: usize,
) -> String {
    let first_render_start = Instant::now();
    black_box(prompt::render_prompt(modules, context));
    let cold_start = process_start.elapsed();
    let first_render = first_render_start.elapsed();

    let mut times = Vec::with_capacity(usize::from(WARM_BENCHMARK_RUNS));
    for _ in 0..WARM_BENCHMARK_RUNS {
        let start = Instant::now();
        black_box(prompt::render_prompt(modules, context));
        times.push(start.elapsed());
    }

    times.sort_unstable();
    let min = times.first().copied().unwrap_or_default();
    let max = times.last().copied().unwrap_or_default();
    let average = times.iter().sum::<std::time::Duration>() / u32::from(WARM_BENCHMARK_RUNS);
    let p99 = times.get(98).copied().unwrap_or(max);
    let direnv_file = modules::utils::find_upward(".envrc");
    let direnv_lookup = direnv_benchmark_label(
        modules,
        direnv_file.as_deref(),
        context.direnv_status_json.as_deref(),
    );
    let git_backend = context.git_backend;

    format!(
        "Using backend: {git_backend:?}\nRayon threads: {rayon_threads}\nDirenv lookup: {direnv_lookup}\nCold start: {:.2}ms (process start to first render; first render {:.2}ms)\n100 warm runs: min={:.2}ms avg={:.2}ms max={:.2}ms p99={:.2}ms\n",
        cold_start.as_secs_f64() * 1000.0,
        first_render.as_secs_f64() * 1000.0,
        min.as_secs_f64() * 1000.0,
        average.as_secs_f64() * 1000.0,
        max.as_secs_f64() * 1000.0,
        p99.as_secs_f64() * 1000.0
    )
}

fn direnv_benchmark_label(
    modules: &[prompt::PromptModule],
    direnv_file: Option<&Path>,
    cached_status_json: Option<&str>,
) -> &'static str {
    if !modules
        .iter()
        .any(|module| matches!(module, prompt::PromptModule::Direnv))
    {
        return "not rendered";
    }

    modules::direnv::benchmark_label(direnv_file, cached_status_json)
}

fn output_succeeded(result: &io::Result<()>) -> bool {
    match result {
        Ok(()) => true,
        Err(error) => error.kind() == io::ErrorKind::BrokenPipe,
    }
}

fn write_stdout(output: &str) -> ExitCode {
    let mut stdout = io::stdout().lock();
    let result = stdout
        .write_all(output.as_bytes())
        .and_then(|()| stdout.flush());

    if output_succeeded(&result) {
        return ExitCode::SUCCESS;
    }

    if let Err(error) = result {
        write_stderr(&format!(
            "Error: failed to write standard output: {error}\n"
        ));
    }
    ExitCode::FAILURE
}

fn write_stderr(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ = stderr.write_all(message.as_bytes());
    let _ = stderr.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_color_flag_always_disables_color() {
        assert!(no_color_requested(true, None));
        assert!(no_color_requested(true, Some(OsStr::new(""))));
    }

    #[test]
    fn no_color_environment_requires_a_nonempty_value() {
        assert!(!no_color_requested(false, None));
        assert!(!no_color_requested(false, Some(OsStr::new(""))));
        assert!(no_color_requested(false, Some(OsStr::new("1"))));
    }

    #[cfg(unix)]
    #[test]
    fn no_color_accepts_non_utf8_environment_values() {
        use std::os::unix::ffi::OsStrExt;

        assert!(no_color_requested(false, Some(OsStr::from_bytes(&[0xff]))));
    }

    #[test]
    fn broken_pipe_counts_as_successful_output() {
        assert!(output_succeeded(&Ok(())));
        assert!(output_succeeded(&Err(io::Error::from(
            io::ErrorKind::BrokenPipe
        ))));
        assert!(!output_succeeded(&Err(io::Error::from(
            io::ErrorKind::Other
        ))));
    }

    #[test]
    fn direnv_benchmark_reports_when_module_is_not_rendered() {
        assert_eq!(
            direnv_benchmark_label(prompt::TRANSIENT_FORMAT, None, None),
            "not rendered"
        );
    }
}
