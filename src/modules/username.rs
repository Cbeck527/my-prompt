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
    #[allow(unused)]
    fn render(&self, format: &str, _context: &ModuleContext) -> Result<Option<String>> {
        Ok(Some(username()))
    }
}
