use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};

pub struct ClaudeModule;

impl Default for ClaudeModule {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Formats a token count into a human-readable string.
/// - 0-999: as-is (e.g., "845")
/// - 1,000-9,999: one decimal (e.g., "1.2k")
/// - 10,000-999,999: no decimal (e.g., "12k", "200k")
/// - 1,000,000-9,999,999: one decimal (e.g., "1.0M", "1.5M")
/// - 10,000,000+: no decimal (e.g., "10M")
#[must_use]
pub fn format_tokens(n: u64) -> String {
    if n >= 10_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000_000 {
        #[allow(clippy::cast_precision_loss)]
        let val = n as f64 / 1_000_000.0;
        format!("{val:.1}M")
    } else if n >= 10_000 {
        format!("{}k", n / 1000)
    } else if n >= 1_000 {
        #[allow(clippy::cast_precision_loss)] // token counts are well within f64 precision
        let val = n as f64 / 1000.0;
        format!("{val:.1}k")
    } else {
        n.to_string()
    }
}

impl Module for ClaudeModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let Some(session) = &context.claude_session else {
            return Ok(None);
        };

        let model = &session.model_name;
        let used = format_tokens(session.context_used);
        let total = format_tokens(session.context_total);
        let pct = session.percentage;

        if context.no_color {
            Ok(Some(format!("[{model} {used}/{total} ({pct}%)]")))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Magenta, false);
            Ok(Some(format!(
                "{}[{} {}/{} ({}%)]{}",
                style.start_codes(),
                model,
                used,
                total,
                pct,
                AnsiStyle::RESET
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_trait::ClaudeSession;

    #[test]
    fn test_format_tokens_small() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(999), "999");
    }

    #[test]
    fn test_format_tokens_thousands() {
        assert_eq!(format_tokens(1000), "1.0k");
        assert_eq!(format_tokens(1234), "1.2k");
        assert_eq!(format_tokens(5678), "5.7k");
        assert_eq!(format_tokens(9999), "10.0k");
    }

    #[test]
    fn test_format_tokens_large() {
        assert_eq!(format_tokens(10000), "10k");
        assert_eq!(format_tokens(12845), "12k");
        assert_eq!(format_tokens(100000), "100k");
        assert_eq!(format_tokens(200000), "200k");
        assert_eq!(format_tokens(999_999), "999k");
    }

    #[test]
    fn test_format_tokens_millions() {
        assert_eq!(format_tokens(1_000_000), "1.0M");
        assert_eq!(format_tokens(1_500_000), "1.5M");
        assert_eq!(format_tokens(2_000_000), "2.0M");
        assert_eq!(format_tokens(9_999_999), "10.0M");
        assert_eq!(format_tokens(10_000_000), "10M");
        assert_eq!(format_tokens(50_000_000), "50M");
    }

    #[test]
    fn test_claude_module_no_session() {
        let module = ClaudeModule::new();
        let context = ModuleContext::default();

        let result = module.render(&context).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_claude_module_with_session_no_color() {
        let module = ClaudeModule::new();
        let context = ModuleContext {
            no_color: true,
            claude_session: Some(ClaudeSession {
                model_name: "Opus".to_string(),
                context_used: 12845,
                context_total: 200000,
                percentage: 6,
            }),
            ..ModuleContext::default()
        };

        let result = module.render(&context).unwrap();
        assert_eq!(result, Some("[Opus 12k/200k (6%)]".to_string()));
    }

    #[test]
    fn test_claude_module_with_session_color() {
        let module = ClaudeModule::new();
        let context = ModuleContext {
            no_color: false,
            claude_session: Some(ClaudeSession {
                model_name: "Sonnet".to_string(),
                context_used: 5000,
                context_total: 200000,
                percentage: 3,
            }),
            ..ModuleContext::default()
        };

        let result = module.render(&context).unwrap();
        let output = result.unwrap();
        assert!(output.contains("Sonnet"));
        assert!(output.contains("5.0k"));
        assert!(output.contains("200k"));
        assert!(output.contains("3%"));
        assert!(output.contains("\x1b[")); // Contains ANSI codes
    }
}
