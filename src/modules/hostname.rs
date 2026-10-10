use crate::module_trait::{Module, ModuleContext};
use crate::modules::utils::sanitize_display_text;

pub(crate) struct HostnameModule;

impl Module for HostnameModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let hostname = sanitize_display_text(context.hostname.as_deref()?);

        if context.no_color {
            Some(format!("@{hostname} "))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Green, true);
            Some(format!(
                "{}@{}{} ",
                style.start_codes(),
                hostname,
                AnsiStyle::RESET
            ))
        }
    }
}

/// Shortens a hostname to its first label, dropping any domain suffix.
///
/// Returns `None` when no usable label remains.
pub(crate) fn short_hostname(hostname: &str) -> Option<&str> {
    hostname.split('.').next().filter(|label| !label.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(hostname: Option<&str>) -> Option<String> {
        HostnameModule.render(&ModuleContext {
            no_color: true,
            hostname: hostname.map(str::to_owned),
            ..ModuleContext::default()
        })
    }

    #[test]
    fn no_hostname_segment_is_rendered_outside_ssh() {
        assert_eq!(render(None), None);
    }

    #[test]
    fn hostname_is_rendered_from_the_context_snapshot() {
        assert_eq!(
            render(Some("remote-host")),
            Some("@remote-host ".to_owned())
        );
    }

    #[test]
    fn short_hostname_drops_the_domain_suffix() {
        assert_eq!(short_hostname("remote-host"), Some("remote-host"));
        assert_eq!(
            short_hostname("remote-host.example.com"),
            Some("remote-host")
        );
        assert_eq!(short_hostname("laptop.local"), Some("laptop"));
    }

    #[test]
    fn short_hostname_rejects_empty_labels() {
        assert_eq!(short_hostname(""), None);
        assert_eq!(short_hostname(".example.com"), None);
    }
}
