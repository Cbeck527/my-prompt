use crate::module_trait::{Module, ModuleContext};
use crate::modules::utils::sanitize_display_text;
use whoami::username;

pub(crate) struct UsernameModule;

impl Module for UsernameModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let actual_username = username().unwrap_or_else(|_| "unknown".to_owned());
        let display_name = match actual_username.as_str() {
            "christopher.becker" => "chris",
            _ => &actual_username,
        };
        let display_name = sanitize_display_text(display_name);

        if context.no_color {
            Some(format!("{display_name} "))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Green, false);
            Some(format!(
                "{}{}{} ",
                style.start_codes(),
                display_name,
                AnsiStyle::RESET
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_renders_a_nonempty_username() {
        let module = UsernameModule;
        let context = ModuleContext::default();

        let result = module.render(&context);
        assert!(result.is_some());

        let output = result.unwrap();
        assert!(!output.is_empty());
    }
}
