use crate::module_trait::{Module, ModuleContext};

pub(crate) struct EnvsModule;

impl Module for EnvsModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let mut text = String::new();
        if context.environments.nix_shell {
            text.push_str("+nix");
        }
        if context.environments.virtual_env {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str("+virtualenv");
        }

        if text.is_empty() {
            return None;
        }

        if context.no_color {
            Some(format!("[{text}] "))
        } else {
            use crate::style::{AnsiStyle, Color};

            Some(format!(
                "{}[{}]{} ",
                AnsiStyle::new(Color::Magenta, false).start_codes(),
                text,
                AnsiStyle::RESET,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_trait::EnvironmentState;

    fn render(environments: EnvironmentState) -> Option<String> {
        EnvsModule.render(&ModuleContext {
            no_color: true,
            environments,
            ..ModuleContext::default()
        })
    }

    #[test]
    fn no_environment_segment_is_rendered_when_both_are_absent() {
        assert_eq!(render(EnvironmentState::default()), None);
    }

    #[test]
    fn nix_shell_is_rendered_from_the_context_snapshot() {
        assert_eq!(
            render(EnvironmentState {
                nix_shell: true,
                virtual_env: false,
            }),
            Some("[+nix] ".to_owned())
        );
    }

    #[test]
    fn virtual_environment_is_rendered_from_the_context_snapshot() {
        assert_eq!(
            render(EnvironmentState {
                nix_shell: false,
                virtual_env: true,
            }),
            Some("[+virtualenv] ".to_owned())
        );
    }

    #[test]
    fn both_environments_preserve_the_personal_display_order() {
        assert_eq!(
            render(EnvironmentState {
                nix_shell: true,
                virtual_env: true,
            }),
            Some("[+nix +virtualenv] ".to_owned())
        );
    }
}
