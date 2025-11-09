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
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let Some(hostname) = hostname().ok() else {
            return Ok(None);
        };

        if context.no_color {
            Ok(Some(format!("{hostname} ")))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Cyan, false);
            Ok(Some(format!(
                "{}{}{} ",
                style.start_codes(),
                hostname,
                AnsiStyle::RESET
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname_renders() {
        let module = HostnameModule::new();
        let context = ModuleContext::default();

        let result = module.render(&context).unwrap();
        assert!(result.is_some(), "Hostname module should render something");

        let output = result.unwrap();
        assert!(!output.is_empty());
    }
}
