use crate::module_trait::{Module, ModuleContext};

pub(crate) struct FailModule;

impl Module for FailModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let exit_code = context.exit_code.unwrap_or(0);
        if exit_code == 0 {
            return None;
        }

        let text = format!("exit: {exit_code}");

        if context.no_color {
            Some(format!("[{text}]\n"))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Red, false);
            Some(format!(
                "{}[{}]{}\n",
                style.start_codes(),
                text,
                AnsiStyle::RESET
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_is_hidden_after_a_successful_command() {
        let module = FailModule;
        let context = ModuleContext {
            exit_code: Some(0),
            ..ModuleContext::default()
        };
        let result = module.render(&context);
        assert_eq!(result, None);
    }

    #[test]
    fn module_shows_a_nonzero_exit_code() {
        let module = FailModule;
        let context = ModuleContext {
            exit_code: Some(42),
            ..ModuleContext::default()
        };
        let result = module.render(&context);
        assert!(result.is_some());
        let output = result.unwrap();
        assert!(output.contains("exit: 42"));
        assert!(output.contains('['));
        assert!(output.contains(']'));
    }

    #[test]
    fn module_has_plain_output_when_color_is_disabled() {
        let module = FailModule;
        let context = ModuleContext {
            exit_code: Some(1),
            no_color: true,
            ..ModuleContext::default()
        };
        let result = module.render(&context);
        assert_eq!(result, Some("[exit: 1]\n".to_string()));
    }
}
