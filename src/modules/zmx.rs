use crate::module_trait::{Module, ModuleContext};
use crate::modules::utils::sanitize_display_text;

pub(crate) struct ZmxModule;

impl Module for ZmxModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let session = sanitize_display_text(context.zmx_session.as_deref()?);

        if context.no_color {
            Some(format!("[+zmx: {session}] "))
        } else {
            use crate::style::{AnsiStyle, Color};

            Some(format!(
                "{}[+zmx: {}]{} ",
                AnsiStyle::new(Color::BrightYellow, false).start_codes(),
                session,
                AnsiStyle::RESET,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(session: Option<&str>) -> Option<String> {
        ZmxModule.render(&ModuleContext {
            no_color: true,
            zmx_session: session.map(str::to_owned),
            ..ModuleContext::default()
        })
    }

    #[test]
    fn no_zmx_segment_is_rendered_outside_a_session() {
        assert_eq!(render(None), None);
    }

    #[test]
    fn session_name_is_rendered_from_the_context_snapshot() {
        assert_eq!(render(Some("work")), Some("[+zmx: work] ".to_owned()));
    }

    #[test]
    fn control_characters_in_the_session_name_are_escaped() {
        assert_eq!(
            render(Some("work\x1b[31m")),
            Some("[+zmx: work\\u{1b}[31m] ".to_owned())
        );
    }
}
