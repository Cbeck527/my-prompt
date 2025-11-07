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
    fn render(&self, format: &str, _context: &ModuleContext) -> Result<Option<String>> {
        let symbol = match format {
            "" => "$",
            custom => custom,
        };

        Ok(Some(symbol.to_string()))
    }
}
