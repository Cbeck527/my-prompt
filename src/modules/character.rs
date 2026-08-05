use crate::module_trait::{Module, ModuleContext};

pub(crate) struct CharacterModule;

impl Module for CharacterModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let symbol = "$";

        if context.no_color {
            Some(format!("{symbol} "))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::White, false);
            Some(format!(
                "{}{}{} ",
                style.start_codes(),
                symbol,
                AnsiStyle::RESET
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_renders_the_shell_symbol() {
        let module = CharacterModule;
        let context = ModuleContext::default();

        let result = module.render(&context);
        assert!(result.is_some());

        let output = result.unwrap();
        assert!(output.contains('$'));
    }
}
