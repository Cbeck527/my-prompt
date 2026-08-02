use std::env;

use crate::module_trait::{Module, ModuleContext};
pub(crate) struct EnvsModule;

impl Default for EnvsModule {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvsModule {
    #[must_use]
    pub(crate) fn new() -> Self {
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
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let present_vars: Vec<_> = SPECIAL_ENV_VARS
            .iter()
            .filter(|v| env::var_os(v.name).is_some())
            .map(|v| format!("+{}", v.display_name))
            .collect();

        if present_vars.is_empty() {
            return None;
        }

        let text = present_vars.join(" ");

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
    use serial_test::serial;
    use std::ffi::OsString;

    /// RAII guard that restores an environment variable when dropped.
    struct EnvVarGuard {
        key: &'static str,
        previous_value: Option<OsString>,
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(value) = self.previous_value.take() {
                    env::set_var(self.key, value);
                } else {
                    env::remove_var(self.key);
                }
            }
        }
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous_value = env::var_os(key);
            unsafe {
                env::set_var(key, value);
            }
            Self {
                key,
                previous_value,
            }
        }

        fn unset(key: &'static str) -> Self {
            let previous_value = env::var_os(key);
            unsafe {
                env::remove_var(key);
            }
            Self {
                key,
                previous_value,
            }
        }
    }

    fn clear_special_env_vars() -> Vec<EnvVarGuard> {
        SPECIAL_ENV_VARS
            .iter()
            .map(|var| EnvVarGuard::unset(var.name))
            .collect()
    }

    #[test]
    #[serial]
    fn test_without_vars() {
        let _guards = clear_special_env_vars();

        let module = EnvsModule::new();
        let context = ModuleContext::default();

        let result = module.render(&context);
        assert!(
            result.is_none(),
            "Envs module shouldn't render without any special vars"
        );
    }

    #[test]
    #[serial]
    fn test_with_nix() {
        let _clear_guards = clear_special_env_vars();
        let _guard = EnvVarGuard::set("IN_NIX_SHELL", "test");

        let module = EnvsModule::new();
        let context = ModuleContext {
            exit_code: Some(0),
            no_color: true,
            ..Default::default()
        };

        let result = module.render(&context);
        assert_eq!(
            result,
            Some("[+nix] ".to_string()),
            "Envs module should render in nix shell"
        );
    }

    #[test]
    #[serial]
    fn test_with_virtualenv() {
        let _clear_guards = clear_special_env_vars();
        let _guard = EnvVarGuard::set("VIRTUAL_ENV", "test");

        let module = EnvsModule::new();
        let context = ModuleContext {
            exit_code: Some(0),
            no_color: true,
            ..Default::default()
        };

        let result = module.render(&context);
        assert_eq!(
            result,
            Some("[+virtualenv] ".to_string()),
            "Envs module should render in virtualenv"
        );
    }

    #[test]
    #[serial]
    fn test_with_all_special_vars() {
        let _clear_guards = clear_special_env_vars();
        let _guards: Vec<_> = SPECIAL_ENV_VARS
            .iter()
            .map(|var| EnvVarGuard::set(var.name, "test"))
            .collect();

        let module = EnvsModule::new();
        let context = ModuleContext {
            exit_code: Some(0),
            no_color: true,
            ..Default::default()
        };

        let result = module.render(&context);
        assert_eq!(
            result,
            Some("[+nix +virtualenv] ".to_string()),
            "Envs module should render all special vars"
        );
    }
}
