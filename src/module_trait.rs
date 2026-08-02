use crate::error::Result;
use clap::ValueEnum;

/// Backend for git operations (branch name, status).
#[derive(ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GitBackend {
    /// Shell out to git binary
    #[default]
    Binary,
    /// Use gix library (pure Rust)
    Gix,
}

/// Session information from Claude Code, passed via stdin JSON.
#[derive(Debug, Clone)]
pub struct ClaudeSession {
    pub model_name: String,
    pub context_used: u64,
    pub context_total: u64,
    pub percentage: u8,
}

#[derive(Debug, Default, Clone)]
pub struct ModuleContext {
    pub exit_code: Option<i32>,
    pub no_color: bool,
    pub claude_session: Option<ClaudeSession>,
    pub git_backend: GitBackend,
    pub direnv_status_json: Option<String>,
}

pub trait Module: Send + Sync {
    /// Renders module output based on the current context.
    ///
    /// Returns `None` if the module is not applicable (e.g., git module outside a repo).
    ///
    /// # Errors
    ///
    /// Returns `PromptError::ExternalCommandFailed` if an external command fails unexpectedly.
    /// Note: Missing binaries should return `Ok(None)`, not an error.
    fn render(&self, context: &ModuleContext) -> Result<Option<String>>;
}
