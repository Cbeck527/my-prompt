use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};

pub struct CharacterModule;

impl Default for CharacterModule {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for CharacterModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let symbol = "$";

        if context.no_color {
            Ok(Some(format!("{symbol} ")))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::White, false);
            Ok(Some(format!(
                "{}{}{} ",
                style.start_codes(),
                symbol,
                AnsiStyle::RESET
            )))
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

        let result = module.render(&context).unwrap();
        assert!(result.is_some());

        let output = result.unwrap();
        assert!(output.contains('$'));
    }
}
