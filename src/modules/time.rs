use chrono::{Local, NaiveTime};

use crate::module_trait::{Module, ModuleContext};

pub(crate) struct TimeModule;

fn format_time(time: NaiveTime) -> String {
    time.format("%I:%M%p").to_string()
}

impl Module for TimeModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let formatted = format_time(Local::now().time());

        if context.no_color {
            Some(format!("[{formatted}] "))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Yellow, false);
            Some(format!(
                "{}[{}]{} ",
                style.start_codes(),
                formatted,
                AnsiStyle::RESET
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midnight_uses_twelve_hour_format() {
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).expect("valid midnight");
        assert_eq!(format_time(midnight), "12:00AM");
    }

    #[test]
    fn noon_uses_twelve_hour_format() {
        let noon = NaiveTime::from_hms_opt(12, 0, 0).expect("valid noon");
        assert_eq!(format_time(noon), "12:00PM");
    }

    #[test]
    fn rendered_time_uses_plain_bracketed_output_without_color() {
        let result = TimeModule.render(&ModuleContext {
            no_color: true,
            ..ModuleContext::default()
        });

        let output = result.expect("time always renders");
        assert!(output.starts_with('['));
        assert!(output.ends_with("] "));
        assert!(!output.contains('\u{1b}'));
    }
}
