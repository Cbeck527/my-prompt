use crate::module_trait::{Module, ModuleContext};
use crate::modules::utils::sanitize_display_text;

pub(crate) struct ClaudeModule;

/// Formats a token count into a human-readable string.
/// - 0-999: as-is (e.g., "845")
/// - 1,000-9,999: one decimal (e.g., "1.2k")
/// - 10,000-999,999: no decimal (e.g., "12k", "200k")
/// - 1,000,000-9,999,999: one decimal (e.g., "1.0M", "1.5M")
/// - 10,000,000+: no decimal (e.g., "10M")
#[must_use]
fn format_tokens(n: u64) -> String {
    if n >= 10_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000_000 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "token counts in the display range retain sufficient decimal precision"
        )]
        let val = n as f64 / 1_000_000.0;
        format!("{val:.1}M")
    } else if n >= 10_000 {
        format!("{}k", n / 1000)
    } else if n >= 1_000 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "token counts in the display range retain sufficient decimal precision"
        )]
        let val = n as f64 / 1000.0;
        format!("{val:.1}k")
    } else {
        n.to_string()
    }
}

impl Module for ClaudeModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let Some(session) = &context.claude_session else {
            return None;
        };

        let model = sanitize_display_text(&session.model_name);
        let used = format_tokens(session.context_used);
        let total = format_tokens(session.context_total);
        let pct = session.percentage;

        if context.no_color {
            Some(format!("[{model} {used}/{total} ({pct}%)]"))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Magenta, false);
            Some(format!(
                "{}[{} {}/{} ({}%)]{}",
                style.start_codes(),
                model,
                used,
                total,
                pct,
                AnsiStyle::RESET
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claude::ClaudeSession;

    #[test]
    fn small_token_counts_are_written_in_full() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(999), "999");
    }

    #[test]
    fn low_thousands_keep_one_decimal_place() {
        assert_eq!(format_tokens(1000), "1.0k");
        assert_eq!(format_tokens(1234), "1.2k");
        assert_eq!(format_tokens(5678), "5.7k");
        assert_eq!(format_tokens(9999), "10.0k");
    }

    #[test]
    fn high_thousands_drop_the_decimal_place() {
        assert_eq!(format_tokens(10000), "10k");
        assert_eq!(format_tokens(12845), "12k");
        assert_eq!(format_tokens(100_000), "100k");
        assert_eq!(format_tokens(200_000), "200k");
        assert_eq!(format_tokens(999_999), "999k");
    }

    #[test]
    fn millions_follow_the_same_display_thresholds() {
        assert_eq!(format_tokens(1_000_000), "1.0M");
        assert_eq!(format_tokens(1_500_000), "1.5M");
        assert_eq!(format_tokens(2_000_000), "2.0M");
        assert_eq!(format_tokens(9_999_999), "10.0M");
        assert_eq!(format_tokens(10_000_000), "10M");
        assert_eq!(format_tokens(50_000_000), "50M");
    }

    #[test]
    fn module_is_hidden_without_a_claude_session() {
        let module = ClaudeModule;
        let context = ModuleContext::default();

        let result = module.render(&context);
        assert_eq!(result, None);
    }

    #[test]
    fn module_renders_plain_session_text_without_color() {
        let module = ClaudeModule;
        let context = ModuleContext {
            no_color: true,
            claude_session: Some(ClaudeSession {
                model_name: "Opus".to_string(),
                context_used: 12845,
                context_total: 200_000,
                percentage: 6,
            }),
            ..ModuleContext::default()
        };

        let result = module.render(&context);
        assert_eq!(result, Some("[Opus 12k/200k (6%)]".to_string()));
    }

    #[test]
    fn module_renders_session_text_with_color() {
        let module = ClaudeModule;
        let context = ModuleContext {
            no_color: false,
            claude_session: Some(ClaudeSession {
                model_name: "Sonnet".to_string(),
                context_used: 5000,
                context_total: 200_000,
                percentage: 3,
            }),
            ..ModuleContext::default()
        };

        let result = module.render(&context);
        let output = result.unwrap();
        assert!(output.contains("Sonnet"));
        assert!(output.contains("5.0k"));
        assert!(output.contains("200k"));
        assert!(output.contains("3%"));
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn colored_output_escapes_model_controls_and_preserves_trusted_ansi() {
        let module = ClaudeModule;
        let context = ModuleContext {
            no_color: false,
            claude_session: Some(ClaudeSession {
                model_name: "Opus\u{1b}".to_string(),
                context_used: 1,
                context_total: 200_000,
                percentage: 0,
            }),
            ..ModuleContext::default()
        };

        let result = module.render(&context);

        assert_eq!(
            result,
            Some("\x1b[35m[Opus\\u{1b} 1/200k (0%)]\x1b[0m".to_string())
        );
    }
}
