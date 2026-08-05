use clap::ValueEnum;

use crate::claude::ClaudeSession;

/// Backend for Git operations such as branch and status discovery.
#[derive(ValueEnum, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum GitBackend {
    /// Shell out to the Git binary.
    #[default]
    Binary,
    /// Use the pure-Rust gix library.
    Gix,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EnvironmentState {
    pub(crate) nix_shell: bool,
    pub(crate) virtual_env: bool,
}

#[derive(Debug, Default)]
pub(crate) struct ModuleContext {
    pub(crate) exit_code: Option<i32>,
    pub(crate) no_color: bool,
    pub(crate) claude_session: Option<ClaudeSession>,
    pub(crate) git_backend: GitBackend,
    pub(crate) direnv_status_json: Option<String>,
    pub(crate) environments: EnvironmentState,
}

pub(crate) trait Module: Send + Sync {
    /// Renders module output based on the current context.
    ///
    /// Returns `None` when the module is not applicable or cannot obtain trustworthy data.
    fn render(&self, context: &ModuleContext) -> Option<String>;
}
