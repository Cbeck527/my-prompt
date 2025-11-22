use std::env;

use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};
pub struct EnvsModule;

impl Default for EnvsModule {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvsModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

struct SpecialEnvVars {
    name: &'static str,
    display_name: &'static str,
}

const SPECIAL_ENV_VARS: &[SpecialEnvVars] = &[
    SpecialEnvVars {
        name: "IN_NIX_SHELL",
        display_name: "nix",
    },
    SpecialEnvVars {
        name: "VIRTUAL_ENV",
        display_name: "virtualenv",
    },
];

impl Module for EnvsModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let present_vars: Vec<_> = SPECIAL_ENV_VARS
            .iter()
            .filter(|v| env::var_os(v.name).is_some())
            .map(|v| format!("+{}", v.display_name))
            .collect();

        if present_vars.is_empty() {
            return Ok(None);
        }

        let text = present_vars.join(" ");

        if context.no_color {
            Ok(Some(format!("[{text}]")))
        } else {
            use crate::style::{AnsiStyle, Color};

            Ok(Some(format!(
                "{}[{}]{} ",
                AnsiStyle::new(Color::Magenta, false).start_codes(),
                text,
                AnsiStyle::RESET,
            )))
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_direnv_without_envrc() {
//         let module = NixModule::new();
//         let context = ModuleContext::default();
//
//         let result = module.render(&context).unwrap();
//         assert!(
//             result.is_none(),
//             "Direnv module shouldn't render without .envrc"
//         );
//     }
// }
