use crate::error::Result;

#[derive(Debug, Default, Clone)]
pub struct ModuleContext {
    pub exit_code: Option<i32>,
    pub no_color: bool,
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
