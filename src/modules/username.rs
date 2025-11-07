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
    fn render(&self, _format: &str, _context: &ModuleContext) -> Result<Option<String>> {
        let actual_username = username();
        let display_name = match actual_username.as_str() {
            "christopher.becker" => "chris",
            _ => &actual_username,
        };
        Ok(Some(display_name.to_string()))
    }
}
