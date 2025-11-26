use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use std::io::{self, Read};
use std::process::ExitCode;
use std::time::Instant;

mod error;
mod module_trait;
mod modules;
mod prompt;
mod style;

use module_trait::GitBackend;

#[derive(Parser)]
#[command(name = "my-prompt")]
#[command(about = "This is my prompt. There are many like it, but this one is mine.")]
#[command(version)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    #[arg(long)]
    debug: bool,

    #[arg(long)]
    bench: bool,

    #[arg(long)]
    code: Option<i32>,

    #[arg(long, value_enum, default_value = "gix")]
    git_backend: GitBackend,

    #[arg(long)]
    no_color: bool,

    #[arg(long, alias = "transient")]
    final_rendering: bool,

    #[arg(long)]
    claude: bool,
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
    total_input_tokens: u64,
    context_window_size: u64,
    used_percentage: f64,
}

fn main() -> ExitCode {
    prompt::init_thread_pool();

    let cli = Cli::parse();

    let (modules, claude_session) = if cli.claude {
        let session = parse_claude_stdin();
        (prompt::CLAUDE_FORMAT, session)
    } else if cli.final_rendering {
        (prompt::TRANSIENT_FORMAT, None)
    } else {
        (prompt::PROMPT_FORMAT, None)
    };

    let result = if cli.bench {
        handle_bench(modules, cli.code, cli.no_color, cli.git_backend, claude_session)
    } else {
        handle_format(modules, cli.debug, cli.code, cli.no_color, cli.git_backend, claude_session)
    };

    match result {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn parse_claude_stdin() -> Option<module_trait::ClaudeSession> {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return None;
    }

    let parsed: ClaudeInput = serde_json::from_str(&input).ok()?;

    Some(module_trait::ClaudeSession {
        model_name: parsed.model.display_name,
        context_used: parsed.context_window.total_input_tokens,
        context_total: parsed.context_window.context_window_size,
        percentage: parsed.context_window.used_percentage.round() as u8,
    })
}

fn handle_format(
    modules: &[prompt::PromptModule],
    debug: bool,
    exit_code: Option<i32>,
    no_color: bool,
    git_backend: GitBackend,
    claude_session: Option<module_trait::ClaudeSession>,
) -> Result<String> {
    let no_color = no_color || std::env::var("NO_COLOR").is_ok();
    let context = module_trait::ModuleContext {
        exit_code,
        no_color,
        git_backend,
        claude_session,
    };

    if debug {
        let start = Instant::now();
        let output = prompt::render_prompt(modules, &context)?;
        let elapsed = start.elapsed();

        eprintln!("Modules: {modules:?}");
        eprintln!("Execution time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);

        Ok(output)
    } else {
        prompt::render_prompt(modules, &context).map_err(|e| anyhow::anyhow!(e))
    }
}

fn handle_bench(
    modules: &[prompt::PromptModule],
    exit_code: Option<i32>,
    no_color: bool,
    git_backend: GitBackend,
    claude_session: Option<module_trait::ClaudeSession>,
) -> Result<String> {
    let no_color = no_color || std::env::var("NO_COLOR").is_ok();
    let context = module_trait::ModuleContext {
        exit_code,
        no_color,
        git_backend,
        claude_session,
    };

    println!("Using backend: {git_backend:?}");

    let mut times = Vec::new();

    for _ in 0..100 {
        let start = Instant::now();
        let _ = prompt::render_prompt(modules, &context).map_err(|e| anyhow::anyhow!(e))?;
        times.push(start.elapsed());
    }

    times.sort();
    let min = times[0];
    let max = times[99];
    let avg: std::time::Duration = times.iter().sum::<std::time::Duration>() / 100;
    let p99 = times[98];

    Ok(format!(
        "100 runs: min={:.2}ms avg={:.2}ms max={:.2}ms p99={:.2}ms\n",
        min.as_secs_f64() * 1000.0,
        avg.as_secs_f64() * 1000.0,
        max.as_secs_f64() * 1000.0,
        p99.as_secs_f64() * 1000.0
    ))
}
