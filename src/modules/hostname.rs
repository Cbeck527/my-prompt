use crate::module_trait::{Module, ModuleContext};
use crate::modules::utils::sanitize_display_text;
use whoami::hostname;

pub(crate) struct HostnameModule;

impl Default for HostnameModule {
    fn default() -> Self {
        Self::new()
    }
}

impl HostnameModule {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Module for HostnameModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let actual_hostname = hostname().ok()?;
        let hostname = sanitize_display_text(&actual_hostname);

        if context.no_color {
            Some(format!("{hostname} "))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Cyan, false);
            Some(format!(
                "{}{}{} ",
                style.start_codes(),
                hostname,
                AnsiStyle::RESET
            ))
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

        let result = module.render(&context);
        assert!(result.is_some(), "Hostname module should render something");

        let output = result.unwrap();
        assert!(!output.is_empty());
    }
}
