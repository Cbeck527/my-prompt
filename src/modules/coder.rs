use crate::module_trait::{Host, Module, ModuleContext};
use crate::modules::utils::sanitize_display_text;

pub(crate) struct CoderModule;

impl Module for CoderModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let Host::Coder(workspace) = &context.host else {
            return None;
        };
        let workspace = sanitize_display_text(workspace);

        if context.no_color {
            Some(format!("[coder: {workspace}] "))
        } else {
            use crate::style::{AnsiStyle, Color};

            Some(format!(
                "{}[coder: {}]{} ",
                AnsiStyle::new(Color::Green, true).start_codes(),
                workspace,
                AnsiStyle::RESET,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(host: Host) -> Option<String> {
        CoderModule.render(&ModuleContext {
            no_color: true,
            host,
            ..ModuleContext::default()
        })
    }

    #[test]
    fn no_coder_segment_is_rendered_outside_a_workspace() {
        assert_eq!(render(Host::Local), None);
        assert_eq!(render(Host::Ssh("remote-host".to_owned())), None);
    }

    #[test]
    fn workspace_name_is_rendered_from_the_context_snapshot() {
        assert_eq!(
            render(Host::Coder("wumbo".to_owned())),
            Some("[coder: wumbo] ".to_owned())
        );
    }

    #[test]
    fn control_characters_in_the_workspace_name_are_escaped() {
        assert_eq!(
            render(Host::Coder("wumbo\x1b[31m".to_owned())),
            Some("[coder: wumbo\\u{1b}[31m] ".to_owned())
        );
    }
}
