use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};
use whoami::fallible::hostname;

pub struct HostnameModule;

impl Default for HostnameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl HostnameModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Module for HostnameModule {
    #[allow(unused)]
    fn render(&self, format: &str, _context: &ModuleContext) -> Result<Option<String>> {
        Ok(hostname().ok())
    }
}
