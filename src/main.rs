use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;
use std::time::Instant;

mod cache;
mod error;
mod module_trait;
mod modules;
mod prompt;
mod style;

// TODO: implement the following modules: direnv

#[derive(Parser)]
#[command(name = "my-prompt")]
#[command(about = "This is my prompt. There are many like it, but this one is mine.")]
#[command(version)]
struct Cli {
    #[arg(long)]
    debug: bool,

    #[arg(long)]
    bench: bool,

    #[arg(long)]
    code: Option<i32>,

    #[arg(long)]
    no_color: bool,

    #[arg(long, alias = "transient")]
    final_rendering: bool,
}

fn main() -> ExitCode {
    prompt::init_thread_pool();

    let cli = Cli::parse();

    let modules = if cli.final_rendering {
        prompt::TRANSIENT_FORMAT
    } else {
        prompt::PROMPT_FORMAT
    };

    let result = if cli.bench {
        handle_bench(modules, cli.code, cli.no_color)
    } else {
        handle_format(modules, cli.debug, cli.code, cli.no_color)
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

fn handle_format(
    modules: &[prompt::PromptModule],
    debug: bool,
    exit_code: Option<i32>,
    no_color: bool,
) -> Result<String> {
    let no_color = no_color || std::env::var("NO_COLOR").is_ok();
    let context = module_trait::ModuleContext {
        exit_code,
        no_color,
    };

    if debug {
        let start = Instant::now();
        let output = prompt::render_prompt(modules, &context)?;
        let elapsed = start.elapsed();

        eprintln!("Modules: {:?}", modules);
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
) -> Result<String> {
    let no_color = no_color || std::env::var("NO_COLOR").is_ok();
    let context = module_trait::ModuleContext {
        exit_code,
        no_color,
    };

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
