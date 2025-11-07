use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;
use std::time::Instant;

mod cache;
mod error;
mod executor;
mod module_trait;
mod modules;
mod parser;
mod registry;
mod style;

const MY_PROMPT_FORMAT: &str = "{fail:red:code:[:]\n}{username:green} {path:white} {git:blue:full,status=red:[:]} {character:white:$} ";

const MY_TRANSIENT_PROMPT_FORMAT: &str = "{time:yellow:12h:[:]} {character:white:$} ";

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
    let cli = Cli::parse();

    let format = if cli.final_rendering {
        MY_TRANSIENT_PROMPT_FORMAT
    } else {
        MY_PROMPT_FORMAT
    };

    let result = if cli.bench {
        handle_bench(format, cli.code, cli.no_color)
    } else {
        handle_format(format, cli.debug, cli.code, cli.no_color)
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
    format: &str,
    debug: bool,
    exit_code: Option<i32>,
    no_color: bool,
) -> Result<String> {
    if debug {
        let start = Instant::now();
        let output = executor::execute(format, exit_code, no_color)?;
        let elapsed = start.elapsed();

        eprintln!("Format: {format}");
        eprintln!("Execution time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);

        Ok(output)
    } else {
        executor::execute(format, exit_code, no_color).map_err(|e| anyhow::anyhow!(e))
    }
}

fn handle_bench(format: &str, exit_code: Option<i32>, no_color: bool) -> Result<String> {
    let mut times = Vec::new();

    for _ in 0..100 {
        let start = Instant::now();
        let _ = executor::execute(format, exit_code, no_color).map_err(|e| anyhow::anyhow!(e))?;
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
