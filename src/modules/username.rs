use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};
use whoami::username;

pub struct UsernameModule;

impl Default for UsernameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl UsernameModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for UsernameModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let actual_username = username();
        let display_name = match actual_username.as_str() {
            "christopher.becker" => "chris",
            _ => &actual_username,
        };

        if context.no_color {
            Ok(Some(format!("{display_name} ")))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Green, false);
            Ok(Some(format!(
                "{}{}{} ",
                style.start_codes(),
                display_name,
                AnsiStyle::RESET
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_renders() {
        let module = UsernameModule::new();
        let context = ModuleContext::default();

        let result = module.render(&context).unwrap();
        assert!(result.is_some());

        let output = result.unwrap();
        assert!(!output.is_empty());
    }
}
