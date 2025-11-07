use crate::error::{PromptError, Result};
use crate::module_trait::{Module, ModuleContext};
use chrono::Local;

pub struct TimeModule;

impl Default for TimeModule {
    fn default() -> Self {
        Self
    }
}

impl Module for TimeModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let now = Local::now();
        let formatted = now.format("%I:%M%p").to_string();

        if context.no_color {
            Ok(Some(format!("[{}] ", formatted)))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Yellow, false);
            Ok(Some(format!("{}[{}]{} ", style.start_codes(), formatted, AnsiStyle::RESET)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_time_module_12h_format() {
        let module = TimeModule;
        let context = ModuleContext::default();

        let result = module.render(&context).unwrap();
        assert!(result.is_some());
        let output = result.unwrap();

        // Should have brackets and space
        assert!(output.starts_with("["));
        assert!(output.contains("]"));

        // Extract just the time part (between brackets)
        let re = Regex::new(r"\[(\d{2}:\d{2}(AM|PM))\]").unwrap();
        assert!(re.is_match(&output), "Expected [hh:MMAM/PM] format, got: {}", output);
    }

    #[test]
    fn test_time_module_no_color() {
        let module = TimeModule;
        let context = ModuleContext {
            exit_code: None,
            no_color: true,
        };

        let result = module.render(&context).unwrap();
        assert!(result.is_some());
        let output = result.unwrap();

        // Should be plain text with brackets
        let re = Regex::new(r"^\[\d{2}:\d{2}(AM|PM)\] $").unwrap();
        assert!(re.is_match(&output), "Expected plain [hh:MMAM/PM] format, got: {}", output);
    }

    #[test]
    fn test_time_module_hour_range() {
        let module = TimeModule;
        let context = ModuleContext::default();

        let result = module.render(&context).unwrap();
        assert!(result.is_some());
        let output = result.unwrap();

        // Extract the hour from the output
        let re = Regex::new(r"\[(\d{2}):\d{2}(AM|PM)\]").unwrap();
        if let Some(caps) = re.captures(&output) {
            let hour = caps[1].parse::<u32>().unwrap();
            assert!(
                hour >= 1 && hour <= 12,
                "12h format hour should be 1-12, got: {}",
                hour
            );
        }
    }
}
