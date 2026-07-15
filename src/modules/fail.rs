use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};

pub struct FailModule;

impl Default for FailModule {
    fn default() -> Self {
        Self::new()
    }
}

impl FailModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for FailModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let exit_code = context.exit_code.unwrap_or(0);
        if exit_code == 0 {
            return Ok(None);
        }

        let text = format!("exit: {exit_code}");

        if context.no_color {
            Ok(Some(format!("[{text}]\n")))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Red, false);
            Ok(Some(format!(
                "{}[{}]{}\n",
                style.start_codes(),
                text,
                AnsiStyle::RESET
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_hidden_on_success() {
        let module = FailModule::new();
        let context = ModuleContext {
            exit_code: Some(0),
            ..ModuleContext::default()
        };
        let result = module.render(&context).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_fail_shows_exit_code() {
        let module = FailModule::new();
        let context = ModuleContext {
            exit_code: Some(42),
            ..ModuleContext::default()
        };
        let result = module.render(&context).unwrap();
        assert!(result.is_some());
        let output = result.unwrap();
        assert!(output.contains("exit: 42"));
        assert!(output.contains('['));
        assert!(output.contains(']'));
    }

    #[test]
    fn test_fail_no_color() {
        let module = FailModule::new();
        let context = ModuleContext {
            exit_code: Some(1),
            no_color: true,
            ..ModuleContext::default()
        };
        let result = module.render(&context).unwrap();
        assert_eq!(result, Some("[exit: 1]\n".to_string()));
    }
}
