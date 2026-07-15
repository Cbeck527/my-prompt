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

    #[arg(long, value_enum, default_value = "binary")]
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
    context_window_size: u64,
    used_percentage: Option<f64>,
    current_usage: Option<ClaudeContextWindowCurrentUsage>,
}

#[derive(Deserialize)]
struct ClaudeContextWindowCurrentUsage {
    input_tokens: u64,
    cache_creation_input_tokens: u64,
    cache_read_input_tokens: u64,
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
        handle_bench(
            modules,
            cli.code,
            cli.no_color,
            cli.git_backend,
            claude_session,
        )
    } else {
        handle_format(
            modules,
            cli.debug,
            cli.code,
            cli.no_color,
            cli.git_backend,
            claude_session,
        )
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
    // current_usage is null before the first API call.
    // ref: https://code.claude.com/docs/en/statusline#context-window-fields
    let context_used = parsed.context_window.current_usage.as_ref().map_or(0, |u| {
        u.input_tokens + u.cache_creation_input_tokens + u.cache_read_input_tokens
    });

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
    exit_code: Option<i32>,
    no_color: bool,
    git_backend: GitBackend,
    claude_session: Option<module_trait::ClaudeSession>,
) -> Result<String> {
    let no_color = no_color || std::env::var("NO_COLOR").is_ok();
    let context = module_trait::ModuleContext {
        exit_code,
        no_color,
        claude_session,
        git_backend,
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
        claude_session,
        git_backend,
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

#[cfg(test)]
mod tests {
    use super::*;

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
                "context_window_size": 200000,
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
                "context_window_size": 200000,
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
        assert_eq!(session.context_total, 200000);
        assert_eq!(session.percentage, 8);
    }

    #[test]
    fn test_parse_minimal_json() {
        let session = parse_claude_json(&minimal_json()).unwrap();
        assert_eq!(session.model_name, "Sonnet");
        assert_eq!(session.context_used, 500); // 300 + 100 + 100
        assert_eq!(session.context_total, 200000);
        assert_eq!(session.percentage, 0);
    }

    #[test]
    fn test_parse_null_percentage() {
        let json = serde_json::json!({
            "model": { "display_name": "Opus" },
            "context_window": {
                "context_window_size": 200000,
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
                "context_window_size": 200000
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
                "context_window_size": 200000,
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
                    "context_window_size": 200000,
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
                "context_window_size": 200000,
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
                "context_window_size": 200000,
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
                "context_window_size": 200000,
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
}
