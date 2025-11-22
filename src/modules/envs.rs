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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// RAII guard that removes an environment variable when dropped,
    /// ensuring cleanup even if test panics
    struct EnvVarGuard {
        key: &'static str,
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                env::remove_var(self.key);
            }
        }
    }

    impl EnvVarGuard {
        fn new(key: &'static str, value: &str) -> Self {
            unsafe {
                env::set_var(key, value);
            }
            Self { key }
        }
    }

    #[test]
    #[serial]
    fn test_without_vars() {
        let module = EnvsModule::new();
        let context = ModuleContext::default();

        let result = module.render(&context).unwrap();
        assert!(
            result.is_none(),
            "Envs module shouldn't render without any special vars"
        );
    }

    #[test]
    #[serial]
    fn test_with_nix() {
        let _guard = EnvVarGuard::new("IN_NIX_SHELL", "test");

        let module = EnvsModule::new();
        let context = ModuleContext {
            exit_code: Some(0),
            no_color: true,
        };

        let result = module.render(&context).unwrap();
        assert_eq!(
            result,
            Some("[+nix]".to_string()),
            "Envs module should render in nix shell"
        );
    }

    #[test]
    #[serial]
    fn test_with_virtualenv() {
        let _guard = EnvVarGuard::new("VIRTUAL_ENV", "test");

        let module = EnvsModule::new();
        let context = ModuleContext {
            exit_code: Some(0),
            no_color: true,
        };

        let result = module.render(&context).unwrap();
        assert_eq!(
            result,
            Some("[+virtualenv]".to_string()),
            "Envs module should render in virtualenv"
        );
    }

    #[test]
    #[serial]
    fn test_with_all_special_vars() {
        let _guards: Vec<_> = SPECIAL_ENV_VARS
            .iter()
            .map(|var| EnvVarGuard::new(var.name, "test"))
            .collect();

        let module = EnvsModule::new();
        let context = ModuleContext {
            exit_code: Some(0),
            no_color: true,
        };

        let result = module.render(&context).unwrap();
        assert_eq!(
            result,
            Some("[+nix +virtualenv]".to_string()),
            "Envs module should render all special vars"
        );
    }
}
