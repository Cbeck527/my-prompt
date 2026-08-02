use crate::module_trait::{Module, ModuleContext};

pub(crate) struct CharacterModule;

impl Default for CharacterModule {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterModule {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self
    }
}

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
    fn test_character_renders() {
        let module = CharacterModule::new();
        let context = ModuleContext::default();

        let result = module.render(&context);
        assert!(result.is_some());

        let output = result.unwrap();
        assert!(output.contains('$'));
    }
}
