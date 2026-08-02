use clap::ValueEnum;

/// Backend for git operations (branch name, status).
#[derive(ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GitBackend {
    /// Shell out to git binary
    #[default]
    Binary,
    /// Use gix library (pure Rust)
    Gix,
}

/// Session information from Claude Code, passed via stdin JSON.
#[derive(Debug, Clone)]
pub(crate) struct ClaudeSession {
    pub(crate) model_name: String,
    pub(crate) context_used: u64,
    pub(crate) context_total: u64,
    pub(crate) percentage: u8,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ModuleContext {
    pub(crate) exit_code: Option<i32>,
    pub(crate) no_color: bool,
    pub(crate) claude_session: Option<ClaudeSession>,
    pub(crate) git_backend: GitBackend,
    pub(crate) direnv_status_json: Option<String>,
}

pub(crate) trait Module: Send + Sync {
    /// Renders module output based on the current context.
    ///
    /// Returns `None` when the module is not applicable or cannot obtain trustworthy data.
    fn render(&self, context: &ModuleContext) -> Option<String>;
}
