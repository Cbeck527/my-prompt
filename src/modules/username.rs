use crate::module_trait::{Host, Module, ModuleContext};
use crate::modules::utils::sanitize_display_text;
use whoami::username;

pub(crate) struct UsernameModule;

impl Module for UsernameModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        // The Coder segment identifies the workspace in place of the user.
        if matches!(context.host, Host::Coder(_)) {
            return None;
        }

        let actual_username = username().unwrap_or_else(|_| "unknown".to_owned());
        let display_name = match actual_username.as_str() {
            "christopher.becker" => "chris",
            _ => &actual_username,
        };
        let display_name = sanitize_display_text(display_name);
        // The hostname module follows directly as `@host` when present.
        let separator = if matches!(context.host, Host::Ssh(_)) {
            ""
        } else {
            " "
        };

        if context.no_color {
            Some(format!("{display_name}{separator}"))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::Green, false);
            Some(format!(
                "{}{}{}{separator}",
                style.start_codes(),
                display_name,
                AnsiStyle::RESET
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_renders_a_nonempty_username() {
        let module = UsernameModule;
        let context = ModuleContext::default();

        let result = module.render(&context);
        assert!(result.is_some());

        let output = result.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn trailing_space_yields_to_the_hostname_segment() {
        let render = |host: Host| {
            UsernameModule
                .render(&ModuleContext {
                    no_color: true,
                    host,
                    ..ModuleContext::default()
                })
                .unwrap()
        };

        assert!(render(Host::Local).ends_with(' '));
        assert!(!render(Host::Ssh("remote-host".to_owned())).ends_with(' '));
    }

    #[test]
    fn no_username_segment_is_rendered_in_a_coder_workspace() {
        assert_eq!(
            UsernameModule.render(&ModuleContext {
                host: Host::Coder("wumbo".to_owned()),
                ..ModuleContext::default()
            }),
            None
        );
    }
}
